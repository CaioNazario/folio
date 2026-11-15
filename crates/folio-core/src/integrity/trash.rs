//! Deletion always goes to the system trash, never permanent (guarantee 4).

use std::path::Path;

use crate::{Error, Result};

/// A place deleted items go to, from which the user can restore them.
pub trait Trash {
    /// Moves `path` to the trash, or explains why it could not.
    fn send(&self, path: &Path) -> std::result::Result<(), String>;
}

/// Sends `path` to `trash`. If the trash fails, nothing is deleted: there is
/// no fallback to permanent deletion.
pub fn delete_to_trash(trash: &impl Trash, path: &Path) -> Result<()> {
    trash.send(path).map_err(|reason| Error::TrashUnavailable {
        path: path.to_path_buf(),
        reason,
    })
}

/// The operating system trash (freedesktop spec on Linux, including the
/// per-mount `$topdir/.Trash-$uid`).
pub struct SystemTrash;

impl Trash for SystemTrash {
    fn send(&self, path: &Path) -> std::result::Result<(), String> {
        ::trash::delete(path).map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::path::PathBuf;

    use super::*;
    use crate::integrity::test_support::write_file;

    #[derive(Default)]
    struct RecordingTrash {
        sent: RefCell<Vec<PathBuf>>,
    }

    impl Trash for RecordingTrash {
        fn send(&self, path: &Path) -> std::result::Result<(), String> {
            self.sent.borrow_mut().push(path.to_path_buf());
            Ok(())
        }
    }

    #[test]
    fn delete_sends_path_to_trash() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_file(dir.path(), "velho.txt", "conteúdo");
        let trash = RecordingTrash::default();

        delete_to_trash(&trash, &path).unwrap();

        assert_eq!(*trash.sent.borrow(), [path]);
    }

    struct BrokenTrash;

    impl Trash for BrokenTrash {
        fn send(&self, _path: &Path) -> std::result::Result<(), String> {
            Err("sem lixeira neste ponto de montagem".into())
        }
    }

    #[test]
    fn delete_fails_with_trash_unavailable_and_keeps_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_file(dir.path(), "velho.txt", "conteúdo");

        let result = delete_to_trash(&BrokenTrash, &path);

        assert!(matches!(result, Err(Error::TrashUnavailable { path: p, .. }) if p == path));
        assert!(path.exists());
    }
}
