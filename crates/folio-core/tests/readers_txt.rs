//! The TXT reader against sample files (spec 07, "Critérios de aceite").

use std::fs;
use std::path::Path;

use folio_core::model::{Block, Inline};
use folio_core::readers::read_txt;

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/txt")
        .join(name);
    fs::read(path).unwrap()
}

fn text(text: &str) -> Inline {
    Inline::Text(text.to_string())
}

#[test]
fn txt_fixtures_decode_with_correct_accents() {
    let expected = [
        Block::Paragraph(vec![
            text("Relatório de ação"),
            Inline::LineBreak,
            text("Coração e pão – 5€"),
        ]),
        Block::Paragraph(vec![text("Segundo parágrafo.")]),
    ];

    for name in ["windows-1252.txt", "utf16le-bom.txt"] {
        assert_eq!(read_txt(&fixture(name)).blocks, expected, "{name}");
    }
}
