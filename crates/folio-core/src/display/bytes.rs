//! The bytes of PDF and DOCX documents for the Reader (spec 07,
//! "Exibição"): pdf.js and docx-preview render them in the webview. A
//! protected DOCX is decrypted in memory; a protected PDF goes as it is,
//! since pdf.js asks for its password.

use std::io::Read;

use uuid::Uuid;

use crate::documents::Format;
use crate::readers::docx_package_bytes;
use crate::workspace::Workspace;
use crate::{Error, Result};

impl Workspace {
    /// The bytes of the PDF or DOCX document `id`; `password` opens a
    /// protected DOCX.
    pub fn document_bytes(&self, id: Uuid, password: Option<&str>) -> Result<Vec<u8>> {
        let (doc, mut file, full) = self.open_valid(id)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|e| Error::io(&full, e))?;
        if doc.format != Format::Docx {
            return Ok(bytes);
        }
        let package = docx_package_bytes(&bytes, password).map_err(|e| Error::read(id, e))?;
        Ok(package.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;
    use crate::documents::test_docs::write_doc;
    use crate::readers::read_docx;
    use crate::readers::test_docx::docx;
    use crate::workspace::test_env::TestEnv;

    fn indexed(workspace: &Workspace, path: &str, content: &[u8]) -> Uuid {
        let path = write_doc(workspace, path, content);
        workspace.index_file(&path).unwrap().unwrap().id
    }

    #[test]
    fn document_bytes_of_unprotected_docx_are_the_file_bytes() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let content = docx("<w:p><w:r><w:t>Aberto</w:t></w:r></w:p>");
        let doc = indexed(&workspace, "Aberto.docx", &content);

        assert_eq!(workspace.document_bytes(doc, None).unwrap(), content);
    }

    /// Saved by LibreOffice with the password `folio`.
    const PROTECTED: &[u8] = include_bytes!("../../tests/fixtures/docx/protegido.docx");

    #[test]
    fn document_bytes_of_protected_docx_with_password_are_decrypted() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Contrato.docx", PROTECTED);

        let bytes = workspace.document_bytes(doc, Some("folio")).unwrap();

        // The decrypted package opens without a password.
        let opened = read_docx(&bytes, None).unwrap();
        assert_eq!(opened, read_docx(PROTECTED, Some("folio")).unwrap());
    }

    #[test]
    fn document_bytes_of_protected_docx_without_password_fails_with_password_required() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Contrato.docx", PROTECTED);

        let result = workspace.document_bytes(doc, None);

        assert!(matches!(result, Err(Error::PasswordRequired(id)) if id == doc));
    }

    #[test]
    fn document_bytes_of_protected_docx_with_wrong_password_fails_with_wrong_password() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Contrato.docx", PROTECTED);

        let result = workspace.document_bytes(doc, Some("errada"));

        assert!(matches!(result, Err(Error::WrongPassword(id)) if id == doc));
    }

    #[test]
    fn document_bytes_of_pdf_are_the_file_bytes() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let content = b"%PDF-1.7\n% protegido ou nao, quem pede a senha e o pdf.js\n".to_vec();
        let doc = indexed(&workspace, "Artigo.pdf", &content);

        assert_eq!(
            workspace.document_bytes(doc, Some("folio")).unwrap(),
            content
        );
    }
}
