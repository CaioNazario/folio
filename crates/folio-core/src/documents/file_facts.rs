//! What the filesystem says about a document file (spec 04).

use std::fs::{self, Metadata};
use std::path::Path;

use crate::clock::system_time_millis;
use crate::indexing::fs_id::FsId;
use crate::{Error, Result};

/// Size and dates of a document, following symlinks to the real file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileFacts {
    pub size: u64,
    pub modified_at: i64,
    pub created_at: Option<i64>,
    /// Device and inode of the real file, for reconnection (spec 05).
    pub fs_id: FsId,
}

impl FileFacts {
    /// Reads the facts of the real file behind `path`. A broken symlink gets
    /// the link's own facts: it still appears, as an invalid document.
    pub fn read(path: &Path) -> Result<Self> {
        let metadata = fs::metadata(path)
            .or_else(|_| fs::symlink_metadata(path))
            .map_err(|e| Error::io(path, e))?;
        Ok(Self::from_metadata(&metadata))
    }

    fn from_metadata(metadata: &Metadata) -> Self {
        Self {
            size: metadata.len(),
            modified_at: metadata.modified().map_or(0, system_time_millis),
            // Birth time comes from statx; not every filesystem records it.
            created_at: metadata.created().ok().map(system_time_millis),
            fs_id: FsId::from_metadata(metadata),
        }
    }
}
