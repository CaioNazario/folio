//! "Mover para…": a file of the inbox moved into the workspace (spec 09,
//! "Ações").

use std::path::{Path, PathBuf};

use super::destination::record_last_destination;
use crate::documents::Document;
use crate::integrity::move_with_suffix;
use crate::paths::{RelativePath, os_bytes};
use crate::workspace::{InvalidName, Workspace};
use crate::{Error, Result};

impl Workspace {
    /// Moves the file `file_name` of the inbox folder `inbox` into the
    /// project or folder `dest`, and returns it as indexed, with a new id.
    pub fn move_from_inbox(
        &self,
        inbox: &Path,
        file_name: &[u8],
        dest: &RelativePath,
    ) -> Result<Document> {
        let source = inbox_file(inbox, file_name)?;
        let moved = move_with_suffix(&source, &dest.resolve(self.root()))?;
        let path = RelativePath::from_absolute(self.root(), &moved)?;
        self.update_index_after_disk(|conn| record_last_destination(conn, dest));
        self.index_file(&path)?
            .ok_or_else(|| Error::io(&moved, std::io::ErrorKind::InvalidData))
    }
}

/// The path of the file `file_name` of the inbox folder, checked before any
/// action: a plain name, and still on disk (spec 09, "Identidade").
pub(crate) fn inbox_file(inbox: &Path, file_name: &[u8]) -> Result<PathBuf> {
    check_plain_file_name(file_name)?;
    let path = inbox.join(os_bytes::from_bytes(file_name));
    if std::fs::symlink_metadata(&path).is_err() {
        return Err(Error::InboxDocumentNotFound(path));
    }
    Ok(path)
}

/// The UI names a file of the inbox by its name: anything that could reach
/// outside the folder (`..`, a `/`) is refused before touching the disk.
fn check_plain_file_name(name: &[u8]) -> Result<()> {
    let reason = match name {
        [] => InvalidName::Empty,
        b"." | b".." => InvalidName::Reserved,
        _ if name.contains(&b'/') || name.contains(&0) => InvalidName::ContainsSlash,
        _ => return Ok(()),
    };
    Err(Error::InvalidName(reason))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::workspace::test_env::TestEnv;

    /// A workspace with the project `Faculdade`, and an inbox folder.
    struct Setup {
        _env: TestEnv,
        workspace: Workspace,
        inbox: std::path::PathBuf,
        dest: RelativePath,
    }

    fn setup() -> Setup {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        fs::create_dir(workspace.root().join("Faculdade")).unwrap();
        let inbox = env.dir("Downloads");
        let dest = RelativePath::from_bytes("Faculdade").unwrap();
        Setup {
            _env: env,
            workspace,
            inbox,
            dest,
        }
    }

    #[test]
    fn move_from_inbox_moves_file_into_workspace_folder() {
        let s = setup();
        fs::write(s.inbox.join("Edital.txt"), "edital").unwrap();

        s.workspace
            .move_from_inbox(&s.inbox, b"Edital.txt", &s.dest)
            .unwrap();

        let moved = s.workspace.root().join("Faculdade/Edital.txt");
        assert_eq!(fs::read_to_string(moved).unwrap(), "edital");
        assert!(!s.inbox.join("Edital.txt").exists());
    }

    #[test]
    fn move_from_inbox_indexes_document_with_new_id() {
        let s = setup();
        fs::write(s.inbox.join("Edital.txt"), "edital").unwrap();

        let doc = s
            .workspace
            .move_from_inbox(&s.inbox, b"Edital.txt", &s.dest)
            .unwrap();

        assert_eq!(s.workspace.document(doc.id).unwrap(), doc);
        assert_eq!(
            doc.path,
            RelativePath::from_bytes("Faculdade/Edital.txt").unwrap()
        );
    }

    #[test]
    fn move_from_inbox_adds_suffix_on_collision() {
        let s = setup();
        fs::write(s.workspace.root().join("Faculdade/Edital.txt"), "antigo").unwrap();
        fs::write(s.inbox.join("Edital.txt"), "novo").unwrap();

        let doc = s
            .workspace
            .move_from_inbox(&s.inbox, b"Edital.txt", &s.dest)
            .unwrap();

        assert_eq!(doc.name, "Edital (1).txt");
        let kept = s.workspace.root().join("Faculdade/Edital.txt");
        assert_eq!(fs::read_to_string(kept).unwrap(), "antigo");
    }

    #[test]
    fn move_from_inbox_fails_with_document_not_found_when_file_is_gone() {
        let s = setup();

        let error = s
            .workspace
            .move_from_inbox(&s.inbox, b"Sumiu.txt", &s.dest)
            .unwrap_err();

        let gone = s.inbox.join("Sumiu.txt");
        assert!(
            matches!(&error, Error::InboxDocumentNotFound(path) if *path == gone),
            "{error:?}"
        );
    }

    #[test]
    fn move_from_inbox_rejects_name_that_is_not_a_plain_file_name() {
        let s = setup();
        let outside = s.inbox.parent().unwrap().join("segredo.txt");
        fs::write(&outside, "fora da Entrada").unwrap();

        for name in [&b"../segredo.txt"[..], b"..", b".", b""] {
            let error = s
                .workspace
                .move_from_inbox(&s.inbox, name, &s.dest)
                .unwrap_err();

            assert!(
                matches!(error, Error::InvalidName(_)),
                "{name:?}: {error:?}"
            );
        }
        assert!(outside.exists());
    }

    #[test]
    fn move_from_inbox_records_last_destination() {
        let s = setup();
        fs::write(s.inbox.join("Edital.txt"), "edital").unwrap();

        s.workspace
            .move_from_inbox(&s.inbox, b"Edital.txt", &s.dest)
            .unwrap();

        assert_eq!(s.workspace.last_destination().unwrap(), Some(s.dest));
    }
}
