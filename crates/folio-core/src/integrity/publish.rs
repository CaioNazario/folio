//! New files are born in a temp file and published without overwriting
//! (guarantee 2).

use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use tempfile::{Builder, NamedTempFile};

use super::move_with_suffix::place_with_suffix;
use super::{Error, Result, rename_no_replace};

/// Writes a new file through a temp in `tmp_dir`, then publishes it as
/// `dest_dir/name`, adding ` (n)` on collision. Returns the final path.
///
/// Any error before publishing drops the temp file, so `dest_dir` never
/// receives a partial file.
pub fn publish_new_file(
    tmp_dir: &Path,
    dest_dir: &Path,
    name: &OsStr,
    write: impl FnOnce(&mut File) -> io::Result<()>,
) -> Result<PathBuf> {
    let temp = NamedTempFile::new_in(tmp_dir).map_err(|e| Error::io(tmp_dir, e))?;
    let temp = fill_temp(temp, write)?;
    match publish_temp(&temp, dest_dir, name) {
        Err(error) if error.crosses_devices() => publish_copy(temp.path(), dest_dir, name),
        published => keep(temp, published?),
    }
}

/// Copies `source` into a hidden `.folio-tmp-*` file inside `dest_dir`, then
/// publishes it there as `name` (with suffix on collision).
///
/// Used when the destination is on another filesystem, where renaming from
/// `.folio/tmp/` fails with `EXDEV`.
pub(crate) fn publish_copy(source: &Path, dest_dir: &Path, name: &OsStr) -> Result<PathBuf> {
    let temp = copy_to_hidden_temp(source, dest_dir)?;
    publish_and_keep(temp, dest_dir, name)
}

/// Copies `source` into a synced hidden temp file inside `dest_dir`.
pub(crate) fn copy_to_hidden_temp(source: &Path, dest_dir: &Path) -> Result<NamedTempFile> {
    let temp = Builder::new()
        .prefix(HIDDEN_TEMP_PREFIX)
        .tempfile_in(dest_dir)
        .map_err(|e| Error::io(dest_dir, e))?;
    fill_temp(temp, |file| {
        io::copy(&mut File::open(source)?, file).map(drop)
    })
}

/// Publishes `temp` as `dest_dir/name` (with suffix on collision).
pub(crate) fn publish_and_keep(
    temp: NamedTempFile,
    dest_dir: &Path,
    name: &OsStr,
) -> Result<PathBuf> {
    let published = publish_temp(&temp, dest_dir, name)?;
    keep(temp, published)
}

/// Prefix of the temp files Folio creates outside `.folio/`; the scan
/// removes leftovers (spec 05).
pub const HIDDEN_TEMP_PREFIX: &str = ".folio-tmp-";

fn fill_temp(
    mut temp: NamedTempFile,
    write: impl FnOnce(&mut File) -> io::Result<()>,
) -> Result<NamedTempFile> {
    write(temp.as_file_mut()).map_err(|e| Error::io(temp.path(), e))?;
    temp.as_file()
        .sync_all()
        .map_err(|e| Error::io(temp.path(), e))?;
    Ok(temp)
}

fn publish_temp(temp: &NamedTempFile, dest_dir: &Path, name: &OsStr) -> Result<PathBuf> {
    place_with_suffix(dest_dir, name, |to| rename_no_replace(temp.path(), to))
}

/// The file now lives at `to`: stop the guard from deleting its old name.
fn keep(temp: NamedTempFile, to: PathBuf) -> Result<PathBuf> {
    temp.into_temp_path()
        .keep()
        .map_err(|e| Error::io(&to, e.error))?;
    Ok(to)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;

    use super::*;
    use crate::integrity::test_support::write_file;

    struct Dirs {
        _root: tempfile::TempDir,
        tmp: PathBuf,
        dest: PathBuf,
    }

    fn dirs() -> Dirs {
        let root = tempfile::tempdir().unwrap();
        let tmp = root.path().join(".folio/tmp");
        let dest = root.path().join("Projeto");
        fs::create_dir_all(&tmp).unwrap();
        fs::create_dir(&dest).unwrap();
        Dirs {
            _root: root,
            tmp,
            dest,
        }
    }

    fn publish(dirs: &Dirs, content: &'static str) -> Result<PathBuf> {
        publish_new_file(&dirs.tmp, &dirs.dest, OsStr::new("Trabalho.md"), |file| {
            file.write_all(content.as_bytes())
        })
    }

    #[test]
    fn publish_writes_content_to_destination() {
        let dirs = dirs();

        let published = publish(&dirs, "# Título").unwrap();

        assert_eq!(published, dirs.dest.join("Trabalho.md"));
        assert_eq!(fs::read_to_string(&published).unwrap(), "# Título");
    }

    fn is_empty(dir: &Path) -> bool {
        fs::read_dir(dir).unwrap().next().is_none()
    }

    #[test]
    fn publish_leaves_no_temp_file_in_folio_tmp() {
        let dirs = dirs();

        publish(&dirs, "conteúdo").unwrap();

        assert!(is_empty(&dirs.tmp));
    }

    #[test]
    fn publish_adds_suffix_and_keeps_existing_file() {
        let dirs = dirs();
        write_file(&dirs.dest, "Trabalho.md", "antigo");

        let published = publish(&dirs, "novo").unwrap();

        assert_eq!(published, dirs.dest.join("Trabalho (1).md"));
        assert_eq!(fs::read_to_string(&published).unwrap(), "novo");
        let existing = fs::read_to_string(dirs.dest.join("Trabalho.md")).unwrap();
        assert_eq!(existing, "antigo");
    }

    #[test]
    fn publish_discards_temp_file_when_write_fails() {
        let dirs = dirs();

        let result = publish_new_file(&dirs.tmp, &dirs.dest, OsStr::new("Trabalho.md"), |_| {
            Err(io::Error::other("escritor falhou"))
        });

        assert!(matches!(result, Err(Error::Io { .. })));
        assert!(is_empty(&dirs.tmp));
        assert!(is_empty(&dirs.dest));
    }

    #[test]
    fn publish_via_destination_dir_leaves_no_hidden_temp_file() {
        let dirs = dirs();
        let source = write_file(&dirs.tmp, "gerado", "conteúdo");

        let published = publish_copy(&source, &dirs.dest, OsStr::new("Trabalho.md")).unwrap();

        assert_eq!(fs::read_to_string(&published).unwrap(), "conteúdo");
        let names: Vec<_> = fs::read_dir(&dirs.dest)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(names, ["Trabalho.md"]);
    }
}
