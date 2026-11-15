//! Content of the open workspace's documents, by id, for the Reader.

use std::fs::File;
use std::io;
use std::path::Path;

use uuid::Uuid;

use super::{DISPLAY_TEXT_LIMIT, DisplayText, display_text};
use crate::workspace::Workspace;
use crate::{Error, Result};

impl Workspace {
    /// The text of the TXT document `id`, cut past [`DISPLAY_TEXT_LIMIT`].
    pub fn text_content(&self, id: Uuid) -> Result<DisplayText> {
        let doc = self.document(id)?;
        if !doc.valid {
            return Err(Error::InvalidDocument(id));
        }
        let full = doc.path.resolve(self.root());
        let file = File::open(&full).map_err(|e| open_error(id, &full, e))?;
        display_text(file, DISPLAY_TEXT_LIMIT).map_err(|e| Error::io(&full, e))
    }
}

/// A file gone from disk is a document not found: the stage says so and
/// the tree gets reconciled (spec 07, "Estados do palco").
fn open_error(id: Uuid, path: &Path, error: io::Error) -> Error {
    if error.kind() == io::ErrorKind::NotFound {
        return Error::DocumentNotFound(id);
    }
    Error::io(path, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::test_docs::write_doc;
    use crate::workspace::test_env::TestEnv;

    fn indexed(workspace: &Workspace, path: &str, content: &[u8]) -> Uuid {
        let path = write_doc(workspace, path, content);
        workspace.index_file(&path).unwrap().unwrap().id
    }

    #[test]
    fn text_content_of_invalid_document_fails_with_invalid_document() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let binary = indexed(&workspace, "Binario.txt", b"\0\0\0dados");

        let result = workspace.text_content(binary);

        assert!(matches!(result, Err(Error::InvalidDocument(id)) if id == binary));
    }

    #[test]
    fn text_content_of_file_gone_from_disk_fails_with_document_not_found() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let gone = indexed(&workspace, "Nota.txt", b"texto");
        std::fs::remove_file(workspace.root().join("Nota.txt")).unwrap();

        let result = workspace.text_content(gone);

        assert!(matches!(result, Err(Error::DocumentNotFound(id)) if id == gone));
    }
}
