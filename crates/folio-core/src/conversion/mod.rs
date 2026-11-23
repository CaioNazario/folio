//! Conversion: a reader, the document model, then a writer, published as a
//! new file next to the original (spec 08, "Fluxo").

mod control;
mod md_assets;
mod queue;
mod read;
mod write;

use std::ffi::OsString;
use std::io::Read;
use std::path::Path;

use uuid::Uuid;

use crate::display::ImageResolver;
use crate::documents::{Document, Format};
use crate::model::DocumentModel;
use crate::paths::RelativePath;
use crate::workspace::Workspace;
use crate::{Error, Result};
pub use control::{Control, Progress, Stage};
pub use queue::{ConversionEvent, ConversionQueue, Emitter, Job, Runner};
use read::read_model;
use write::publish_derivative;

impl Workspace {
    /// Converts the document `id` to `target`, publishing the derivative in
    /// the original's directory, and returns it as indexed. `password`
    /// opens a protected original.
    pub fn convert(&self, id: Uuid, target: Format, password: Option<&str>) -> Result<Document> {
        self.convert_with(id, target, password, &Control::silent())
    }

    /// [`Workspace::convert`], reporting progress to `control` and stopping
    /// when it is cancelled.
    pub fn convert_with(
        &self,
        id: Uuid,
        target: Format,
        password: Option<&str>,
        control: &Control,
    ) -> Result<Document> {
        let (original, bytes) = self.read_original(id, target)?;
        control.report(Stage::Reading, None);
        let mut images = ImageResolver::new(self, &original.path);
        let model = read_model(original.format, &bytes, password, control, &mut |url| {
            images.lookup(url)
        })
        .map_err(|e| Error::read(id, e))?;
        control.report(Stage::Writing, None);
        let published = self.publish(&original, target, &model, control)?;
        let path = RelativePath::from_absolute(self.root(), &published)?;
        // The name keeps the original's stem, so it is always a document.
        let derivative = self.index_file(&path)?.expect("a derivative is a document");
        // The link keeps both files' mtimes as they are now (spec 04).
        let original = self.index_file(&original.path)?.unwrap_or(original);
        self.record_conversion(&original, &derivative);
        Ok(derivative)
    }

    /// The original `id` and its bytes, once it is known to convert to
    /// `target`.
    fn read_original(&self, id: Uuid, target: Format) -> Result<(Document, Vec<u8>)> {
        let (original, mut file, full) = self.open_valid(id)?;
        if original.format == target {
            return Err(Error::ConversionUnavailable(id, target));
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|e| Error::io(&full, e))?;
        Ok((original, bytes))
    }

    /// Writes `model` in `target` next to `original` (next to the link, for
    /// a symlink, spec 04) and returns the derivative's path.
    fn publish(
        &self,
        original: &Document,
        target: Format,
        model: &DocumentModel,
        control: &Control,
    ) -> Result<std::path::PathBuf> {
        let full = original.path.resolve(self.root());
        let dir = full.parent().unwrap_or(self.root());
        let name = derivative_name(original, target);
        publish_derivative(self.tmp_dir(), dir, &name, model, target, control)
            .map_err(|failure| failure.into_error(original.id))
    }
}

/// The original's name with the extension of `target`: `Trabalho.pdf` →
/// `Trabalho.docx`.
fn derivative_name(original: &Document, target: Format) -> OsString {
    let name = Path::new(original.path.file_name());
    name.with_extension(target.as_str()).into_os_string()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;
    use crate::documents::test_docs::write_doc;
    use crate::model::{Block, DocumentModel};
    use crate::readers::test_docx::zip_of;
    use crate::workspace::test_env::TestEnv;
    use crate::writers::write_docx;

    fn indexed(workspace: &Workspace, path: &str, content: &[u8]) -> Uuid {
        let path = write_doc(workspace, path, content);
        workspace.index_file(&path).unwrap().unwrap().id
    }

    #[test]
    fn convert_publishes_derivative_next_to_original() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let txt = indexed(&workspace, "Notas/Aula.txt", b"Primeira linha\n");

        let derivative = workspace.convert(txt, Format::Md, None).unwrap();

        assert_eq!(derivative.path.display(), "Notas/Aula.md");
        let written = fs::read_to_string(workspace.root().join("Notas/Aula.md")).unwrap();
        assert_eq!(written, "Primeira linha\n");
    }

    #[test]
    fn convert_records_link_from_original_to_derivative() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let txt = indexed(&workspace, "Aula.txt", b"texto\n");
        env.clock.set(7_000);

        let derivative = workspace.convert(txt, Format::Md, None).unwrap();

        let link = workspace.converted_from(derivative.id).unwrap().unwrap();
        assert_eq!(
            (link.document.id, link.source_format, link.target_format),
            (txt, Format::Txt, Format::Md)
        );
        assert_eq!(
            (
                link.converted_at,
                link.source_changed,
                link.derivative_changed
            ),
            (7_000, false, false)
        );
    }

    #[test]
    fn convert_to_own_format_fails_with_conversion_unavailable() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let txt = indexed(&workspace, "Aula.txt", b"texto\n");

        let result = workspace.convert(txt, Format::Txt, None);

        assert!(matches!(
            result,
            Err(Error::ConversionUnavailable(id, Format::Txt)) if id == txt
        ));
        assert!(!workspace.root().join("Aula (1).txt").exists());
    }

    #[test]
    fn convert_invalid_document_fails_with_invalid_document() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let binary = indexed(&workspace, "Binario.txt", b"\0\0\0dados");

        let result = workspace.convert(binary, Format::Md, None);

        assert!(matches!(result, Err(Error::InvalidDocument(id)) if id == binary));
        assert!(!workspace.root().join("Binario.md").exists());
    }

    fn is_empty(dir: &Path) -> bool {
        fs::read_dir(dir).unwrap().next().is_none()
    }

    #[test]
    fn failed_conversion_leaves_no_file_no_link_and_no_temp() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        // A DOCX by its signature whose body is not XML.
        let parts: &[(&str, &[u8])] = &[("word/document.xml", b"<w:document><")];
        let broken = indexed(&workspace, "Quebrado.docx", &zip_of(parts));

        let result = workspace.convert(broken, Format::Md, None);

        assert!(matches!(result, Err(Error::ReadFailed(id)) if id == broken));
        assert!(!workspace.root().join("Quebrado.md").exists());
        assert!(workspace.derivatives(broken).unwrap().is_empty());
        assert!(is_empty(workspace.tmp_dir()));
    }

    #[test]
    fn writer_failure_becomes_write_failed() {
        let id = Uuid::from_u128(9);

        let error = Error::write(id, "typst: unknown variable");

        assert!(matches!(error, Error::WriteFailed(failed) if failed == id));
    }

    /// A PNG header of 4 × 3 pixels.
    fn png() -> Vec<u8> {
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        png.extend_from_slice(&[0, 0, 0, 4, 0, 0, 0, 3, 8, 6, 0, 0, 0]);
        png
    }

    /// A DOCX with one image, alt text "Diagrama".
    fn docx_with_image() -> Vec<u8> {
        let image = Block::Image {
            bytes: png(),
            mime: "image/png".to_string(),
            alt: "Diagrama".to_string(),
        };
        let model = DocumentModel {
            blocks: vec![image],
            ..DocumentModel::default()
        };
        write_docx(&model).unwrap()
    }

    #[test]
    fn convert_to_md_writes_images_to_assets_folder() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let docx = indexed(&workspace, "Notas/Aula.docx", &docx_with_image());

        workspace.convert(docx, Format::Md, None).unwrap();

        let notas = workspace.root().join("Notas");
        let md = fs::read_to_string(notas.join("Aula.md")).unwrap();
        assert_eq!(md, "![Diagrama](Aula.assets/imagem-1.png)\n");
        let image = fs::read(notas.join("Aula.assets/imagem-1.png")).unwrap();
        assert_eq!(image, png());
    }

    #[test]
    fn md_and_assets_folder_share_suffix_when_folder_name_is_taken() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let docx = indexed(&workspace, "Aula.docx", &docx_with_image());
        fs::create_dir(workspace.root().join("Aula.assets")).unwrap();

        let derivative = workspace.convert(docx, Format::Md, None).unwrap();

        assert_eq!(derivative.path.display(), "Aula (1).md");
        let html = workspace.markdown_content(derivative.id).unwrap().html;
        assert!(html.contains("<img src=\"data:image/png;base64,"), "{html}");
        assert!(
            workspace
                .root()
                .join("Aula (1).assets/imagem-1.png")
                .is_file()
        );
        assert!(is_empty(&workspace.root().join("Aula.assets")));
    }

    #[test]
    fn name_collision_adds_suffix_and_keeps_existing_file() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let txt = indexed(&workspace, "Aula.txt", b"novo\n");
        write_doc(&workspace, "Aula.md", b"antigo");

        let derivative = workspace.convert(txt, Format::Md, None).unwrap();

        assert_eq!(derivative.path.display(), "Aula (1).md");
        let existing = fs::read_to_string(workspace.root().join("Aula.md")).unwrap();
        assert_eq!(existing, "antigo");
    }

    /// A DOCX in the Office encrypted container.
    fn encrypted_docx() -> Vec<u8> {
        let cursor = std::io::Cursor::new(Vec::new());
        let mut compound = cfb::CompoundFile::create(cursor).unwrap();
        for name in ["EncryptionInfo", "EncryptedPackage"] {
            compound.create_stream(name).unwrap();
        }
        compound.flush().unwrap();
        compound.into_inner().into_inner()
    }

    #[test]
    fn convert_protected_document_without_password_fails_with_password_required() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let docx = indexed(&workspace, "Contrato.docx", &encrypted_docx());

        let result = workspace.convert(docx, Format::Txt, None);

        assert!(matches!(result, Err(Error::PasswordRequired(id)) if id == docx));
        assert!(!workspace.root().join("Contrato.txt").exists());
    }

    #[test]
    fn convert_to_docx_and_pdf_indexes_valid_documents() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let md = indexed(&workspace, "Tese.md", b"# Tese\n\nCorpo\n");

        let docx = workspace.convert(md, Format::Docx, None).unwrap();
        let pdf = workspace.convert(md, Format::Pdf, None).unwrap();

        assert_eq!((docx.format, docx.valid), (Format::Docx, true));
        assert_eq!((pdf.format, pdf.valid), (Format::Pdf, true));
        assert_eq!(workspace.derivatives(md).unwrap().len(), 2);
    }

    #[test]
    fn link_records_original_mtime_from_disk_not_from_index() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let txt = indexed(&workspace, "Aula.txt", b"texto\n");
        // Edited after indexing; no scan ran since.
        fs::write(workspace.root().join("Aula.txt"), b"editado\n").unwrap();

        let derivative = workspace.convert(txt, Format::Md, None).unwrap();

        let link = workspace.converted_from(derivative.id).unwrap().unwrap();
        assert!(!link.source_changed);
    }

    /// A control that keeps every progress it receives.
    fn recording() -> (
        std::sync::Arc<Control>,
        std::sync::Arc<std::sync::Mutex<Vec<Progress>>>,
    ) {
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = seen.clone();
        let control = Control::new(move |progress| sink.lock().unwrap().push(progress));
        (std::sync::Arc::new(control), seen)
    }

    #[test]
    fn convert_reports_reading_then_writing() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let txt = indexed(&workspace, "Aula.txt", b"texto\n");
        let (control, seen) = recording();

        workspace
            .convert_with(txt, Format::Md, None, &control)
            .unwrap();

        let stages: Vec<_> = seen.lock().unwrap().iter().map(|p| p.stage).collect();
        assert_eq!(stages, [Stage::Reading, Stage::Writing]);
    }

    /// A control that cancels itself when it hears `stage`.
    fn cancelling_at(stage: Stage) -> std::sync::Arc<Control> {
        let slot = std::sync::Arc::new(std::sync::OnceLock::<std::sync::Weak<Control>>::new());
        let me = slot.clone();
        let control = std::sync::Arc::new(Control::new(move |progress| {
            let control = me.get().and_then(std::sync::Weak::upgrade);
            if progress.stage == stage {
                control.inspect(|control| control.cancel());
            }
        }));
        slot.set(std::sync::Arc::downgrade(&control)).unwrap();
        control
    }

    #[test]
    fn cancelled_conversion_leaves_no_file_no_link_and_no_temp() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let docx = indexed(&workspace, "Aula.docx", &docx_with_image());

        let result = workspace.convert_with(docx, Format::Md, None, &cancelling_at(Stage::Writing));

        assert!(matches!(result, Err(Error::ConversionCancelled(id)) if id == docx));
        assert!(!workspace.root().join("Aula.md").exists());
        assert!(!workspace.root().join("Aula.assets").exists());
        assert!(workspace.derivatives(docx).unwrap().is_empty());
        assert!(is_empty(workspace.tmp_dir()));
    }

    fn two_page_pdf() -> Vec<u8> {
        use crate::readers::test_pdf::{pdf, text};
        pdf(&[
            vec![text("Primeira", 72.0, 700.0)],
            vec![text("Segunda", 72.0, 700.0)],
        ])
    }

    #[test]
    fn reading_pdf_reports_fraction_of_pages_read() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let pdf = indexed(&workspace, "Ata.pdf", &two_page_pdf());
        let (control, seen) = recording();

        workspace
            .convert_with(pdf, Format::Txt, None, &control)
            .unwrap();

        let reading: Vec<_> = seen
            .lock()
            .unwrap()
            .iter()
            .filter(|p| p.stage == Stage::Reading)
            .map(|p| p.fraction)
            .collect();
        assert_eq!(reading, [None, Some(0.5), Some(1.0)]);
    }

    #[test]
    fn cancel_while_reading_pdf_stops_before_next_page() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let pdf = indexed(&workspace, "Ata.pdf", &two_page_pdf());
        let control = cancelling_at(Stage::Reading);

        let result = workspace.convert_with(pdf, Format::Txt, None, &control);

        assert!(matches!(result, Err(Error::ConversionCancelled(id)) if id == pdf));
        assert!(!workspace.root().join("Ata.txt").exists());
    }

    #[test]
    fn md_local_image_reaches_docx() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        write_doc(&workspace, "Notas/img/g.png", &png());
        let md = indexed(&workspace, "Notas/Aula.md", b"![Diagrama](img/g.png)\n");

        let docx = workspace.convert(md, Format::Docx, None).unwrap();

        let bytes = fs::read(docx.path.resolve(workspace.root())).unwrap();
        let model = crate::readers::read_docx(&bytes, None).unwrap();
        assert!(
            matches!(&model.blocks[..], [Block::Image { alt, .. }] if alt == "Diagrama"),
            "{:?}",
            model.blocks
        );
    }

    /// A real 1 × 1 PNG, which Typst decodes.
    const PIXEL: &[u8] = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f,
        0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0xf8,
        0xcf, 0xc0, 0xf0, 0x1f, 0x00, 0x05, 0x00, 0x01, 0xff, 0x89, 0x99, 0x3d, 0x1d, 0x00, 0x00,
        0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn md_local_image_reaches_pdf() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        write_doc(&workspace, "Notas/g.png", PIXEL);
        let md = indexed(
            &workspace,
            "Notas/Aula.md",
            b"Texto\n\n![Diagrama](g.png)\n",
        );

        let pdf = workspace.convert(md, Format::Pdf, None).unwrap();

        let bytes = fs::read(pdf.path.resolve(workspace.root())).unwrap();
        assert_eq!(crate::writers::pdf_drawn(&bytes).image_widths.len(), 1);
    }

    #[test]
    fn md_remote_image_is_not_downloaded_and_becomes_link() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let md = indexed(
            &workspace,
            "Aula.md",
            b"![logo](https://exemplo.org/logo.png)\n",
        );

        let docx = workspace.convert(md, Format::Docx, None).unwrap();

        let bytes = fs::read(docx.path.resolve(workspace.root())).unwrap();
        let model = crate::readers::read_docx(&bytes, None).unwrap();
        let link = crate::model::Inline::Link {
            target: "https://exemplo.org/logo.png".to_string(),
            content: vec![crate::model::Inline::Text("logo".to_string())],
        };
        assert_eq!(model.blocks, [Block::Paragraph(vec![link])]);
    }
}
