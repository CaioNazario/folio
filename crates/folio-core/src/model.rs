//! The document model: a neutral representation of a document that readers
//! produce and writers consume (spec 07, "Modelo de documento").
//!
//! Left out in 0.1.0: footnotes, page headers and footers, columns, colors,
//! fonts and sizes, comments and revisions, fields.

/// A document's content, independent of its file format.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocumentModel {
    pub blocks: Vec<Block>,
}

/// A block-level element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// Level from 1 to 6.
    Heading {
        level: u8,
        content: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
    /// Each item is a list of blocks, so lists can nest.
    List {
        ordered: bool,
        items: Vec<Vec<Block>>,
    },
    Quote(Vec<Block>),
    Code {
        language: Option<String>,
        text: String,
    },
    /// Rows of cells; simple tables only, without merged cells.
    Table {
        rows: Vec<Vec<Vec<Inline>>>,
    },
    Image {
        bytes: Vec<u8>,
        mime: String,
        alt: String,
    },
    Separator,
    PageBreak,
}

/// An inline element inside a heading, paragraph or table cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    Text(String),
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Code(String),
    Link {
        target: String,
        content: Vec<Inline>,
    },
    LineBreak,
}
