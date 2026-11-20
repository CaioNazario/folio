//! DOCX protected by password: the Office encrypted container (spec 04),
//! decrypted in memory with `office-crypto`. Neither the password nor the
//! decrypted package ever reaches the disk.

use std::borrow::Cow;
use std::io::Cursor;
use std::panic;

use crate::readers::ReadError;

/// The ZIP package of a DOCX: `bytes` themselves, or decrypted with
/// `password` when protected. A password is ignored for an unprotected
/// DOCX.
pub fn package_bytes<'a>(
    bytes: &'a [u8],
    password: Option<&str>,
) -> Result<Cow<'a, [u8]>, ReadError> {
    if !is_encrypted(bytes) {
        return Ok(Cow::Borrowed(bytes));
    }
    let password = password.ok_or(ReadError::PasswordRequired)?;
    decrypt(bytes, password).map(Cow::Owned)
}

/// The package inside the encrypted container `bytes`.
fn decrypt(bytes: &[u8], password: &str) -> Result<Vec<u8>, ReadError> {
    let decrypted = office_decrypt(bytes, password)?;
    // `office-crypto` does not check the password: a wrong one decrypts
    // into noise instead of a ZIP.
    if !is_zip(&decrypted) {
        return Err(ReadError::WrongPassword);
    }
    Ok(decrypted)
}

// With `panic = "abort"` the `catch_unwind` below cannot contain the panic,
// and a broken file would close the app.
#[cfg(panic = "abort")]
compile_error!(
    "folio-core needs `panic = \"unwind\"`: a malformed encrypted DOCX makes \
     office-crypto panic, and the reader contains that panic with catch_unwind"
);

/// `office-crypto` panics on some malformed containers instead of
/// failing: a panic there means a broken file, not a crash of the app.
fn office_decrypt(bytes: &[u8], password: &str) -> Result<Vec<u8>, ReadError> {
    let raw = bytes.to_vec();
    let outcome = panic::catch_unwind(|| office_crypto::decrypt_from_bytes(raw, password));
    match outcome {
        Ok(decrypted) => decrypted.map_err(|error| ReadError::Failed(error.to_string())),
        Err(_) => Err(ReadError::Failed("malformed encrypted container".into())),
    }
}

fn is_zip(bytes: &[u8]) -> bool {
    zip::ZipArchive::new(Cursor::new(bytes)).is_ok()
}

/// Whether `bytes` are the Office encrypted container: an OLE compound
/// file with the `EncryptionInfo` and `EncryptedPackage` streams.
fn is_encrypted(bytes: &[u8]) -> bool {
    cfb::CompoundFile::open(Cursor::new(bytes)).is_ok_and(|compound| {
        compound.is_stream("EncryptionInfo") && compound.is_stream("EncryptedPackage")
    })
}

#[cfg(test)]
mod tests {
    use super::super::read_docx;
    use super::super::test_docx::docx;
    use crate::model::{Block, Inline};
    use crate::readers::ReadError;

    /// Saved by LibreOffice with the password `folio`.
    const PROTECTED: &[u8] = include_bytes!("../../../tests/fixtures/docx/protegido.docx");

    #[test]
    fn read_docx_with_right_password_reads_protected_docx() {
        let model = read_docx(PROTECTED, Some("folio")).unwrap();

        let heading = Block::Heading {
            level: 1,
            content: vec![Inline::Text("Contrato".to_string())],
        };
        let clause = Block::Paragraph(vec![Inline::Text("Cláusula confidencial.".to_string())]);
        assert_eq!(model.blocks, [heading, clause]);
    }

    #[test]
    fn read_docx_of_protected_docx_without_password_fails_with_password_required() {
        assert_eq!(read_docx(PROTECTED, None), Err(ReadError::PasswordRequired));
    }

    #[test]
    fn read_docx_of_protected_docx_with_wrong_password_fails_with_wrong_password() {
        assert_eq!(
            read_docx(PROTECTED, Some("errada")),
            Err(ReadError::WrongPassword)
        );
    }

    #[test]
    fn read_docx_ignores_password_of_unprotected_docx() {
        let bytes = docx("<w:p><w:r><w:t>Aberto</w:t></w:r></w:p>");

        let model = read_docx(&bytes, Some("qualquer")).unwrap();

        assert_eq!(model, read_docx(&bytes, None).unwrap());
    }

    /// The encrypted container with empty streams, as no Office app saves.
    fn broken_container() -> Vec<u8> {
        let mut compound = cfb::CompoundFile::create(std::io::Cursor::new(Vec::new())).unwrap();
        for name in ["EncryptionInfo", "EncryptedPackage"] {
            compound.create_stream(name).unwrap();
        }
        compound.flush().unwrap();
        compound.into_inner().into_inner()
    }

    #[test]
    fn read_docx_of_broken_encrypted_container_fails_with_read_failed() {
        let result = read_docx(&broken_container(), Some("folio"));

        assert!(matches!(result, Err(ReadError::Failed(_))));
    }
}
