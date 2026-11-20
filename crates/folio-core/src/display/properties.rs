//! Everything Folio knows about a document, for the Properties dialog (spec
//! 07, "Propriedades").

use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

use super::view::folder_names;
use crate::documents::{ConversionLink, Document};
use crate::model::Metadata;
use crate::workspace::Workspace;
use crate::{Error, Result};

/// The Properties dialog of a document, in one command.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DocumentProperties {
    /// Name, format, size and filesystem dates.
    pub document: Document,
    /// Names of the project and folders it is in, outermost first; empty at
    /// the root.
    pub location: Vec<String>,
    /// What the format says about it; `None` when nothing was extracted:
    /// password protected (`document.protected`) or unreadable
    /// (`document.valid` false).
    pub metadata: Option<Metadata>,
    /// The document it was converted from, if any.
    pub converted_from: Option<ConversionLink>,
    /// The documents converted from it.
    pub derivatives: Vec<ConversionLink>,
}

impl Workspace {
    /// The Properties of the document `id`.
    pub fn document_properties(&self, id: Uuid) -> Result<DocumentProperties> {
        let document = self.document(id)?;
        let metadata = self.readable_metadata(&document)?;
        Ok(DocumentProperties {
            location: folder_names(&document.path),
            document,
            metadata,
            converted_from: self.converted_from(id)?,
            derivatives: self.derivatives(id)?,
        })
    }

    /// The metadata of `document`; none when it is invalid, protected (spec
    /// 04) or its content cannot be read (the reason is logged): the rest of
    /// the Properties still shows.
    fn readable_metadata(&self, document: &Document) -> Result<Option<Metadata>> {
        if !document.valid {
            return Ok(None);
        }
        match self.document_metadata(document.id) {
            Err(Error::ReadFailed(_)) => Ok(None),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::Format;
    use crate::documents::test_docs::{MTIME, write_doc};
    use crate::readers::test_docx::zip_of;
    use crate::workspace::test_env::TestEnv;

    fn indexed(workspace: &Workspace, path: &str, content: &[u8]) -> Uuid {
        let path = write_doc(workspace, path, content);
        workspace.index_file(&path).unwrap().unwrap().id
    }

    #[test]
    fn properties_report_file_facts() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Faculdade/Notas/Aula.txt", b"texto");

        let properties = workspace.document_properties(doc).unwrap();

        let file = &properties.document;
        assert_eq!(
            (file.name.as_str(), file.format, file.size, file.modified_at),
            ("Aula.txt", Format::Txt, 5, MTIME)
        );
        assert_eq!(properties.location, ["Faculdade", "Notas"]);
    }

    #[test]
    fn properties_include_document_metadata() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(
            &workspace,
            "Tese.md",
            b"---\ntitle: Tese\nauthor: Ana\n---\n\nCorpo\n",
        );

        let properties = workspace.document_properties(doc).unwrap();

        let metadata = properties.metadata.unwrap();
        assert_eq!(
            (metadata.title.as_deref(), metadata.author.as_deref()),
            (Some("Tese"), Some("Ana"))
        );
    }

    /// A DOCX in the Office encrypted container.
    fn encrypted_docx() -> Vec<u8> {
        let mut compound = cfb::CompoundFile::create(std::io::Cursor::new(Vec::new())).unwrap();
        for name in ["EncryptionInfo", "EncryptedPackage"] {
            compound.create_stream(name).unwrap();
        }
        compound.flush().unwrap();
        compound.into_inner().into_inner()
    }

    #[test]
    fn properties_of_protected_document_say_metadata_not_extracted() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Contrato.docx", &encrypted_docx());

        let properties = workspace.document_properties(doc).unwrap();

        assert!(properties.document.protected);
        assert_eq!(properties.metadata, None);
    }

    /// Records `source` → `target` as spec 08 does, with the files as they
    /// were at `modified_at`.
    fn record(workspace: &Workspace, source: Uuid, target: Uuid, modified_at: i64) {
        let sql = "INSERT INTO conversions VALUES (?1, ?2, ?3, 'md', 'txt', 5000, ?4, ?4)";
        let id = workspace.ids.new_id().to_string();
        let params = (id, source.to_string(), target.to_string(), modified_at);
        workspace.conn().execute(sql, params).unwrap();
    }

    #[test]
    fn properties_list_origin_and_derivatives_with_outdated_warnings() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let md = indexed(&workspace, "Tese.md", b"# Tese");
        let txt = indexed(&workspace, "Tese.txt", b"Tese");
        // The source changed after the conversion: the derivative may be stale.
        record(&workspace, md, txt, MTIME - 1);

        let source = workspace.document_properties(md).unwrap();
        let derivative = workspace.document_properties(txt).unwrap();

        let ids: Vec<_> = source
            .derivatives
            .iter()
            .map(|link| link.document.id)
            .collect();
        assert_eq!(ids, [txt]);
        let origin = derivative.converted_from.unwrap();
        assert_eq!((origin.document.id, origin.source_changed), (md, true));
    }

    #[test]
    fn properties_of_invalid_document_still_report_file_facts() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let binary = indexed(&workspace, "Imagem.md", b"\0\0PNG");

        let properties = workspace.document_properties(binary).unwrap();

        assert_eq!(properties.document.name, "Imagem.md");
        assert_eq!(properties.metadata, None);
    }

    #[test]
    fn properties_of_document_with_unreadable_content_report_file_facts_without_metadata() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        // A ZIP with `document.xml` passes the signature; its XML is broken.
        let broken = zip_of(&[("word/document.xml", b"<w:document><w:body".as_slice())]);
        let doc = indexed(&workspace, "Contrato.docx", &broken);

        let properties = workspace.document_properties(doc).unwrap();

        assert_eq!(properties.document.name, "Contrato.docx");
        assert_eq!(properties.metadata, None);
    }
}
