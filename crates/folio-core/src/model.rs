//! The document model: a neutral representation of a document that readers
//! produce and writers consume (spec 07, "Modelo de documento").
//!
//! Left out in 0.1.0: footnotes, page headers and footers, columns, colors,
//! fonts and sizes of each passage, comments and revisions, fields. The
//! typography of the document as a whole is kept: the face of the body
//! and of each heading level.

use serde_json::{Map, Value};

/// A document's content, independent of its file format.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocumentModel {
    pub metadata: Metadata,
    pub blocks: Vec<Block>,
    pub typography: Typography,
}

/// The faces a document sets its text in, when the reader knows them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Typography {
    pub body: Face,
    /// The face of each heading level, from level 1; shorter than six
    /// when the document has fewer levels.
    pub headings: Vec<Face>,
}

/// A font family, size and emphasis.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Face {
    pub family: Option<String>,
    /// In half-points, the unit of `w:sz`.
    pub size: Option<u16>,
    pub bold: bool,
    pub italic: bool,
}

/// What a document says about itself, as cached in `document_metadata`
/// (spec 04). Dates are the document's own, in ms UTC, not the
/// filesystem's.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Vec<String>,
    pub created_at: Option<i64>,
    pub modified_at: Option<i64>,
    /// Exact for PDF, approximate for DOCX, absent for MD and TXT.
    pub pages: Option<u32>,
    /// What is specific to the format.
    pub extra: Map<String, Value>,
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
