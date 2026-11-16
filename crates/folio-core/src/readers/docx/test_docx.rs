//! DOCX packages built in memory for the reader tests.

use std::io::{Cursor, Write};

use zip::ZipWriter;
use zip::write::SimpleFileOptions;

pub use super::context::RELS;

/// The `w`, `r`, `wp`, `a` and `pic` namespaces of a WordprocessingML part.
pub const NAMESPACES: &str = concat!(
    r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" "#,
    r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" "#,
    r#"xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" "#,
    r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" "#,
    r#"xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture""#,
);

/// A DOCX whose `document.xml` body is `body`.
pub fn docx(body: &str) -> Vec<u8> {
    docx_with(body, &[])
}

/// A DOCX with `body` and the other `parts`, given as (name, content).
pub fn docx_with(body: &str, parts: &[(&str, &[u8])]) -> Vec<u8> {
    let document = format!("<w:document {NAMESPACES}><w:body>{body}</w:body></w:document>");
    let mut all = vec![("word/document.xml", document.as_bytes())];
    all.extend_from_slice(parts);
    zip_of(&all)
}

/// A ZIP holding `parts`, given as (name, content).
pub fn zip_of(parts: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, content) in parts {
        zip.start_file(*name, SimpleFileOptions::default()).unwrap();
        zip.write_all(content).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

/// A `styles.xml` part holding the `w:style` elements in `styles`.
pub fn styles(styles: &str) -> String {
    format!("<w:styles {NAMESPACES}>{styles}</w:styles>")
}

/// A `numbering.xml` part where `w:numId` 1 is bulleted and 2 is decimal,
/// at every level.
pub fn numbering() -> String {
    let levels = |format: &str| {
        (0..3)
            .map(|level| format!(r#"<w:lvl w:ilvl="{level}"><w:numFmt w:val="{format}"/></w:lvl>"#))
            .collect::<String>()
    };
    format!(
        concat!(
            "<w:numbering {}>",
            r#"<w:abstractNum w:abstractNumId="10">{}</w:abstractNum>"#,
            r#"<w:abstractNum w:abstractNumId="20">{}</w:abstractNum>"#,
            r#"<w:num w:numId="1"><w:abstractNumId w:val="10"/></w:num>"#,
            r#"<w:num w:numId="2"><w:abstractNumId w:val="20"/></w:num>"#,
            "</w:numbering>",
        ),
        NAMESPACES,
        levels("bullet"),
        levels("decimal"),
    )
}

/// A paragraph of the list `num_id` at `level`, holding `text`.
pub fn list_item(num_id: u32, level: u32, text: &str) -> String {
    format!(
        concat!(
            r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="{}"/><w:numId w:val="{}"/></w:numPr></w:pPr>"#,
            "<w:r><w:t>{}</w:t></w:r></w:p>",
        ),
        level, num_id, text
    )
}

/// A `document.xml.rels` part with the `relationships`, given as (id,
/// type, target); `hyperlink` targets are external.
pub fn rels(relationships: &[(&str, &str, &str)]) -> String {
    let base = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    let each = relationships.iter().map(|(id, kind, target)| {
        let mode = if *kind == "hyperlink" {
            r#" TargetMode="External""#
        } else {
            ""
        };
        format!(r#"<Relationship Id="{id}" Type="{base}/{kind}" Target="{target}"{mode}/>"#)
    });
    let ns = "http://schemas.openxmlformats.org/package/2006/relationships";
    format!(
        r#"<Relationships xmlns="{ns}">{}</Relationships>"#,
        each.collect::<String>()
    )
}

/// A `docProps/core.xml` part holding the `properties`, as Word writes
/// them (`dc:title`, `cp:keywords`, `dcterms:created`...).
pub fn core(properties: &str) -> String {
    format!(
        concat!(
            "<cp:coreProperties ",
            r#"xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" "#,
            r#"xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" "#,
            r#"xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">{}</cp:coreProperties>"#,
        ),
        properties
    )
}
