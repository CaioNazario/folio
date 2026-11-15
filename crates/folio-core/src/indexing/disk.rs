//! What the disk has directly in a directory (spec 05, step 1).

use std::fs;
use std::io;
use std::path::Path;

use crate::paths::RelativePath;
use crate::workspace::ops::entry_path;
use crate::{Error, Result};

/// Paths of the entries directly in `dir` that are not directories: regular
/// files and symlinks to files (symlinks to directories are never followed).
/// None when `dir` is gone from disk.
pub fn file_paths(root: &Path, dir: Option<&RelativePath>) -> Result<Vec<RelativePath>> {
    let full = dir.map_or_else(|| root.to_path_buf(), |dir| dir.resolve(root));
    let entries = match fs::read_dir(&full) {
        Ok(entries) => entries,
        // A directory gone from disk has nothing: its documents become missing.
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(Error::io(&full, e)),
    };
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| Error::io(&full, e))?;
        if !entry.path().is_dir() {
            paths.push(entry_path(dir, &entry.file_name())?);
        }
    }
    Ok(paths)
}
