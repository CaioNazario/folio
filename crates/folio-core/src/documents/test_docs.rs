//! Helpers to write document files in workspace tests.

use std::fs;
use std::time::{Duration, SystemTime};

use crate::paths::RelativePath;
use crate::workspace::Workspace;

/// 2023-11-14T22:13:20Z, in ms.
pub const MTIME: i64 = 1_700_000_000_000;

/// Writes `content` at `path` inside the workspace with a known mtime.
pub fn write_doc(workspace: &Workspace, path: &str, content: &[u8]) -> RelativePath {
    let full = workspace.root().join(path);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(&full, content).unwrap();
    let mtime = SystemTime::UNIX_EPOCH + Duration::from_millis(MTIME as u64);
    fs::File::options()
        .write(true)
        .open(&full)
        .unwrap()
        .set_modified(mtime)
        .unwrap();
    RelativePath::from_bytes(path).unwrap()
}
