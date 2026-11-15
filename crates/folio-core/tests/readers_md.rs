//! The Markdown reader against a sample file (spec 07, "Critérios de aceite").

use std::fs;
use std::path::Path;

use folio_core::model::{Block, Inline, Metadata};
use folio_core::readers::read_md;

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/md")
        .join(name);
    fs::read(path).unwrap()
}

fn text(text: &str) -> Inline {
    Inline::Text(text.to_string())
}

fn para(content: &str) -> Block {
    Block::Paragraph(vec![text(content)])
}

fn expected_metadata() -> Metadata {
    let mut extra = serde_json::Map::new();
    extra.insert("curso".to_string(), serde_json::json!("Engenharia"));
    Metadata {
        title: Some("Notas de estudo".to_string()),
        author: Some("Ana".to_string()),
        keywords: vec!["rust".to_string(), "leitura".to_string()],
        // 2026-11-15T00:00:00Z.
        created_at: Some(1_794_700_800_000),
        extra,
        ..Metadata::default()
    }
}

/// The title and the paragraph with emphasis and a link.
fn opening() -> [Block; 2] {
    let link = Inline::Link {
        target: "https://exemplo.org".to_string(),
        content: vec![text("link")],
    };
    let paragraph = Block::Paragraph(vec![
        text("Texto com "),
        Inline::Bold(vec![text("negrito")]),
        text(", "),
        Inline::Italic(vec![text("itálico")]),
        text(" e um "),
        link,
        text("."),
    ]);
    let title = Block::Heading {
        level: 1,
        content: vec![text("Notas de estudo")],
    };
    [title, paragraph]
}

fn nested_list() -> Block {
    let details = Block::List {
        ordered: false,
        items: vec![vec![para("detalhe")]],
    };
    Block::List {
        ordered: true,
        items: vec![vec![para("Primeiro"), details], vec![para("Segundo")]],
    }
}

fn table() -> Block {
    let row = |a: &str, b: &str| vec![vec![text(a)], vec![text(b)]];
    Block::Table {
        rows: vec![row("Tema", "Status"), row("Rust", "ok")],
    }
}

fn expected_blocks() -> Vec<Block> {
    let code = Block::Code {
        language: Some("rust".to_string()),
        text: "fn main() {}\n".to_string(),
    };
    let quote = Block::Quote(vec![para("Uma citação.")]);
    let mut blocks = opening().to_vec();
    blocks.extend([nested_list(), quote, code, table(), Block::Separator]);
    blocks
}

#[test]
fn md_fixture_reads_into_expected_model() {
    let model = read_md(&fixture("notas.md"));

    assert_eq!(model.metadata, expected_metadata());
    assert_eq!(model.blocks, expected_blocks());
}
