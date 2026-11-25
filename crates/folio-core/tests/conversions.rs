//! The 12 conversions against sample documents, through the workspace as the
//! app runs them (spec 08, "Critério mínimo de qualidade" and "Critérios de
//! aceite"). Each derivative is read back and compared with the original:
//! no text lost or reordered, and the structure the original has (headings,
//! lists, emphasis) reaches every destination that can carry it.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use folio_core::clock::SystemClock;
use folio_core::db::GlobalDb;
use folio_core::documents::Format;
use folio_core::ids::UuidV7Ids;
use folio_core::integrity::Trash;
use folio_core::model::{Block, DocumentModel, Inline};
use folio_core::paths::RelativePath;
use folio_core::readers::{read_docx, read_md, read_pdf, read_txt};
use folio_core::services::Services;
use folio_core::workspace::{FolioDirStore, Workspace};

/// Conversions never delete anything.
struct NoTrash;

impl Trash for NoTrash {
    fn send(&self, _: &Path) -> Result<(), String> {
        unreachable!("a conversion sent a file to the trash")
    }
}

/// A workspace in a temp home, with one fixture copied into it.
struct Env {
    _home: tempfile::TempDir,
    workspace: Workspace,
}

fn env() -> Env {
    let home = tempfile::tempdir().unwrap();
    let (clock, ids) = (Arc::new(SystemClock), Arc::new(UuidV7Ids));
    let global = GlobalDb::open(&home.path().join("global.db"), clock.clone(), ids.clone());
    let services = Services {
        home: home.path().to_path_buf(),
        global: global.unwrap(),
        store: Box::new(FolioDirStore),
        clock,
        ids,
        trash: Arc::new(NoTrash),
    };
    let root = home.path().join("Folio");
    fs::create_dir(&root).unwrap();
    let workspace = Workspace::open(&root, &services).unwrap();
    Env {
        _home: home,
        workspace,
    }
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// The sample document of each source format.
fn source_fixture(format: Format) -> PathBuf {
    match format {
        Format::Txt => fixture("txt/windows-1252.txt"),
        Format::Md => fixture("md/notas.md"),
        Format::Docx => fixture("docx/relatorio.docx"),
        Format::Pdf => fixture("pdf/relatorio.pdf"),
    }
}

fn read(format: Format, bytes: &[u8]) -> DocumentModel {
    match format {
        Format::Txt => read_txt(bytes),
        Format::Md => read_md(bytes),
        Format::Docx => read_docx(bytes, None).unwrap(),
        Format::Pdf => read_pdf(bytes, None).unwrap(),
    }
}

/// Converts the sample of `source` to `target`; returns the original's
/// model and the derivative's, read back.
fn convert(source: Format, target: Format) -> (DocumentModel, DocumentModel) {
    let (original, written) = convert_bytes(source, target);
    (read(source, &original), read(target, &written))
}

/// Converts the sample of `source` to `target`; returns the bytes of the
/// original and of the derivative.
fn convert_bytes(source: Format, target: Format) -> (Vec<u8>, Vec<u8>) {
    let env = env();
    let fixture = source_fixture(source);
    let original = fs::read(&fixture).unwrap();
    let name = fixture.file_name().unwrap();
    fs::write(env.workspace.root().join(name), &original).unwrap();
    let path = RelativePath::from_bytes(name.as_encoded_bytes().to_vec()).unwrap();
    let id = env.workspace.index_file(&path).unwrap().unwrap().id;
    let derivative = env.workspace.convert(id, target, None).unwrap();
    let written = fs::read(derivative.path.resolve(env.workspace.root())).unwrap();
    assert_eq!(
        fs::read(&fixture).unwrap(),
        original,
        "the original changed"
    );
    (original, written)
}

/// The words of a model, in order: runs of letters and digits.
fn words(model: &DocumentModel) -> Vec<String> {
    let mut text = String::new();
    model
        .blocks
        .iter()
        .for_each(|block| block_text(block, &mut text));
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn block_text(block: &Block, text: &mut String) {
    match block {
        Block::Heading { content, .. } | Block::Paragraph(content) => inline_text(content, text),
        Block::List { items, .. } => items.iter().flatten().for_each(|b| block_text(b, text)),
        Block::Quote(blocks) => blocks.iter().for_each(|b| block_text(b, text)),
        Block::Code { text: code, .. } => text.push_str(code),
        Block::Table { rows } => rows
            .iter()
            .flatten()
            .for_each(|cell| inline_text(cell, text)),
        Block::Image { .. } | Block::Separator | Block::PageBreak => {}
    }
    text.push('\n');
}

fn inline_text(inlines: &[Inline], text: &mut String) {
    for inline in inlines {
        match inline {
            Inline::Text(t) | Inline::Code(t) => text.push_str(t),
            Inline::Bold(content) | Inline::Italic(content) => inline_text(content, text),
            Inline::Link { content, .. } => inline_text(content, text),
            Inline::LineBreak => text.push('\n'),
        }
        text.push(' ');
    }
}

/// Every word of `original` appears in `derivative`, in the same order;
/// the derivative may add words (a link's address in TXT).
fn assert_text_kept(original: &DocumentModel, derivative: &DocumentModel) {
    let (want, have) = (words(original), words(derivative));
    let mut rest = have.iter();
    for word in &want {
        assert!(
            rest.any(|w| w == word),
            "lost or reordered at {word:?}\noriginal:   {want:?}\nderivative: {have:?}"
        );
    }
}

/// Headings, lists and emphasis of a model.
#[derive(Debug, Default, PartialEq, Eq)]
struct Structure {
    headings: usize,
    lists: usize,
    emphasis: bool,
}

fn structure(model: &DocumentModel) -> Structure {
    let mut found = Structure::default();
    model
        .blocks
        .iter()
        .for_each(|block| count(block, &mut found));
    found
}

fn count(block: &Block, found: &mut Structure) {
    match block {
        Block::Heading { content, .. } => {
            found.headings += 1;
            found.emphasis |= emphasized(content);
        }
        Block::Paragraph(content) => found.emphasis |= emphasized(content),
        Block::List { items, .. } => {
            found.lists += 1;
            items.iter().flatten().for_each(|b| count(b, found));
        }
        Block::Quote(blocks) => blocks.iter().for_each(|b| count(b, found)),
        _ => {}
    }
}

fn emphasized(inlines: &[Inline]) -> bool {
    inlines.iter().any(|inline| match inline {
        Inline::Bold(_) | Inline::Italic(_) => true,
        Inline::Link { content, .. } => emphasized(content),
        _ => false,
    })
}

/// Checks one conversion: text always; structure unless the destination is
/// TXT, the one format without markup.
fn check(source: Format, target: Format) {
    let (original, derivative) = convert(source, target);
    if source != Format::Txt {
        let has = structure(&original);
        assert!(
            has.headings > 0 && has.lists > 0 && has.emphasis,
            "sample without structure: {has:?}"
        );
    }
    assert_text_kept(&original, &derivative);
    if target != Format::Txt {
        assert_eq!(structure(&derivative), structure(&original), "structure");
    }
}

macro_rules! conversions {
    ($($name:ident: $source:ident => $target:ident,)*) => {
        $(
            #[test]
            fn $name() {
                check(Format::$source, Format::$target);
            }
        )*
    };
}

conversions! {
    pdf_to_txt: Pdf => Txt,
    pdf_to_md: Pdf => Md,
    pdf_to_docx: Pdf => Docx,
    txt_to_pdf: Txt => Pdf,
    txt_to_md: Txt => Md,
    txt_to_docx: Txt => Docx,
    md_to_pdf: Md => Pdf,
    md_to_txt: Md => Txt,
    md_to_docx: Md => Docx,
    docx_to_pdf: Docx => Pdf,
    docx_to_txt: Docx => Txt,
    docx_to_md: Docx => Md,
}

/// The text of the part `name` of the DOCX package `docx`.
fn docx_part(docx: &[u8], name: &str) -> String {
    use std::io::Read;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut text = String::new();
    zip.by_name(name)
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    text
}

/// The DOCX reader does not read fonts or sizes, so the styles part of
/// the derivative is checked against the typography of the original.
#[test]
fn pdf_to_docx_keeps_body_and_heading_typography() {
    let (original, docx) = convert_bytes(Format::Pdf, Format::Docx);
    let typography = read(Format::Pdf, &original).typography;
    let styles = docx_part(&docx, "word/styles.xml");

    assert_eq!(typography.body.family.as_deref(), Some("Liberation Serif"));
    let body_size = typography.body.size.unwrap();
    assert!(styles.contains(&format!(
        r#"<w:rFonts w:ascii="Liberation Serif" w:hAnsi="Liberation Serif" w:eastAsia="Liberation Serif" w:cs="Liberation Serif"/><w:sz w:val="{body_size}"/>"#
    )), "{styles}");
    let heading_size = typography.headings[0].size.unwrap();
    assert!(
        styles.contains(&format!(r#"<w:sz w:val="{heading_size}"/>"#)),
        "{styles}"
    );
}
