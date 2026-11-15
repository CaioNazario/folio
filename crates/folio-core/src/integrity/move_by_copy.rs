//! Move across filesystems, where rename does not work (guarantee 3).

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use super::publish::{copy_to_hidden_temp, publish_and_keep};
use super::{Error, Result};

/// Moves `from` into `dest_dir` by copying: copy into a hidden temp next to
/// the destination, check the copy's hash, publish without overwriting
/// (suffix on collision), and only then remove the source.
///
/// On any failure the source stays intact and the temp file is dropped.
pub fn move_by_copy(from: &Path, dest_dir: &Path) -> Result<PathBuf> {
    let Some(name) = from.file_name() else {
        return Err(Error::io(from, io::ErrorKind::InvalidInput));
    };
    let temp = copy_to_hidden_temp(from, dest_dir)?;
    verify_copy(from, temp.path())?;
    let moved = publish_and_keep(temp, dest_dir, name)?;
    fs::remove_file(from).map_err(|e| Error::io(from, e))?;
    Ok(moved)
}

/// Fails with [`Error::CopyMismatch`] unless `copy` has the same BLAKE3 hash
/// as `source`.
fn verify_copy(source: &Path, copy: &Path) -> Result<()> {
    if hash_file(source)? == hash_file(copy)? {
        return Ok(());
    }
    Err(Error::CopyMismatch(source.to_path_buf()))
}

fn hash_file(path: &Path) -> Result<blake3::Hash> {
    let file = File::open(path).map_err(|e| Error::io(path, e))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update_reader(file).map_err(|e| Error::io(path, e))?;
    Ok(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrity::test_support::write_file;

    fn setup() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let from = write_file(dir.path(), "Trabalho.pdf", "novo");
        let dest = dir.path().join("Projeto");
        fs::create_dir(&dest).unwrap();
        (dir, from, dest)
    }

    #[test]
    fn move_by_copy_publishes_copy_and_removes_source() {
        let (_dir, from, dest) = setup();

        let moved = move_by_copy(&from, &dest).unwrap();

        assert_eq!(moved, dest.join("Trabalho.pdf"));
        assert_eq!(fs::read_to_string(&moved).unwrap(), "novo");
        assert!(!from.exists());
    }

    #[test]
    fn move_by_copy_adds_suffix_when_destination_taken() {
        let (_dir, from, dest) = setup();
        write_file(&dest, "Trabalho.pdf", "antigo");

        let moved = move_by_copy(&from, &dest).unwrap();

        assert_eq!(moved, dest.join("Trabalho (1).pdf"));
        assert_eq!(
            fs::read_to_string(dest.join("Trabalho.pdf")).unwrap(),
            "antigo"
        );
    }

    #[test]
    fn move_by_copy_keeps_source_when_destination_is_unavailable() {
        let (dir, from, _dest) = setup();

        let result = move_by_copy(&from, &dir.path().join("inexistente"));

        assert!(matches!(result, Err(Error::Io { .. })));
        assert_eq!(fs::read_to_string(&from).unwrap(), "novo");
    }

    #[test]
    fn verify_copy_fails_with_copy_mismatch_when_content_differs() {
        let (dir, from, _dest) = setup();
        let copy = write_file(dir.path(), "copia.pdf", "truncado");

        let result = verify_copy(&from, &copy);

        assert!(matches!(result, Err(Error::CopyMismatch(path)) if path == from));
    }
}
