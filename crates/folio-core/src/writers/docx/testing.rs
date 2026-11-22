//! Helpers for the DOCX writer tests: models in, packages and parts out.

use std::io::{Cursor, Read};

use zip::ZipArchive;

use super::write_docx;
use crate::model::{Block, DocumentModel, Inline};

/// The names of the parts in the package `docx`.
pub fn part_names(docx: &[u8]) -> Vec<String> {
    let zip = ZipArchive::new(Cursor::new(docx)).unwrap();
    zip.file_names()
        .map(|name| name.unwrap().into_owned())
        .collect()
}

/// The text of the part `name` of the package `docx`.
pub fn part(docx: &[u8], name: &str) -> String {
    let mut zip = ZipArchive::new(Cursor::new(docx)).unwrap();
    let mut text = String::new();
    zip.by_name(name)
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    text
}

/// The blocks the DOCX reader finds in the DOCX of `blocks`.
pub fn read_back(blocks: Vec<Block>) -> Vec<Block> {
    blocks_of(&docx_of(blocks))
}

/// The DOCX of a model holding only `blocks`.
pub fn docx_of(blocks: Vec<Block>) -> Vec<u8> {
    let model = DocumentModel {
        blocks,
        ..DocumentModel::default()
    };
    write_docx(&model).unwrap()
}

/// The blocks the DOCX reader finds in `docx`.
pub fn blocks_of(docx: &[u8]) -> Vec<Block> {
    crate::readers::read_docx(docx, None).unwrap().blocks
}

/// `word/document.xml` of the DOCX of `blocks`.
pub fn document_xml(blocks: Vec<Block>) -> String {
    part(&docx_of(blocks), "word/document.xml")
}

pub fn text(text: &str) -> Inline {
    Inline::Text(text.to_string())
}

pub fn paragraph(content: &str) -> Block {
    Block::Paragraph(vec![text(content)])
}

pub fn link(target: &str, content: &str) -> Inline {
    Inline::Link {
        target: target.to_string(),
        content: vec![text(content)],
    }
}

pub fn list(ordered: bool, items: &[&str]) -> Block {
    let items = items.iter().map(|item| vec![paragraph(item)]).collect();
    Block::List { ordered, items }
}

pub fn row(cells: &[&str]) -> Vec<Vec<Inline>> {
    cells.iter().map(|cell| vec![text(cell)]).collect()
}

pub fn table(rows: Vec<Vec<Vec<Inline>>>) -> Block {
    Block::Table { rows }
}

/// A PNG of `width` × `height` pixels, header only.
pub fn png(width: u32, height: u32) -> Vec<u8> {
    let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    png.extend_from_slice(&width.to_be_bytes());
    png.extend_from_slice(&height.to_be_bytes());
    png.extend_from_slice(&[8, 6, 0, 0, 0]);
    png
}

pub fn image(mime: &str, bytes: Vec<u8>, alt: &str) -> Block {
    Block::Image {
        bytes,
        mime: mime.to_string(),
        alt: alt.to_string(),
    }
}
