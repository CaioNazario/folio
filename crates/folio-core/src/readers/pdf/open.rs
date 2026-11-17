//! Opening a PDF, with the reader errors for what can go wrong.

use hayro_interpret::hayro_syntax::{DecryptionError, LoadPdfError, Pdf};

use super::super::ReadError;

const NEEDS_PASSWORD: LoadPdfError = LoadPdfError::Decryption(DecryptionError::PasswordProtected);

/// The PDF in `bytes`. The empty password comes first: it opens a PDF
/// with no user password (only an owner one), and then `password` is
/// ignored.
pub fn open(bytes: &[u8], password: Option<&str>) -> Result<Pdf, ReadError> {
    match (load(bytes, ""), password) {
        (Err(NEEDS_PASSWORD), Some(password)) => {
            load(bytes, password).map_err(|e| read_error(e, ReadError::WrongPassword))
        }
        (result, _) => result.map_err(|e| read_error(e, ReadError::PasswordRequired)),
    }
}

fn load(bytes: &[u8], password: &str) -> Result<Pdf, LoadPdfError> {
    Pdf::new_with_password(bytes.to_vec(), password)
}

/// `error` as a reader error, `refused` when the password did not open
/// the PDF.
fn read_error(error: LoadPdfError, refused: ReadError) -> ReadError {
    match error {
        NEEDS_PASSWORD => refused,
        other => ReadError::Failed(format!("{other:?}")),
    }
}
