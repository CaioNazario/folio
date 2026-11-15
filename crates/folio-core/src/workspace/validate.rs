//! Checks a directory must pass before it can be opened as a workspace.

use std::fs;
use std::path::{Path, PathBuf};

use rustix::fs::{Access, access};

use crate::{Error, Result};

/// The directory that marks a workspace.
pub const FOLIO_DIR: &str = ".folio";

/// Fails unless `root` can be opened as a workspace. `home` is refused like
/// `/`: scanning it would index the whole user directory.
pub fn validate_root(root: &Path, home: &Path) -> Result<()> {
    if !root.is_dir() {
        return Err(Error::WorkspaceNotFound(root.to_path_buf()));
    }
    // Before the permission check: `/` is never writable, but the reason to
    // refuse it is that scanning it would index the whole system.
    if root.parent().is_none() || same_dir(root, home) {
        return Err(Error::ForbiddenRoot(root.to_path_buf()));
    }
    if access(root, Access::READ_OK | Access::WRITE_OK | Access::EXEC_OK).is_err() {
        return Err(Error::PermissionDenied(root.to_path_buf()));
    }
    if let Some(ancestor) = enclosing_workspace(root) {
        return Err(Error::NestedWorkspace { ancestor });
    }
    Ok(())
}

/// The nearest ancestor of `root` that is itself a workspace.
fn enclosing_workspace(root: &Path) -> Option<PathBuf> {
    root.ancestors()
        .skip(1)
        .find(|ancestor| ancestor.join(FOLIO_DIR).is_dir())
        .map(Path::to_path_buf)
}

/// Compares canonical forms, so `~/x/..` or a symlink to the home still match.
fn same_dir(a: &Path, b: &Path) -> bool {
    let canonical = |path: &Path| fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path));
    canonical(a) == canonical(b)
}
