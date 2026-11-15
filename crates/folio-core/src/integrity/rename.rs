//! Rename that never overwrites (guarantee 1).

use std::path::Path;

use rustix::fs::{CWD, RenameFlags, renameat_with};
use rustix::io::Errno;

use crate::{Error, Result};

/// Renames `from` to `to`, failing atomically if `to` already exists.
pub fn rename_no_replace(from: &Path, to: &Path) -> Result<()> {
    renameat_with(CWD, from, CWD, to, RenameFlags::NOREPLACE)
        .map_err(|errno| rename_error(errno, from, to))
}

/// Errnos a filesystem returns when it lacks `RENAME_NOREPLACE` (some FUSE
/// and NFS mounts). Never fall back to a plain rename, which would overwrite.
const UNSUPPORTED: [Errno; 3] = [Errno::INVAL, Errno::NOSYS, Errno::OPNOTSUPP];

fn rename_error(errno: Errno, from: &Path, to: &Path) -> Error {
    if errno == Errno::EXIST {
        return Error::AlreadyExists(to.to_path_buf());
    }
    if UNSUPPORTED.contains(&errno) {
        return Error::NoAtomicSupport(to.to_path_buf());
    }
    Error::io(from, errno)
}

#[cfg(test)]
mod tests {
    use std::{fs, io};

    use super::*;
    use crate::integrity::test_support::{snapshot, write_file};

    #[test]
    fn rename_moves_file_to_free_destination() {
        let dir = tempfile::tempdir().unwrap();
        let from = write_file(dir.path(), "a.txt", "conteúdo");
        let to = dir.path().join("b.txt");

        rename_no_replace(&from, &to).unwrap();

        assert!(!from.exists());
        assert_eq!(fs::read_to_string(&to).unwrap(), "conteúdo");
    }

    #[test]
    fn rename_fails_with_already_exists_and_changes_nothing_when_destination_taken() {
        let dir = tempfile::tempdir().unwrap();
        let from = write_file(dir.path(), "a.txt", "origem");
        let to = write_file(dir.path(), "b.txt", "destino");
        let before = snapshot(dir.path());

        let result = rename_no_replace(&from, &to);

        assert!(matches!(result, Err(Error::AlreadyExists(path)) if path == to));
        assert_eq!(snapshot(dir.path()), before);
    }

    #[test]
    fn rename_fails_with_already_exists_when_destination_is_directory() {
        let dir = tempfile::tempdir().unwrap();
        let from = write_file(dir.path(), "a.txt", "origem");
        let to = dir.path().join("pasta");
        fs::create_dir(&to).unwrap();

        let result = rename_no_replace(&from, &to);

        assert!(matches!(result, Err(Error::AlreadyExists(path)) if path == to));
        assert!(from.exists());
    }

    #[test]
    fn rename_maps_unsupported_errno_to_no_atomic_support() {
        let (from, to) = (Path::new("/w/a.txt"), Path::new("/w/b.txt"));

        let error = rename_error(Errno::INVAL, from, to);

        assert!(matches!(error, Error::NoAtomicSupport(path) if path == to));
    }

    #[test]
    fn rename_returns_io_error_with_path_when_source_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let from = dir.path().join("sumiu.txt");

        let result = rename_no_replace(&from, &dir.path().join("b.txt"));

        assert!(matches!(
            result,
            Err(Error::Io { path, source }) if path == from && source.kind() == io::ErrorKind::NotFound
        ));
    }
}
