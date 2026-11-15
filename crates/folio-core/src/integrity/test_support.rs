//! Test helpers shared by the integrity tests.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Hash of every file under `root`, keyed by path relative to `root`.
///
/// Comparing two snapshots proves nothing beyond the expected change happened.
pub fn snapshot(root: &Path) -> BTreeMap<PathBuf, blake3::Hash> {
    let mut hashes = BTreeMap::new();
    collect(root, root, &mut hashes);
    hashes
}

fn collect(root: &Path, dir: &Path, hashes: &mut BTreeMap<PathBuf, blake3::Hash>) {
    for entry in fs::read_dir(dir).expect("read_dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect(root, &path, hashes);
            continue;
        }
        let relative = path.strip_prefix(root).expect("inside root").to_path_buf();
        hashes.insert(relative, blake3::hash(&fs::read(&path).expect("read")));
    }
}

/// Writes `content` to `dir/name` and returns the full path.
pub fn write_file(dir: &Path, name: &str, content: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, content).expect("write test file");
    path
}
