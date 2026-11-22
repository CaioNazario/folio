//! Why a reader could not build the document model (spec 07, "Erros").

/// A reader failure. The workspace turns it into an [`crate::Error`] with
/// the document id; the reason only goes to the log.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ReadError {
    /// Broken ZIP or XML, or a required part missing.
    #[error("cannot read the document: {0}")]
    Failed(String),
    /// The document is protected and no password was given.
    #[error("the document is protected by password")]
    PasswordRequired,
    /// The password given does not open the document.
    #[error("wrong password")]
    WrongPassword,
    /// Whoever asked for the document stopped the reading.
    #[error("reading cancelled")]
    Cancelled,
}
