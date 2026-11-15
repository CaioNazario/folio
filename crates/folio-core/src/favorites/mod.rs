//! Favorite documents, projects and folders of the workspace (spec 06).

use std::fs;

use rusqlite::Row;
use uuid::Uuid;

use crate::db::columns::uuid_column;
use crate::paths::RelativePath;
use crate::workspace::Workspace;
use crate::{Error, Result};

/// What a favorite points to: a document by id, a directory by path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FavoriteTarget {
    Document(Uuid),
    Project(RelativePath),
    Folder(RelativePath),
}

/// A favorite as the sidebar shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Favorite {
    pub id: Uuid,
    /// File or directory name; invalid UTF-8 bytes show as U+FFFD.
    pub name: String,
    pub target: FavoriteTarget,
}

impl Workspace {
    /// Marks the document `id` as a favorite.
    pub fn add_favorite_document(&self, id: Uuid) -> Result<()> {
        self.document(id)?;
        self.insert_favorite(Some(id.to_string()), None)
    }

    /// Marks the project or folder at `path` as a favorite.
    pub fn add_favorite_dir(&self, path: &RelativePath) -> Result<()> {
        let full = path.resolve(self.root());
        fs::metadata(&full).map_err(|e| Error::io(&full, e))?;
        self.insert_favorite(None, Some(path))
    }

    /// Takes the favorite `id` off the list; nothing happens on disk.
    pub fn remove_favorite(&self, id: Uuid) -> Result<()> {
        self.conn()
            .execute("DELETE FROM favorites WHERE id = ?1", [id.to_string()])
            .map(drop)
            .map_err(|e| self.db_error(e))
    }

    /// Favorites in the order they were added, without missing documents.
    pub fn favorites(&self) -> Result<Vec<Favorite>> {
        let conn = self.conn();
        let query = || -> rusqlite::Result<Vec<Favorite>> {
            conn.prepare(LIST)?.query_map([], favorite_row)?.collect()
        };
        query().map_err(|e| self.db_error(e))
    }

    fn insert_favorite(&self, document: Option<String>, path: Option<&RelativePath>) -> Result<()> {
        let params = (
            self.ids.new_id().to_string(),
            document,
            path,
            self.clock.now_millis(),
        );
        self.conn()
            .execute(INSERT, params)
            .map(drop)
            .map_err(|e| self.db_error(e))
    }
}

/// A favorite added again keeps its place: the unique indexes refuse it.
const INSERT: &str = "INSERT INTO favorites (id, document_id, path, added_at)
     VALUES (?1, ?2, ?3, ?4) ON CONFLICT DO NOTHING";

/// The path of a document favorite comes from its document, which must be
/// on disk. Ties in `added_at` keep the insertion order through `rowid`.
const LIST: &str = "SELECT f.id, f.document_id, coalesce(d.path, f.path)
     FROM favorites f LEFT JOIN documents d ON d.id = f.document_id
     WHERE d.missing_since IS NULL
     ORDER BY f.added_at, f.rowid";

fn favorite_row(row: &Row<'_>) -> rusqlite::Result<Favorite> {
    let path: RelativePath = row.get(2)?;
    let target = match row.get::<_, Option<String>>(1)? {
        Some(_) => FavoriteTarget::Document(uuid_column(row, 1)?),
        None => dir_target(path.clone()),
    };
    Ok(Favorite {
        id: uuid_column(row, 0)?,
        name: path.file_name().to_string_lossy().into_owned(),
        target,
    })
}

/// A directory at the root is a project; any deeper one is a folder.
fn dir_target(path: RelativePath) -> FavoriteTarget {
    match path.parent() {
        None => FavoriteTarget::Project(path),
        Some(_) => FavoriteTarget::Folder(path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::test_docs::write_doc;
    use crate::indexing::{MISSING_RETENTION_MS, ScanOptions};
    use crate::workspace::test_env::TestEnv;

    fn indexed(workspace: &Workspace, path: &str) -> Uuid {
        let path = write_doc(workspace, path, b"text");
        workspace.index_file(&path).unwrap().unwrap().id
    }

    fn rel(text: &str) -> RelativePath {
        RelativePath::from_bytes(text).unwrap()
    }

    fn names(workspace: &Workspace) -> Vec<String> {
        let favorites = workspace.favorites().unwrap();
        favorites
            .into_iter()
            .map(|favorite| favorite.name)
            .collect()
    }

    #[test]
    fn favorites_are_listed_in_insertion_order() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let zeta = indexed(&workspace, "Faculdade/Zeta.txt");
        let alfa = indexed(&workspace, "Alfa.txt");

        workspace.add_favorite_document(zeta).unwrap();
        workspace.add_favorite_dir(&rel("Faculdade")).unwrap();
        workspace.add_favorite_document(alfa).unwrap();

        assert_eq!(names(&workspace), ["Zeta.txt", "Faculdade", "Alfa.txt"]);
    }

    #[test]
    fn adding_existing_favorite_again_changes_nothing() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Faculdade/Nota.txt");
        workspace.add_favorite_document(doc).unwrap();
        workspace.add_favorite_dir(&rel("Faculdade")).unwrap();
        let before = workspace.favorites().unwrap();
        env.clock.set(5_000);

        workspace.add_favorite_dir(&rel("Faculdade")).unwrap();
        workspace.add_favorite_document(doc).unwrap();

        assert_eq!(workspace.favorites().unwrap(), before);
    }

    #[test]
    fn removing_favorite_takes_it_off_the_list() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Faculdade/Nota.txt");
        workspace.add_favorite_document(doc).unwrap();
        workspace.add_favorite_dir(&rel("Faculdade")).unwrap();
        let first = workspace.favorites().unwrap()[0].id;

        workspace.remove_favorite(first).unwrap();

        assert_eq!(names(&workspace), ["Faculdade"]);
    }

    #[test]
    fn favorites_report_kind_name_and_target() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Faculdade/Prova/Nota.txt");
        workspace.add_favorite_document(doc).unwrap();
        workspace.add_favorite_dir(&rel("Faculdade")).unwrap();
        workspace.add_favorite_dir(&rel("Faculdade/Prova")).unwrap();

        assert_eq!(
            shown(&workspace),
            [
                ("Nota.txt".to_string(), FavoriteTarget::Document(doc)),
                (
                    "Faculdade".to_string(),
                    FavoriteTarget::Project(rel("Faculdade"))
                ),
                (
                    "Prova".to_string(),
                    FavoriteTarget::Folder(rel("Faculdade/Prova"))
                ),
            ]
        );
    }

    #[test]
    fn favoriting_unknown_document_fails_with_document_not_found() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let unknown = Uuid::from_u128(99);

        let result = workspace.add_favorite_document(unknown);

        assert!(matches!(result, Err(Error::DocumentNotFound(id)) if id == unknown));
    }

    #[test]
    fn favoriting_missing_dir_fails_with_not_found() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();

        let result = workspace.add_favorite_dir(&rel("Inexistente"));

        assert!(matches!(
            result,
            Err(Error::Io { source, .. }) if source.kind() == std::io::ErrorKind::NotFound
        ));
    }

    #[test]
    fn document_favorite_survives_rename_and_move_through_folio() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Faculdade/Rascunho.txt");
        workspace.add_favorite_document(doc).unwrap();

        workspace.rename_document(doc, "Final").unwrap();
        workspace.move_document(doc, None).unwrap();

        assert_eq!(
            shown(&workspace),
            [("Final.txt".to_string(), FavoriteTarget::Document(doc))]
        );
    }

    /// `(name, target)` of each favorite, without the favorite's own id.
    fn shown(workspace: &Workspace) -> Vec<(String, FavoriteTarget)> {
        let favorites = workspace.favorites().unwrap();
        favorites.into_iter().map(|f| (f.name, f.target)).collect()
    }

    #[test]
    fn document_favorite_survives_move_outside_folio_with_reconnection() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Faculdade/Nota.txt");
        workspace.add_favorite_document(doc).unwrap();
        let root = workspace.root();
        fs::create_dir(root.join("Trabalho")).unwrap();
        fs::rename(
            root.join("Faculdade/Nota.txt"),
            root.join("Trabalho/Ata.txt"),
        )
        .unwrap();

        workspace.reconcile_dir(Some(&rel("Trabalho"))).unwrap();

        assert_eq!(
            shown(&workspace),
            [("Ata.txt".to_string(), FavoriteTarget::Document(doc))]
        );
    }

    #[test]
    fn document_favorite_is_hidden_while_missing() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Faculdade/Nota.txt");
        workspace.add_favorite_document(doc).unwrap();
        workspace.add_favorite_dir(&rel("Faculdade")).unwrap();

        workspace.trash_document(doc).unwrap();

        assert_eq!(names(&workspace), ["Faculdade"]);
    }

    #[test]
    fn document_favorite_leaves_with_purged_document() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Nota.txt");
        workspace.add_favorite_document(doc).unwrap();
        workspace.trash_document(doc).unwrap();
        env.clock.set(1_000 + MISSING_RETENTION_MS + 1);

        workspace.purge_expired_missing().unwrap();

        let rows: i64 = workspace
            .conn()
            .query_row("SELECT count(*) FROM favorites", [], |row| row.get(0))
            .unwrap();
        assert_eq!(rows, 0);
    }

    #[test]
    fn dir_favorite_follows_rename_and_move_through_folio() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        fs::create_dir_all(workspace.root().join("Faculdade/Prova")).unwrap();
        workspace.add_favorite_dir(&rel("Faculdade")).unwrap();
        workspace.add_favorite_dir(&rel("Faculdade/Prova")).unwrap();

        workspace.rename_dir(&rel("Faculdade"), "Escola").unwrap();
        workspace.move_dir(&rel("Escola/Prova"), None).unwrap();

        assert_eq!(
            shown(&workspace),
            [
                ("Escola".to_string(), FavoriteTarget::Project(rel("Escola"))),
                ("Prova".to_string(), FavoriteTarget::Project(rel("Prova"))),
            ]
        );
    }

    #[test]
    fn dir_favorite_renamed_outside_folio_is_removed_by_scan() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let root = workspace.root();
        fs::create_dir_all(root.join("Faculdade")).unwrap();
        fs::create_dir_all(root.join("Trabalho")).unwrap();
        workspace.add_favorite_dir(&rel("Faculdade")).unwrap();
        workspace.add_favorite_dir(&rel("Trabalho")).unwrap();
        fs::rename(root.join("Faculdade"), root.join("Escola")).unwrap();

        let options = ScanOptions {
            remove_temp_files: false,
        };
        workspace.full_scan(options, &mut |_| {}).unwrap();

        assert_eq!(names(&workspace), ["Trabalho"]);
    }
}
