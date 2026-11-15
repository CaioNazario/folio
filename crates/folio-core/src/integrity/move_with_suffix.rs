//! Move into a directory, adding ` (n)` on name collision (guarantee 1).

use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

use super::{Error, Result, move_by_copy, rename_no_replace, with_suffix};

/// Moves `from` into `dest_dir` and returns the final path.
///
/// A file on another filesystem falls back to [`move_by_copy`].
///
/// Keeps the name when free; otherwise tries `(1)`, `(2)`... in a loop, so a
/// name taken by another process between attempts just leads to the next one.
pub fn move_with_suffix(from: &Path, dest_dir: &Path) -> Result<PathBuf> {
    let Some(name) = from.file_name() else {
        return Err(Error::io(from, io::ErrorKind::InvalidInput));
    };
    match place_with_suffix(dest_dir, name, |to| rename_no_replace(from, to)) {
        Err(error) if error.crosses_devices() && from.is_file() => move_by_copy(from, dest_dir),
        moved => moved,
    }
}

/// Calls `place` with `dest_dir/name`, then `name (1)`, `name (2)`... until it
/// succeeds or fails with anything other than [`Error::AlreadyExists`].
pub(crate) fn place_with_suffix(
    dest_dir: &Path,
    name: &OsStr,
    mut place: impl FnMut(&Path) -> Result<()>,
) -> Result<PathBuf> {
    let mut to = dest_dir.join(name);
    let mut n = 0;
    loop {
        match place(&to) {
            Err(Error::AlreadyExists(_)) => n += 1,
            other => return other.map(|()| to),
        }
        to = dest_dir.join(with_suffix(name, n));
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::integrity::test_support::write_file;

    fn setup() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let from = write_file(dir.path(), "Trabalho.docx", "novo");
        let dest = dir.path().join("destino");
        fs::create_dir(&dest).unwrap();
        (dir, from, dest)
    }

    #[test]
    fn move_with_suffix_keeps_name_when_free() {
        let (_dir, from, dest) = setup();

        let moved = move_with_suffix(&from, &dest).unwrap();

        assert_eq!(moved, dest.join("Trabalho.docx"));
        assert_eq!(fs::read_to_string(&moved).unwrap(), "novo");
    }

    #[test]
    fn move_with_suffix_uses_1_when_name_taken() {
        let (_dir, from, dest) = setup();
        write_file(&dest, "Trabalho.docx", "antigo");

        let moved = move_with_suffix(&from, &dest).unwrap();

        assert_eq!(moved, dest.join("Trabalho (1).docx"));
        assert_eq!(fs::read_to_string(&moved).unwrap(), "novo");
        assert_eq!(
            fs::read_to_string(dest.join("Trabalho.docx")).unwrap(),
            "antigo"
        );
    }

    #[test]
    fn move_with_suffix_skips_to_next_free_number() {
        let (_dir, from, dest) = setup();
        for name in ["Trabalho.docx", "Trabalho (1).docx", "Trabalho (2).docx"] {
            write_file(&dest, name, "antigo");
        }

        let moved = move_with_suffix(&from, &dest).unwrap();

        assert_eq!(moved, dest.join("Trabalho (3).docx"));
        assert_eq!(fs::read_to_string(&moved).unwrap(), "novo");
    }
}
