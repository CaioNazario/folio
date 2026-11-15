//! Content of the open workspace's documents, by id, for the Reader.

use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use super::image_files::ImageResolver;
use super::markdown::{MarkdownContent, markdown_display};
use super::{DISPLAY_TEXT_LIMIT, DisplayText, display_text};
use crate::documents::Document;
use crate::workspace::Workspace;
use crate::{Error, Result};

impl Workspace {
    /// The text of the TXT document `id`, cut past [`DISPLAY_TEXT_LIMIT`].
    pub fn text_content(&self, id: Uuid) -> Result<DisplayText> {
        let (_, file, full) = self.open_valid(id)?;
        display_text(file, DISPLAY_TEXT_LIMIT).map_err(|e| Error::io(&full, e))
    }

    /// The HTML of the Markdown document `id`, cut past
    /// [`DISPLAY_TEXT_LIMIT`], with its images resolved.
    pub fn markdown_content(&self, id: Uuid) -> Result<MarkdownContent> {
        let (doc, file, full) = self.open_valid(id)?;
        let mut images = ImageResolver::new(self, &doc.path);
        let mut resolve = |url: &str| images.resolve(url);
        markdown_display(file, DISPLAY_TEXT_LIMIT, &mut resolve).map_err(|e| Error::io(&full, e))
    }

    /// The document `id` and its file, open for reading, as long as it is
    /// valid and still on disk.
    fn open_valid(&self, id: Uuid) -> Result<(Document, File, PathBuf)> {
        let doc = self.document(id)?;
        if !doc.valid {
            return Err(Error::InvalidDocument(id));
        }
        let full = doc.path.resolve(self.root());
        let file = File::open(&full).map_err(|e| self.open_error(&doc, &full, e))?;
        Ok((doc, file, full))
    }

    /// A file gone from disk is a document not found: its directory is
    /// reconciled, so it becomes missing, and the stage says so (spec 05,
    /// "Quando a indexação roda"; spec 07, "Estados do palco").
    fn open_error(&self, doc: &Document, path: &Path, error: io::Error) -> Error {
        if error.kind() != io::ErrorKind::NotFound {
            return Error::io(path, error);
        }
        if let Err(failure) = self.reconcile_dir(doc.path.parent().as_ref()) {
            log::warn!("reconciling the directory of a missing document failed: {failure}");
        }
        Error::DocumentNotFound(doc.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::image_files::IMAGE_LIMIT;
    use crate::documents::test_docs::write_doc;
    use crate::paths::RelativePath;
    use crate::workspace::test_env::TestEnv;

    fn indexed(workspace: &Workspace, path: &str, content: &[u8]) -> Uuid {
        let path = write_doc(workspace, path, content);
        workspace.index_file(&path).unwrap().unwrap().id
    }

    #[test]
    fn text_content_of_invalid_document_fails_with_invalid_document() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let binary = indexed(&workspace, "Binario.txt", b"\0\0\0dados");

        let result = workspace.text_content(binary);

        assert!(matches!(result, Err(Error::InvalidDocument(id)) if id == binary));
    }

    #[test]
    fn text_content_of_file_gone_from_disk_fails_with_document_not_found() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let gone = indexed(&workspace, "Nota.txt", b"texto");
        std::fs::remove_file(workspace.root().join("Nota.txt")).unwrap();

        let result = workspace.text_content(gone);

        assert!(matches!(result, Err(Error::DocumentNotFound(id)) if id == gone));
    }

    fn listed_names(workspace: &Workspace, dir: Option<&RelativePath>) -> Vec<String> {
        let documents = workspace.list_documents(dir).unwrap();
        documents.into_iter().map(|doc| doc.name).collect()
    }

    #[test]
    fn opening_missing_document_returns_not_found_and_marks_it_absent() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let gone = indexed(&workspace, "Nota.txt", b"texto");
        indexed(&workspace, "Fica.txt", b"texto");
        std::fs::remove_file(workspace.root().join("Nota.txt")).unwrap();

        let result = workspace.text_content(gone);

        assert!(matches!(result, Err(Error::DocumentNotFound(id)) if id == gone));
        assert_eq!(listed_names(&workspace, None), ["Fica.txt"]);
    }

    #[test]
    fn opening_missing_document_reconciles_its_directory() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let gone = indexed(&workspace, "Notas/Velha.txt", b"texto");
        std::fs::remove_file(workspace.root().join("Notas/Velha.txt")).unwrap();
        std::fs::write(workspace.root().join("Notas/Nova.txt"), b"texto").unwrap();

        _ = workspace.text_content(gone);

        let notas = RelativePath::from_bytes(b"Notas".to_vec()).unwrap();
        assert_eq!(listed_names(&workspace, Some(&notas)), ["Nova.txt"]);
    }

    #[test]
    fn opening_document_in_missing_directory_marks_it_absent() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let gone = indexed(&workspace, "Notas/Doc.txt", b"texto");
        std::fs::remove_dir_all(workspace.root().join("Notas")).unwrap();

        let result = workspace.text_content(gone);

        assert!(matches!(result, Err(Error::DocumentNotFound(id)) if id == gone));
        let notas = RelativePath::from_bytes(b"Notas".to_vec()).unwrap();
        assert!(listed_names(&workspace, Some(&notas)).is_empty());
    }

    #[test]
    fn md_content_of_invalid_document_fails_with_invalid_document() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let binary = indexed(&workspace, "Imagem.md", b"\0\0PNG");

        let result = workspace.markdown_content(binary);

        assert!(matches!(result, Err(Error::InvalidDocument(id)) if id == binary));
    }

    /// A PNG signature followed by some bytes.
    const PNG: &[u8] = b"\x89PNG\r\n\x1a\nIHDR";

    fn html_of(workspace: &Workspace, markdown: &str) -> String {
        let doc = indexed(workspace, "Notas/Doc.md", markdown.as_bytes());
        workspace.markdown_content(doc).unwrap().html
    }

    #[test]
    fn md_image_inside_workspace_is_embedded_as_data_url() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        write_doc(&workspace, "Notas/img/g.png", PNG);

        let html = html_of(&workspace, "![gráfico](img/g.png)\n");

        let data = "data:image/png;base64,iVBORw0KGgpJSERS";
        assert!(
            html.contains(&format!(r#"<img src="{data}" alt="gráfico">"#)),
            "{html}"
        );
    }

    #[test]
    fn md_image_with_unrecognized_content_becomes_placeholder() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        write_doc(&workspace, "Notas/falso.png", b"<html>nada</html>");

        let html = html_of(&workspace, "![um falso](falso.png)\n");

        assert!(
            html.contains(r#"<span data-folio-image="unsupported">um falso</span>"#),
            "{html}"
        );
    }

    #[test]
    fn md_missing_image_becomes_placeholder() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();

        let html = html_of(&workspace, "![sumiu](nao-existe.png)\n");

        assert!(
            html.contains(r#"<span data-folio-image="missing">sumiu</span>"#),
            "{html}"
        );
    }

    #[test]
    fn md_image_outside_workspace_becomes_placeholder() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        std::fs::write(env.home().join("fora.png"), PNG).unwrap();

        let html = html_of(&workspace, "![a](../../fora.png) ![b](/etc/x.png)\n");

        assert!(
            html.contains(r#"<span data-folio-image="outside">a</span>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<span data-folio-image="outside">b</span>"#),
            "{html}"
        );
    }

    #[test]
    fn md_remote_image_becomes_placeholder_with_link() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();

        let html = html_of(&workspace, "![logo](https://exemplo.org/logo.png)\n");

        let link = r#"<a href="https://exemplo.org/logo.png">https://exemplo.org/logo.png</a>"#;
        assert!(
            html.contains(&format!(
                r#"<span data-folio-image="remote">logo {link}</span>"#
            )),
            "{html}"
        );
    }

    /// A PNG of exactly `size` bytes.
    fn png_of_size(size: u64) -> Vec<u8> {
        let mut bytes = PNG.to_vec();
        bytes.resize(size as usize, 0);
        bytes
    }

    #[test]
    fn md_image_over_10_mb_becomes_placeholder() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        write_doc(
            &workspace,
            "Notas/grande.png",
            &png_of_size(IMAGE_LIMIT + 1),
        );

        let html = html_of(&workspace, "![grande](grande.png)\n");

        assert!(html.contains(r#"<span data-folio-image="too-large">grande</span>"#));
    }

    #[test]
    fn md_images_past_50_mb_per_document_become_placeholders() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        write_doc(&workspace, "Notas/foto.png", &png_of_size(IMAGE_LIMIT));

        let html = html_of(&workspace, &"![foto](foto.png)\n\n".repeat(6));

        let embedded = html.matches("<img src=\"data:image/png").count();
        let refused = html
            .matches(r#"<span data-folio-image="too-large">foto</span>"#)
            .count();
        assert_eq!((embedded, refused), (5, 1));
    }
}
