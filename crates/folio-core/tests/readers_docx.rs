//! The DOCX reader against a sample file (spec 07, "Critérios de aceite").
//!
//! `relatorio.docx` was saved by LibreOffice from an HTML page; its
//! `docProps/core.xml` was then replaced by one as Word writes it, since
//! the HTML import leaves out dates and joins the keywords.

use std::fs;
use std::path::Path;

use folio_core::model::{Block, Inline, Metadata};
use folio_core::readers::{read_docx, read_docx_metadata};

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/docx")
        .join(name);
    fs::read(path).unwrap()
}

fn text(text: &str) -> Inline {
    Inline::Text(text.to_string())
}

fn para(content: &str) -> Block {
    Block::Paragraph(vec![text(content)])
}

fn heading(level: u8, content: &str) -> Block {
    Block::Heading {
        level,
        content: vec![text(content)],
    }
}

/// The paragraph with emphasis and a link.
fn emphasis() -> Block {
    let link = Inline::Link {
        target: "https://exemplo.com/".to_string(),
        content: vec![text("link")],
    };
    Block::Paragraph(vec![
        text("Um texto com "),
        Inline::Bold(vec![text("negrito")]),
        text(", "),
        Inline::Italic(vec![text("itálico")]),
        text(" e um "),
        link,
        text("."),
    ])
}

/// The ordered list with a bulleted list inside its first item.
fn list() -> Block {
    let nested = Block::List {
        ordered: false,
        items: vec![vec![para("Subitem")]],
    };
    Block::List {
        ordered: true,
        items: vec![vec![para("Primeiro"), nested], vec![para("Segundo")]],
    }
}

fn table() -> Block {
    let row = |a: &str, b: &str| vec![vec![text(a)], vec![text(b)]];
    Block::Table {
        rows: vec![row("Item", "Preço"), row("Café", "R$ 5")],
    }
}

fn image() -> Block {
    Block::Image {
        bytes: fixture("ponto.png"),
        mime: "image/png".to_string(),
        alt: "Um ponto vermelho".to_string(),
    }
}

#[test]
fn docx_fixture_reads_into_expected_model() {
    let model = read_docx(&fixture("relatorio.docx"), None).unwrap();

    let expected = [
        heading(1, "Relatório"),
        emphasis(),
        heading(2, "Lista"),
        list(),
        table(),
        image(),
        // The HTML's `page-break-before` comes out as a page break at the
        // end of the picture's paragraph.
        Block::PageBreak,
        para("Depois da quebra."),
    ];
    assert_eq!(model.blocks, expected);
}

#[test]
fn docx_fixture_metadata_comes_from_core_properties() {
    let metadata = read_docx_metadata(&fixture("relatorio.docx")).unwrap();

    let mut extra = serde_json::Map::new();
    extra.insert("comments".to_string(), "Notas do semestre".into());
    extra.insert("revision".to_string(), "3".into());
    let expected = Metadata {
        title: Some("Relatório de leitura".to_string()),
        author: Some("Ana Souza".to_string()),
        subject: Some("Leitura".to_string()),
        keywords: vec!["rust".to_string(), "leitura".to_string()],
        // 2026-11-15T10:00:00Z and 2026-11-16T12:30:00Z.
        created_at: Some(1_794_736_800_000),
        modified_at: Some(1_794_832_200_000),
        // One explicit page break.
        pages: Some(2),
        extra,
    };
    assert_eq!(metadata, expected);
}
