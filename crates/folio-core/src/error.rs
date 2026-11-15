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
    #[error("no workspace is open")]
    NoWorkspaceOpen,
    #[error("workspace is already open in another window")]
    WorkspaceInUse,
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(PathBuf),
    #[error("{0} cannot be a workspace: it is the system root or the home")]
    ForbiddenRoot(PathBuf),
    #[error("directory is inside the workspace {ancestor}")]
    NestedWorkspace { ancestor: PathBuf },
    #[error("cannot read and write {0}")]
    PermissionDenied(PathBuf),
    #[error("a symlink with a relative target cannot change directory: {0}")]
    RelativeSymlink(crate::paths::RelativePath),
    #[error("document not found: {0}")]
    DocumentNotFound(uuid::Uuid),
    #[error("document does not match its format: {0}")]
    InvalidDocument(uuid::Uuid),
    #[error("reading position does not fit the document format")]
    InvalidPosition,
    #[error("cannot move a directory into itself or a descendant")]
    MoveIntoItself,
    #[error("invalid name: {0}")]
    InvalidName(#[from] crate::workspace::InvalidName),
    #[error("invalid workspace path: {0}")]
    InvalidPath(#[from] crate::paths::InvalidRelativePath),
    #[error("database was opened by a newer Folio: {0}")]
    NewerDatabase(PathBuf),
    #[error("database error on {path}: {source}")]
    Database {
        path: PathBuf,
        #[source]
        source: rusqlite::Error,
    },
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

    /// Wraps a SQLite error with the database file it happened on.
    pub fn database(path: &Path, source: rusqlite::Error) -> Self {
        Self::Database {
            path: path.to_path_buf(),
            source,
        }
    }

    /// True when a rename failed because source and destination are on
    /// different filesystems (`EXDEV`).
    pub fn crosses_devices(&self) -> bool {
        matches!(self, Self::Io { source, .. } if source.kind() == io::ErrorKind::CrossesDevices)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
