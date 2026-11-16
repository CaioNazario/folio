//! Errors as they cross the bridge to the UI (Design spec 11, "O backend
//! não devolve texto"): one variant per error, tagged by `kind`, carrying
//! only the data the message needs. Technical details stay in the log.

use std::path::Path;

use serde::Serialize;
use ts_rs::TS;

use crate::Error;
use crate::paths::InvalidRelativePath;
use crate::workspace::InvalidName;

/// An [`Error`] as data the UI turns into a message in its own language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum BridgeError {
    AlreadyExists {
        path: String,
        name: String,
    },
    NoAtomicSupport {
        path: String,
    },
    /// The system trash refused `path`; the reason goes to the log.
    TrashUnavailable {
        path: String,
    },
    CopyMismatch {
        path: String,
    },
    NoWorkspaceOpen,
    WorkspaceInUse,
    WorkspaceNotFound {
        path: String,
    },
    ForbiddenRoot {
        path: String,
    },
    /// `path` is the workspace the chosen directory is inside of.
    NestedWorkspace {
        path: String,
    },
    PermissionDenied {
        path: String,
    },
    /// `path` is relative to the workspace root.
    RelativeSymlink {
        path: String,
    },
    DocumentNotFound {
        id: String,
    },
    InvalidDocument {
        id: String,
    },
    /// The reader could not build the document; the reason goes to the log.
    ReadFailed {
        id: String,
    },
    /// The document is protected; the UI asks for its password.
    PasswordRequired {
        id: String,
    },
    WrongPassword {
        id: String,
    },
    InvalidPosition,
    MoveIntoItself,
    InvalidName {
        reason: InvalidName,
    },
    InvalidPath {
        reason: InvalidRelativePath,
    },
    NewerDatabase {
        path: String,
    },
    /// A SQLite error on the database file at `path`; its detail goes to the
    /// log.
    Database {
        path: String,
    },
    /// An operating system error; its detail goes to the log.
    Io {
        path: String,
    },
}

impl From<&Error> for BridgeError {
    fn from(error: &Error) -> Self {
        path_only(error)
            .or_else(|| document_only(error))
            .unwrap_or_else(|| with_other_data(error))
    }
}

/// The errors that carry nothing but a path.
fn path_only(error: &Error) -> Option<BridgeError> {
    let bridged = match error {
        Error::NoAtomicSupport(path) => BridgeError::NoAtomicSupport { path: text(path) },
        Error::TrashUnavailable { path, .. } => BridgeError::TrashUnavailable { path: text(path) },
        Error::CopyMismatch(path) => BridgeError::CopyMismatch { path: text(path) },
        Error::WorkspaceNotFound(path) => BridgeError::WorkspaceNotFound { path: text(path) },
        Error::ForbiddenRoot(path) => BridgeError::ForbiddenRoot { path: text(path) },
        Error::NestedWorkspace { ancestor } => BridgeError::NestedWorkspace {
            path: text(ancestor),
        },
        Error::PermissionDenied(path) => BridgeError::PermissionDenied { path: text(path) },
        Error::NewerDatabase(path) => BridgeError::NewerDatabase { path: text(path) },
        Error::RelativeSymlink(path) => BridgeError::RelativeSymlink {
            path: path.display(),
        },
        Error::Database { path, .. } => BridgeError::Database { path: text(path) },
        Error::Io { path, .. } => BridgeError::Io { path: text(path) },
        _ => return None,
    };
    Some(bridged)
}

/// The errors that carry nothing but a document id.
fn document_only(error: &Error) -> Option<BridgeError> {
    let bridged = match error {
        Error::DocumentNotFound(id) => BridgeError::DocumentNotFound { id: id.to_string() },
        Error::InvalidDocument(id) => BridgeError::InvalidDocument { id: id.to_string() },
        Error::ReadFailed(id) => BridgeError::ReadFailed { id: id.to_string() },
        Error::PasswordRequired(id) => BridgeError::PasswordRequired { id: id.to_string() },
        Error::WrongPassword(id) => BridgeError::WrongPassword { id: id.to_string() },
        _ => return None,
    };
    Some(bridged)
}

fn with_other_data(error: &Error) -> BridgeError {
    match error {
        Error::AlreadyExists(path) => BridgeError::AlreadyExists {
            path: text(path),
            name: file_name(path),
        },
        Error::InvalidName(reason) => BridgeError::InvalidName {
            reason: reason.clone(),
        },
        Error::InvalidPath(reason) => BridgeError::InvalidPath {
            reason: reason.clone(),
        },
        Error::NoWorkspaceOpen => BridgeError::NoWorkspaceOpen,
        Error::WorkspaceInUse => BridgeError::WorkspaceInUse,
        Error::InvalidPosition => BridgeError::InvalidPosition,
        Error::MoveIntoItself => BridgeError::MoveIntoItself,
        _ => unreachable!("other errors are bridged by `path_only` or `document_only`"),
    }
}

fn text(path: &Path) -> String {
    path.display().to_string()
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::{Value, json};

    use super::*;

    fn bridged(error: Error) -> Value {
        serde_json::to_value(BridgeError::from(&error)).unwrap()
    }

    #[test]
    fn bridge_error_serializes_with_kind_tag_and_named_fields() {
        let error = Error::AlreadyExists(PathBuf::from("/home/ana/Folio/Trabalho.docx"));

        assert_eq!(
            bridged(error),
            json!({
                "kind": "AlreadyExists",
                "path": "/home/ana/Folio/Trabalho.docx",
                "name": "Trabalho.docx",
            })
        );
    }

    #[test]
    fn bridge_error_of_unit_variant_has_only_kind() {
        let bridged = [
            Error::WorkspaceInUse,
            Error::MoveIntoItself,
            Error::InvalidPosition,
        ]
        .map(bridged);

        assert_eq!(
            bridged,
            [
                json!({ "kind": "WorkspaceInUse" }),
                json!({ "kind": "MoveIntoItself" }),
                json!({ "kind": "InvalidPosition" }),
            ]
        );
    }

    #[test]
    fn bridge_error_of_io_error_keeps_only_path() {
        let source = std::io::Error::other("disk on fire");
        let error = Error::io(Path::new("/home/ana/Folio/Nota.txt"), source);

        assert_eq!(
            bridged(error),
            json!({ "kind": "Io", "path": "/home/ana/Folio/Nota.txt" })
        );
    }

    #[test]
    fn bridge_error_of_database_error_keeps_only_path() {
        let source = rusqlite::Error::InvalidQuery;
        let error = Error::database(Path::new("/home/ana/Folio/.folio/folio.db"), source);

        assert_eq!(
            bridged(error),
            json!({ "kind": "Database", "path": "/home/ana/Folio/.folio/folio.db" })
        );
    }

    #[test]
    fn bridge_error_of_trash_unavailable_drops_reason() {
        let error = Error::TrashUnavailable {
            path: PathBuf::from("/media/pendrive/Nota.txt"),
            reason: "no trash on this volume".to_string(),
        };

        assert_eq!(
            bridged(error),
            json!({ "kind": "TrashUnavailable", "path": "/media/pendrive/Nota.txt" })
        );
    }

    #[test]
    fn bridge_error_carries_document_id() {
        let id = uuid::Uuid::from_u128(7);

        let bridged = [Error::DocumentNotFound(id), Error::InvalidDocument(id)].map(bridged);

        let id = "00000000-0000-0000-0000-000000000007";
        assert_eq!(
            bridged,
            [
                json!({ "kind": "DocumentNotFound", "id": id }),
                json!({ "kind": "InvalidDocument", "id": id }),
            ]
        );
    }

    #[test]
    fn bridge_error_of_read_failed_carries_document_id() {
        let id = uuid::Uuid::from_u128(7);

        assert_eq!(
            bridged(Error::ReadFailed(id)),
            json!({ "kind": "ReadFailed", "id": "00000000-0000-0000-0000-000000000007" })
        );
    }

    #[test]
    fn bridge_error_of_password_errors_carries_document_id() {
        let id = uuid::Uuid::from_u128(7);

        let bridged = [Error::PasswordRequired(id), Error::WrongPassword(id)].map(bridged);

        let id = "00000000-0000-0000-0000-000000000007";
        assert_eq!(
            bridged,
            [
                json!({ "kind": "PasswordRequired", "id": id }),
                json!({ "kind": "WrongPassword", "id": id }),
            ]
        );
    }

    #[test]
    fn bridge_error_flattens_invalid_name_reason() {
        let error = Error::InvalidName(crate::workspace::InvalidName::Hidden);

        assert_eq!(
            bridged(error),
            json!({ "kind": "InvalidName", "reason": "Hidden" })
        );
    }

    #[test]
    fn bridge_error_flattens_invalid_path_reason() {
        let error = Error::InvalidPath(crate::paths::InvalidRelativePath::OutsideRoot);

        assert_eq!(
            bridged(error),
            json!({ "kind": "InvalidPath", "reason": "OutsideRoot" })
        );
    }

    /// Errors that carry a path, each with the `kind` it bridges to.
    fn path_errors() -> [(Error, &'static str); 8] {
        let path = || PathBuf::from("/home/ana/Folio");
        let relative = crate::paths::RelativePath::from_bytes("Folio/atalho").unwrap();
        [
            (Error::NoAtomicSupport(path()), "NoAtomicSupport"),
            (Error::CopyMismatch(path()), "CopyMismatch"),
            (Error::WorkspaceNotFound(path()), "WorkspaceNotFound"),
            (Error::ForbiddenRoot(path()), "ForbiddenRoot"),
            (
                Error::NestedWorkspace { ancestor: path() },
                "NestedWorkspace",
            ),
            (Error::PermissionDenied(path()), "PermissionDenied"),
            (Error::RelativeSymlink(relative), "RelativeSymlink"),
            (Error::NewerDatabase(path()), "NewerDatabase"),
        ]
    }

    #[test]
    fn bridge_error_of_other_path_errors_carries_path() {
        for (error, kind) in path_errors() {
            let path = match &error {
                Error::RelativeSymlink(_) => "Folio/atalho",
                _ => "/home/ana/Folio",
            };

            assert_eq!(bridged(error), json!({ "kind": kind, "path": path }));
        }
    }

    #[test]
    fn bridge_error_of_no_workspace_open_has_only_kind() {
        assert_eq!(
            bridged(Error::NoWorkspaceOpen),
            json!({ "kind": "NoWorkspaceOpen" })
        );
    }
}
