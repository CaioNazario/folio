//! The last destination of "Mover para…" and of an import, shown first
//! in the destination dropdown (spec 09, "Interface").

use rusqlite::{Connection, OptionalExtension};

use crate::Result;
use crate::paths::RelativePath;
use crate::workspace::Workspace;

impl Workspace {
    /// The project or folder that received the last move from the inbox or
    /// import; `None` before any, or when it no longer exists.
    pub fn last_destination(&self) -> Result<Option<RelativePath>> {
        let stored: Option<RelativePath> = self
            .conn()
            .query_row("SELECT path FROM last_destination", [], |row| row.get(0))
            .optional()
            .map_err(|e| self.db_error(e))?;
        // Gone from disk, outside Folio: the disk is the truth (spec 01).
        Ok(stored.filter(|dest| dest.resolve(self.root()).is_dir()))
    }
}

/// Records `dest` as the last destination, replacing the one before.
pub(crate) fn record_last_destination(
    conn: &mut Connection,
    dest: &RelativePath,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO last_destination (id, path) VALUES (1, ?1)
         ON CONFLICT (id) DO UPDATE SET path = excluded.path",
        [dest],
    )
    .map(drop)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::workspace::test_env::TestEnv;

    /// A workspace whose last destination is the project `Faculdade`.
    fn moved_into_faculdade(env: &TestEnv) -> Workspace {
        let workspace = env.open_workspace();
        fs::create_dir(workspace.root().join("Faculdade")).unwrap();
        let inbox = env.dir("Downloads");
        fs::write(inbox.join("Edital.txt"), "edital").unwrap();
        let dest = RelativePath::from_bytes("Faculdade").unwrap();
        workspace
            .move_from_inbox(&inbox, b"Edital.txt", &dest)
            .unwrap();
        workspace
    }

    #[test]
    fn last_destination_follows_folder_renamed_by_folio() {
        let env = TestEnv::new();
        let workspace = moved_into_faculdade(&env);
        let faculdade = RelativePath::from_bytes("Faculdade").unwrap();

        workspace.rename_dir(&faculdade, "Universidade").unwrap();

        let renamed = RelativePath::from_bytes("Universidade").unwrap();
        assert_eq!(workspace.last_destination().unwrap(), Some(renamed));
    }

    #[test]
    fn last_destination_is_none_when_folder_no_longer_exists() {
        let env = TestEnv::new();
        let workspace = moved_into_faculdade(&env);

        fs::remove_dir_all(workspace.root().join("Faculdade")).unwrap();

        assert_eq!(workspace.last_destination().unwrap(), None);
    }
}
