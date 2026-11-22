//! Helpers for the PDF writer tests: models in, PDFs and the text the PDF
//! reader finds in them out.

use super::write_pdf;
use crate::model::{Block, DocumentModel, Inline};

pub fn text(text: &str) -> Inline {
    Inline::Text(text.to_string())
}

pub fn paragraph(content: &str) -> Block {
    Block::Paragraph(vec![text(content)])
}

/// The PDF of a model holding only `blocks`.
pub fn pdf_of(blocks: Vec<Block>) -> Vec<u8> {
    let model = DocumentModel {
        blocks,
        ..DocumentModel::default()
    };
    write_pdf(&model).unwrap()
}

/// The blocks the PDF reader finds in the PDF of `blocks`.
pub fn read_back(blocks: Vec<Block>) -> Vec<Block> {
    crate::readers::read_pdf(&pdf_of(blocks), None)
        .unwrap()
        .blocks
}

pub fn link(target: &str, content: &str) -> Inline {
    Inline::Link {
        target: target.to_string(),
        content: vec![text(content)],
    }
}

/// Whether `pdf` holds a link annotation to `uri`, written with or
/// without a space after the key.
pub fn has_uri(pdf: &[u8], uri: &str) -> bool {
    let found = |entry: String| {
        pdf.windows(entry.len())
            .any(|window| window == entry.as_bytes())
    };
    found(format!("/URI({uri})")) || found(format!("/URI ({uri})"))
}

pub fn list(ordered: bool, items: &[&str]) -> Block {
    let items = items.iter().map(|item| vec![paragraph(item)]).collect();
    Block::List { ordered, items }
}

/// The text of `blocks`, in order, with one space between blocks.
pub fn plain_text(blocks: &[Block]) -> String {
    let mut words = Vec::new();
    for block in blocks {
        match block {
            Block::Heading { content, .. } | Block::Paragraph(content) => {
                words.push(inline_text(content))
            }
            Block::List { items, .. } => words.extend(items.iter().map(|item| plain_text(item))),
            _ => {}
        }
    }
    words.join(" ")
}

pub fn inline_text(content: &[Inline]) -> String {
    let text = content.iter().map(|inline| match inline {
        Inline::Text(text) | Inline::Code(text) => text.clone(),
        Inline::Bold(inner) | Inline::Italic(inner) | Inline::Link { content: inner, .. } => {
            inline_text(inner)
        }
        Inline::LineBreak => " ".to_string(),
    });
    text.collect()
}

/// The text the PDF reader finds in the PDF of `blocks`.
pub fn text_read_back(blocks: Vec<Block>) -> String {
    plain_text(&read_back(blocks))
}

/// `text` with each run of whitespace as one space.
pub fn words(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn row(cells: &[&str]) -> Vec<Vec<Inline>> {
    cells.iter().map(|cell| vec![text(cell)]).collect()
}

pub fn image(mime: &str, bytes: Vec<u8>, alt: &str) -> Block {
    Block::Image {
        bytes,
        mime: mime.to_string(),
        alt: alt.to_string(),
    }
}
