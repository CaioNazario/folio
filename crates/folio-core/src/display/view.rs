//! What the document view shows around the stage: breadcrumb, badge,
//! conversion menu and where the user stopped (Design spec 05; specs 06
//! and 08).

use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

use crate::Result;
use crate::documents::{Document, Format};
use crate::paths::RelativePath;
use crate::recents::ReadingPosition;
use crate::workspace::Workspace;

/// A document as the document view opens it.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DocumentView {
    pub document: Document,
    /// Names of the project and folders it is in, outermost first; empty at
    /// the root.
    pub folders: Vec<String>,
    /// It is the result of a conversion (spec 04, derived documents).
    pub derived: bool,
    /// Formats it can be converted to (spec 08).
    pub conversion_targets: Vec<Format>,
    /// Where the user stopped reading it, if anywhere (spec 06).
    pub position: Option<ReadingPosition>,
}

impl Workspace {
    /// The document `id` with what the view shows around its content.
    pub fn document_view(&self, id: Uuid) -> Result<DocumentView> {
        let document = self.document(id)?;
        Ok(DocumentView {
            folders: folder_names(&document.path),
            derived: self.converted_from(id)?.is_some(),
            conversion_targets: conversion_targets(&document),
            position: self.position(id)?,
            document,
        })
    }
}

/// Every format can be converted to every other (spec 08, the 12
/// conversions); an invalid document cannot be read, so it converts to none.
fn conversion_targets(document: &Document) -> Vec<Format> {
    if !document.valid {
        return Vec::new();
    }
    let others = Format::ALL
        .into_iter()
        .filter(|format| *format != document.format);
    others.collect()
}

/// The names of the directories above `path`, outermost first.
fn folder_names(path: &RelativePath) -> Vec<String> {
    let Some(parent) = path.parent() else {
        return Vec::new();
    };
    let names = parent.as_bytes().split(|byte| *byte == b'/');
    names
        .map(|name| String::from_utf8_lossy(name).into_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::test_docs::write_doc;
    use crate::workspace::test_env::TestEnv;

    fn indexed(workspace: &Workspace, path: &str, content: &[u8]) -> Uuid {
        let path = write_doc(workspace, path, content);
        workspace.index_file(&path).unwrap().unwrap().id
    }

    #[test]
    fn document_view_lists_breadcrumb_folders() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let nested = indexed(&workspace, "Faculdade/Prova/Nota.txt", b"texto");
        let loose = indexed(&workspace, "Solta.txt", b"texto");

        let nested = workspace.document_view(nested).unwrap();
        let loose = workspace.document_view(loose).unwrap();

        assert_eq!(nested.folders, ["Faculdade", "Prova"]);
        assert_eq!(loose.folders, Vec::<String>::new());
    }

    /// `source` converted to `target`, as spec 08 records it.
    fn link(workspace: &Workspace, source: Uuid, target: Uuid) {
        let sql = "INSERT INTO conversions VALUES (?1, ?2, ?3, 'txt', 'md', 0, 0, 0)";
        let id = workspace.ids.new_id().to_string();
        workspace
            .conn()
            .execute(sql, (id, source.to_string(), target.to_string()))
            .unwrap();
    }

    #[test]
    fn document_view_marks_derived_document() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let source = indexed(&workspace, "Nota.txt", b"texto");
        let derived = indexed(&workspace, "Nota.md", b"texto");
        link(&workspace, source, derived);

        let flags = [source, derived].map(|id| workspace.document_view(id).unwrap().derived);

        assert_eq!(flags, [false, true]);
    }

    #[test]
    fn document_view_offers_other_formats_as_conversion_targets() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let md = indexed(&workspace, "Nota.md", b"# Nota");

        let view = workspace.document_view(md).unwrap();

        assert_eq!(
            view.conversion_targets,
            [Format::Pdf, Format::Docx, Format::Txt]
        );
    }

    #[test]
    fn document_view_of_invalid_document_offers_no_conversion() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        // Not a PDF inside: indexed as invalid (spec 04).
        let invalid = indexed(&workspace, "Fatura.pdf", b"texto");

        let view = workspace.document_view(invalid).unwrap();

        assert_eq!(view.conversion_targets, []);
    }

    #[test]
    fn document_view_includes_saved_position() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let txt = indexed(&workspace, "Nota.txt", b"texto");
        workspace.record_opened(txt).unwrap();
        let stopped = ReadingPosition::Scroll { fraction: 0.25 };
        workspace.save_position(txt, stopped).unwrap();

        let view = workspace.document_view(txt).unwrap();

        assert_eq!(view.position, Some(stopped));
    }
}
