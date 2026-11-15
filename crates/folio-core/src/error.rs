use std::io;
use std::path::{Path, PathBuf};

/// Failures the core reports, kept distinct so the UI can explain the cause.
///
/// The messages are for logs only: the UI picks its own text per variant.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("an item with this name already exists: {0}")]
    AlreadyExists(PathBuf),
    #[error("filesystem cannot rename without overwriting: {0}")]
    NoAtomicSupport(PathBuf),
    #[error("system trash unavailable for {path}: {reason}")]
    TrashUnavailable { path: PathBuf, reason: String },
    #[error("copy does not match its source: {0}")]
    CopyMismatch(PathBuf),
    #[error("workspace is already open in another window")]
    WorkspaceInUse,
    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

impl Error {
    /// Wraps a system I/O error with the path it happened on.
    pub fn io(path: &Path, source: impl Into<io::Error>) -> Self {
        Self::Io {
            path: path.to_path_buf(),
            source: source.into(),
        }
    }

    /// True when a rename failed because source and destination are on
    /// different filesystems (`EXDEV`).
    pub fn crosses_devices(&self) -> bool {
        matches!(self, Self::Io { source, .. } if source.kind() == io::ErrorKind::CrossesDevices)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
