//! The format metadata of a document, cached in `document_metadata` (spec 04)
//! and extracted by the readers (spec 07, "Metadados").

mod store;

use std::fs;

use uuid::Uuid;

use crate::documents::file_facts::FileFacts;
use crate::documents::{Document, Format};
use crate::model::Metadata;
use crate::readers::{ReadError, read_docx_metadata, read_md, read_pdf_metadata};
use crate::workspace::Workspace;
use crate::{Error, Result};

impl Workspace {
    /// What the document `id` says about itself, from the cache while the
    /// file is unchanged. `None` when it is password protected: its content
    /// never reaches the index.
    pub fn document_metadata(&self, id: Uuid) -> Result<Option<Metadata>> {
        let doc = self.document(id)?;
        if !doc.valid {
            return Err(Error::InvalidDocument(id));
        }
        if doc.protected {
            return Ok(None);
        }
        // TXT never has metadata.
        if doc.format == Format::Txt {
            return Ok(Some(Metadata::default()));
        }
        self.cached_metadata(&doc).map(Some)
    }

    /// The metadata of `doc` from the cache, extracted again when its file
    /// changed since.
    fn cached_metadata(&self, doc: &Document) -> Result<Metadata> {
        let full = doc.path.resolve(self.root());
        let facts = FileFacts::read(&full)?;
        let cached = store::load(&self.conn(), doc.id, &facts).map_err(|e| self.db_error(e))?;
        if let Some(cached) = cached {
            return Ok(cached);
        }
        let bytes = fs::read(&full).map_err(|e| Error::io(&full, e))?;
        let metadata = extract(doc.format, &bytes).map_err(|e| Error::read(doc.id, e))?;
        store::save(&self.conn(), doc.id, &facts, &metadata).map_err(|e| self.db_error(e))?;
        Ok(metadata)
    }
}

/// The metadata of a file in `format`, read from its `bytes`.
fn extract(format: Format, bytes: &[u8]) -> std::result::Result<Metadata, ReadError> {
    match format {
        Format::Docx => read_docx_metadata(bytes),
        Format::Pdf => read_pdf_metadata(bytes),
        _ => Ok(read_md(bytes).metadata),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::test_docs::write_doc;
    use crate::readers::test_docx::{core, docx_with, zip_of};
    use crate::readers::test_pdf::{pdf_with_info, text};
    use crate::workspace::test_env::TestEnv;

    fn indexed(workspace: &Workspace, path: &str, content: &[u8]) -> Uuid {
        let path = write_doc(workspace, path, content);
        workspace.index_file(&path).unwrap().unwrap().id
    }

    const FRONT_MATTER: &[u8] =
        b"---\ntitle: Tese\nauthor: Ana\ntags: [fisica, notas]\n---\n\nCorpo\n";

    fn cached_title(workspace: &Workspace, id: Uuid) -> Option<String> {
        let sql = "SELECT title FROM document_metadata WHERE document_id = ?1";
        workspace
            .conn()
            .query_row(sql, [id.to_string()], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn md_metadata_is_extracted_and_cached_on_first_request() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Tese.md", FRONT_MATTER);

        let metadata = workspace.document_metadata(doc).unwrap().unwrap();

        assert_eq!(
            (metadata.title.as_deref(), metadata.author.as_deref()),
            (Some("Tese"), Some("Ana"))
        );
        assert_eq!(metadata.keywords, ["fisica", "notas"]);
        assert_eq!(cached_title(&workspace, doc).as_deref(), Some("Tese"));
    }

    #[test]
    fn cached_metadata_is_reused_while_file_is_unchanged() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Tese.md", FRONT_MATTER);
        workspace.document_metadata(doc).unwrap();
        // Same size and mtime, different title: only the cache knows "Tese".
        let same_size = String::from_utf8(FRONT_MATTER.to_vec())
            .unwrap()
            .replace("Tese", "Tema");
        write_doc(&workspace, "Tese.md", same_size.as_bytes());

        let metadata = workspace.document_metadata(doc).unwrap().unwrap();

        assert_eq!(metadata.title.as_deref(), Some("Tese"));
    }

    #[test]
    fn metadata_is_extracted_again_when_file_changed() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Tese.md", FRONT_MATTER);
        workspace.document_metadata(doc).unwrap();
        write_doc(&workspace, "Tese.md", b"---\ntitle: Nova tese\n---\n");

        let metadata = workspace.document_metadata(doc).unwrap().unwrap();

        assert_eq!(metadata.title.as_deref(), Some("Nova tese"));
        assert_eq!(cached_title(&workspace, doc).as_deref(), Some("Nova tese"));
    }

    #[test]
    fn docx_metadata_is_extracted_and_cached_on_first_request() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let core = core("<dc:title>Contrato</dc:title><dc:creator>Ana</dc:creator>");
        let docx = docx_with("", &[("docProps/core.xml", core.as_bytes())]);
        let doc = indexed(&workspace, "Contrato.docx", &docx);

        let metadata = workspace.document_metadata(doc).unwrap().unwrap();

        assert_eq!(
            (metadata.title.as_deref(), metadata.author.as_deref()),
            (Some("Contrato"), Some("Ana"))
        );
        assert_eq!(cached_title(&workspace, doc).as_deref(), Some("Contrato"));
    }

    #[test]
    fn pdf_metadata_is_extracted_and_cached_on_first_request() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let pages = [vec![text("Corpo.", 72.0, 700.0)]];
        let pdf = pdf_with_info(&pages, "/Title (Ata) /Author (Ana)");
        let doc = indexed(&workspace, "Ata.pdf", &pdf);

        let metadata = workspace.document_metadata(doc).unwrap().unwrap();

        assert_eq!(
            (metadata.title.as_deref(), metadata.pages),
            (Some("Ata"), Some(1))
        );
        assert_eq!(cached_title(&workspace, doc).as_deref(), Some("Ata"));
    }

    #[test]
    fn docx_with_broken_content_fails_with_read_failed() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        // A ZIP with `word/document.xml` passes the signature check.
        let broken = zip_of(&[("word/document.xml", b"<w:document><w:body>".as_slice())]);
        let doc = indexed(&workspace, "Quebrado.docx", &broken);

        let result = workspace.document_metadata(doc);

        assert!(matches!(result, Err(Error::ReadFailed(id)) if id == doc));
    }

    #[test]
    fn txt_document_has_empty_metadata() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Tese.txt", FRONT_MATTER);

        let metadata = workspace.document_metadata(doc).unwrap();

        assert_eq!(metadata, Some(Metadata::default()));
    }

    /// A DOCX in the Office encrypted container, as Word saves it with a
    /// password.
    fn encrypted_docx() -> Vec<u8> {
        let mut compound = cfb::CompoundFile::create(std::io::Cursor::new(Vec::new())).unwrap();
        for name in ["EncryptionInfo", "EncryptedPackage"] {
            compound.create_stream(name).unwrap();
        }
        compound.flush().unwrap();
        compound.into_inner().into_inner()
    }

    #[test]
    fn protected_document_has_no_metadata_extracted() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Contrato.docx", &encrypted_docx());

        let metadata = workspace.document_metadata(doc).unwrap();

        assert_eq!(metadata, None);
    }

    #[test]
    fn metadata_of_invalid_document_fails_with_invalid_document() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let binary = indexed(&workspace, "Imagem.md", b"\0\0PNG");

        let result = workspace.document_metadata(binary);

        assert!(matches!(result, Err(Error::InvalidDocument(id)) if id == binary));
    }

    #[test]
    fn recomputing_after_clearing_cache_gives_same_metadata() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Tese.md", FRONT_MATTER);
        let first = workspace.document_metadata(doc).unwrap();
        workspace
            .conn()
            .execute("DELETE FROM document_metadata", [])
            .unwrap();

        let again = workspace.document_metadata(doc).unwrap();

        assert_eq!(again, first);
    }
}
