//! Exclusive hold on a workspace's `.folio/`, and temp cleanup on open
//! (guarantee 7).

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io;
use std::path::{Path, PathBuf};

use super::{Error, Result};

/// Held for as long as a window has the workspace open; dropping it releases
/// the lock.
pub struct WorkspaceGuard {
    _lock: File,
    tmp_dir: PathBuf,
}

impl WorkspaceGuard {
    /// Takes the exclusive lock on `folio_dir/lock`, then empties (or creates)
    /// `folio_dir/tmp`. The lock comes first so another window's temp files
    /// are never deleted.
    pub fn acquire(folio_dir: &Path) -> Result<Self> {
        let lock = lock_exclusive(&folio_dir.join("lock"))?;
        let tmp_dir = folio_dir.join("tmp");
        empty_dir(&tmp_dir)?;
        Ok(Self {
            _lock: lock,
            tmp_dir,
        })
    }

    /// Where new files are written before being published.
    pub fn tmp_dir(&self) -> &Path {
        &self.tmp_dir
    }
}

fn lock_exclusive(path: &Path) -> Result<File> {
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(path)
        .map_err(|e| Error::io(path, e))?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => Err(Error::WorkspaceInUse),
        Err(TryLockError::Error(e)) => Err(Error::io(path, e)),
    }
}

/// Leaves `dir` existing and empty. It holds only Folio's own temp files,
/// so deleting them is safe once the lock is held.
fn empty_dir(dir: &Path) -> Result<()> {
    match fs::remove_dir_all(dir) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(Error::io(dir, e)),
        _ => {}
    }
    fs::create_dir(dir).map_err(|e| Error::io(dir, e))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::integrity::Error;
    use crate::integrity::test_support::write_file;

    fn folio_dir() -> (tempfile::TempDir, PathBuf) {
        let root = tempfile::tempdir().unwrap();
        let folio = root.path().join(".folio");
        fs::create_dir(&folio).unwrap();
        (root, folio)
    }

    #[test]
    fn open_creates_lock_in_folio_dir() {
        let (_root, folio) = folio_dir();

        let _guard = WorkspaceGuard::acquire(&folio).unwrap();

        assert!(folio.join("lock").is_file());
    }

    #[test]
    fn open_fails_with_workspace_in_use_when_already_open() {
        let (_root, folio) = folio_dir();
        let _first = WorkspaceGuard::acquire(&folio).unwrap();

        let second = WorkspaceGuard::acquire(&folio);

        assert!(matches!(second, Err(Error::WorkspaceInUse)));
    }

    fn entries(dir: &Path) -> usize {
        fs::read_dir(dir).unwrap().count()
    }

    #[test]
    fn open_succeeds_again_after_owner_is_dropped() {
        let (_root, folio) = folio_dir();
        drop(WorkspaceGuard::acquire(&folio).unwrap());

        let reopened = WorkspaceGuard::acquire(&folio);

        assert!(matches!(reopened, Ok(guard) if guard.tmp_dir() == folio.join("tmp")));
    }

    #[test]
    fn open_empties_folio_tmp() {
        let (_root, folio) = folio_dir();
        let tmp = folio.join("tmp");
        fs::create_dir_all(tmp.join("pasta-de-imagens")).unwrap();
        write_file(&tmp, "conversao-interrompida", "metade");

        let guard = WorkspaceGuard::acquire(&folio).unwrap();

        assert_eq!(entries(guard.tmp_dir()), 0);
    }

    #[test]
    fn open_creates_folio_tmp_when_missing() {
        let (_root, folio) = folio_dir();

        let guard = WorkspaceGuard::acquire(&folio).unwrap();

        assert!(guard.tmp_dir().is_dir());
    }

    #[test]
    fn second_open_fails_without_deleting_first_temp_files() {
        let (_root, folio) = folio_dir();
        let first = WorkspaceGuard::acquire(&folio).unwrap();
        write_file(first.tmp_dir(), "conversao-em-andamento", "parcial");

        let second = WorkspaceGuard::acquire(&folio);

        assert!(matches!(second, Err(Error::WorkspaceInUse)));
        assert_eq!(entries(first.tmp_dir()), 1);
    }
}
