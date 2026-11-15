//! What the disk has directly in a directory (spec 05, step 1).

use std::fs;
use std::path::Path;

use crate::paths::RelativePath;
use crate::workspace::ops::entry_path;
use crate::{Error, Result};

/// Paths of the entries directly in `dir` that are not directories: regular
/// files and symlinks to files (symlinks to directories are never followed).
pub fn file_paths(root: &Path, dir: Option<&RelativePath>) -> Result<Vec<RelativePath>> {
    let full = dir.map_or_else(|| root.to_path_buf(), |dir| dir.resolve(root));
    let mut paths = Vec::new();
    for entry in fs::read_dir(&full).map_err(|e| Error::io(&full, e))? {
        let entry = entry.map_err(|e| Error::io(&full, e))?;
        if !entry.path().is_dir() {
            paths.push(entry_path(dir, &entry.file_name())?);
        }
    }
    Ok(paths)
}
