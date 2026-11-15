//! Directory creation that never reuses an existing name (guarantee 1).

use std::fs;
use std::io;
use std::path::Path;

use crate::{Error, Result};

/// Creates the directory `path`, failing if anything already has that name.
pub fn create_dir_no_replace(path: &Path) -> Result<()> {
    fs::create_dir(path).map_err(|source| match source.kind() {
        io::ErrorKind::AlreadyExists => Error::AlreadyExists(path.to_path_buf()),
        _ => Error::io(path, source),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrity::test_support::{snapshot, write_file};

    #[test]
    fn create_dir_fails_with_already_exists_when_name_taken() {
        let dir = tempfile::tempdir().unwrap();
        let taken = write_file(dir.path(), "Faculdade", "arquivo");
        let before = snapshot(dir.path());

        let result = create_dir_no_replace(&taken);

        assert!(matches!(result, Err(Error::AlreadyExists(path)) if path == taken));
        assert_eq!(snapshot(dir.path()), before);
    }
}
