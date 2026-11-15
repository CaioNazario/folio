//! Operations on documents, by id (spec 04). All follow spec 01: disk first,
//! never overwriting.

use std::ffi::OsString;
use std::fs;
use std::path::Path;

use rusqlite::Connection;
use uuid::Uuid;

use super::Document;
use crate::integrity::{delete_to_trash, move_with_suffix, rename_no_replace};
use crate::paths::RelativePath;
use crate::workspace::ops::entry_path;
use crate::workspace::{Workspace, validate_name};
use crate::{Error, Result};

impl Workspace {
    /// Renames the document to `new_stem`, keeping its extension so a rename
    /// never changes the format.
    pub fn rename_document(&self, id: Uuid, new_stem: &str) -> Result<Document> {
        let doc = self.document_on_disk(id)?;
        let new_name = with_extension_of(&validate_name(new_stem)?, &doc.path);
        let new_path = entry_path(doc.path.parent().as_ref(), &new_name)?;
        rename_no_replace(
            &doc.path.resolve(self.root()),
            &new_path.resolve(self.root()),
        )?;
        self.update_index_after_disk(|conn| set_path(conn, id, &new_path));
        self.document(id)
    }

    /// Moves the document into `parent` (the root for `None`), adding
    /// ` (n)` when the name is taken there (spec 01).
    pub fn move_document(&self, id: Uuid, parent: Option<&RelativePath>) -> Result<Document> {
        let doc = self.document_on_disk(id)?;
        self.ensure_movable(&doc, parent)?;
        let dest_dir = parent.map_or_else(|| self.root().to_path_buf(), |p| p.resolve(self.root()));
        let moved = move_with_suffix(&doc.path.resolve(self.root()), &dest_dir)?;
        let new_path = RelativePath::from_absolute(self.root(), &moved)?;
        self.update_index_after_disk(|conn| set_path(conn, id, &new_path));
        self.document(id)
    }

    /// Sends the document to the system trash. It leaves the interface and
    /// stays missing in the index, so restoring it brings back its id
    /// (spec 05).
    pub fn trash_document(&self, id: Uuid) -> Result<()> {
        let doc = self.document_on_disk(id)?;
        delete_to_trash(&*self.trash, &doc.path.resolve(self.root()))?;
        let now = self.clock.now_millis();
        self.update_index_after_disk(|conn| mark_missing(conn, id, now));
        Ok(())
    }

    /// The document with `id`, as long as its file (or link) is still on disk.
    fn document_on_disk(&self, id: Uuid) -> Result<Document> {
        let doc = self.document(id)?;
        if fs::symlink_metadata(doc.path.resolve(self.root())).is_err() {
            return Err(Error::DocumentNotFound(id));
        }
        Ok(doc)
    }

    /// A symlink with a relative target would break in another directory.
    fn ensure_movable(&self, doc: &Document, parent: Option<&RelativePath>) -> Result<()> {
        if doc.path.parent().as_ref() == parent {
            return Ok(());
        }
        let target = fs::read_link(doc.path.resolve(self.root()));
        if target.is_ok_and(|target| target.is_relative()) {
            return Err(Error::RelativeSymlink(doc.path.clone()));
        }
        Ok(())
    }
}

/// `stem` plus the extension of `path`, exactly as it is on disk.
fn with_extension_of(stem: &str, path: &RelativePath) -> OsString {
    let mut name = OsString::from(stem);
    if let Some(extension) = Path::new(path.file_name()).extension() {
        name.push(".");
        name.push(extension);
    }
    name
}

fn mark_missing(conn: &mut Connection, id: Uuid, since: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE documents SET missing_since = ?2 WHERE id = ?1",
        (id.to_string(), since),
    )
    .map(drop)
}

fn set_path(conn: &mut Connection, id: Uuid, path: &RelativePath) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE documents SET path = ?2 WHERE id = ?1",
        (id.to_string(), path),
    )
    .map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::test_docs::write_doc;
    use crate::workspace::InvalidName;
    use crate::workspace::test_env::TestEnv;

    fn rel(text: &str) -> RelativePath {
        RelativePath::from_bytes(text).unwrap()
    }

    /// Writes and indexes a document, returning its id.
    fn indexed(workspace: &Workspace, path: &str) -> Uuid {
        let path = write_doc(workspace, path, b"%PDF-1.7\n");
        workspace.index_file(&path).unwrap().unwrap().id
    }

    #[test]
    fn rename_document_keeps_extension_and_id() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = indexed(&workspace, "Faculdade/Trabalho.PDF");

        let renamed = workspace.rename_document(id, " Trabalho final ").unwrap();

        assert_eq!(
            (renamed.id, renamed.path),
            (id, rel("Faculdade/Trabalho final.PDF"))
        );
        assert!(
            workspace
                .root()
                .join("Faculdade/Trabalho final.PDF")
                .is_file()
        );
    }

    #[test]
    fn rename_document_rejects_invalid_name() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = indexed(&workspace, "Trabalho.pdf");

        let result = workspace.rename_document(id, "a/b");

        assert!(matches!(
            result,
            Err(Error::InvalidName(InvalidName::ContainsSlash))
        ));
    }

    #[test]
    fn rename_document_fails_with_already_exists() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = indexed(&workspace, "Trabalho.pdf");
        write_doc(&workspace, "Final.pdf", b"outro");

        let result = workspace.rename_document(id, "Final");

        assert!(matches!(result, Err(Error::AlreadyExists(_))));
        assert_eq!(workspace.document(id).unwrap().path, rel("Trabalho.pdf"));
    }

    #[test]
    fn move_document_keeps_id_and_updates_path() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = indexed(&workspace, "Faculdade/Trabalho.pdf");
        env.dir("Folio/Arquivo");

        let moved = workspace.move_document(id, Some(&rel("Arquivo"))).unwrap();

        assert_eq!((moved.id, moved.path), (id, rel("Arquivo/Trabalho.pdf")));
        assert!(workspace.root().join("Arquivo/Trabalho.pdf").is_file());
    }

    #[test]
    fn move_document_adds_suffix_when_name_taken() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = indexed(&workspace, "Faculdade/Trabalho.pdf");
        write_doc(&workspace, "Arquivo/Trabalho.pdf", b"o que ja estava");

        let moved = workspace.move_document(id, Some(&rel("Arquivo"))).unwrap();

        assert_eq!(moved.path, rel("Arquivo/Trabalho (1).pdf"));
        let kept = std::fs::read(workspace.root().join("Arquivo/Trabalho.pdf")).unwrap();
        assert_eq!(kept, b"o que ja estava");
    }

    /// A symlink at `link` (inside the workspace) pointing to `target`, indexed.
    fn indexed_link(env: &TestEnv, workspace: &Workspace, link: &str, target: &str) -> Uuid {
        let real = env.dir("Downloads").join("Boleto.pdf");
        std::fs::write(&real, b"%PDF-1.4\n").unwrap();
        let full = workspace.root().join(link);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(target.replace("{real}", real.to_str().unwrap()), &full)
            .unwrap();
        workspace.index_file(&rel(link)).unwrap().unwrap().id
    }

    #[test]
    fn moving_relative_symlink_to_other_dir_fails_with_relative_symlink() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = indexed_link(
            &env,
            &workspace,
            "Pessoal/Boleto.pdf",
            "../../Downloads/Boleto.pdf",
        );
        env.dir("Folio/Contas");

        let result = workspace.move_document(id, Some(&rel("Contas")));

        assert!(
            matches!(result, Err(Error::RelativeSymlink(path)) if path == rel("Pessoal/Boleto.pdf"))
        );
        assert!(workspace.root().join("Pessoal/Boleto.pdf").is_symlink());
    }

    #[test]
    fn renaming_relative_symlink_in_same_dir_is_allowed() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = indexed_link(
            &env,
            &workspace,
            "Pessoal/Boleto.pdf",
            "../../Downloads/Boleto.pdf",
        );

        let renamed = workspace.rename_document(id, "Boleto de novembro").unwrap();

        let link = workspace.root().join("Pessoal/Boleto de novembro.pdf");
        assert_eq!((renamed.valid, link.is_symlink()), (true, true));
    }

    #[test]
    fn moving_symlink_moves_the_link_and_leaves_target_untouched() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = indexed_link(&env, &workspace, "Pessoal/Boleto.pdf", "{real}");
        env.dir("Folio/Contas");

        workspace.move_document(id, Some(&rel("Contas"))).unwrap();

        assert!(workspace.root().join("Contas/Boleto.pdf").is_symlink());
        assert!(env.home().join("Downloads/Boleto.pdf").is_file());
    }

    #[test]
    fn trashing_document_sends_file_to_trash_and_hides_it() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = indexed(&workspace, "Faculdade/Trabalho.pdf");

        workspace.trash_document(id).unwrap();

        assert_eq!(
            env.trash.sent(),
            [workspace.root().join("Faculdade/Trabalho.pdf")]
        );
        assert_eq!(
            workspace.list_documents(Some(&rel("Faculdade"))).unwrap(),
            []
        );
    }

    #[test]
    fn operation_on_file_gone_from_disk_fails_with_document_not_found() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = indexed(&workspace, "Trabalho.pdf");
        std::fs::remove_file(workspace.root().join("Trabalho.pdf")).unwrap();

        let result = workspace.rename_document(id, "Outro");

        assert!(matches!(result, Err(Error::DocumentNotFound(missing)) if missing == id));
    }

    #[test]
    fn renaming_dir_updates_paths_of_documents_inside_keeping_ids() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let direct = indexed(&workspace, "Faculdade/Plano.pdf");
        let nested = indexed(&workspace, "Faculdade/TCC/Trabalho.pdf");

        workspace
            .rename_dir(&rel("Faculdade"), "Universidade")
            .unwrap();

        assert_eq!(
            workspace.document(direct).unwrap().path,
            rel("Universidade/Plano.pdf")
        );
        assert_eq!(
            workspace.document(nested).unwrap().path,
            rel("Universidade/TCC/Trabalho.pdf")
        );
    }

    #[test]
    fn moving_dir_updates_paths_of_documents_inside_keeping_ids() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = indexed(&workspace, "Faculdade/TCC/Trabalho.pdf");
        env.dir("Folio/Arquivo");

        workspace
            .move_dir(&rel("Faculdade/TCC"), Some(&rel("Arquivo")))
            .unwrap();

        assert_eq!(
            workspace.document(id).unwrap().path,
            rel("Arquivo/TCC/Trabalho.pdf")
        );
    }
}
