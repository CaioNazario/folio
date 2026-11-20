//! The DOCX reader: WordprocessingML parts translated into the document
//! model (spec 07, "Leitores").

mod body;
mod context;
mod images;
mod inlines;
mod lists;
mod metadata;
mod numbering;
mod package;
mod page_breaks;
mod protection;
mod rels;
mod styles;
mod tables;
#[cfg(test)]
pub(crate) mod test_docx;
mod xml;

use context::Context;
use metadata::metadata_of;
pub use metadata::read_docx_metadata;
use package::Package;
pub use protection::package_bytes;

use super::ReadError;
use crate::model::DocumentModel;

/// The document model of a DOCX file; `password` opens a protected one.
pub fn read_docx(bytes: &[u8], password: Option<&str>) -> Result<DocumentModel, ReadError> {
    let bytes = package_bytes(bytes, password)?;
    let mut package = Package::open(&bytes)?;
    let document = package.xml("word/document.xml")?;
    let context = Context::of(&mut package)?;
    // A document without a body is empty, not broken.
    let blocks = document
        .child("body")
        .map(|body| body::blocks_of(body, &context));
    Ok(DocumentModel {
        metadata: metadata_of(&mut package, &document)?,
        blocks: blocks.unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::test_docx::{NAMESPACES, docx, docx_with, styles, zip_of};
    use super::*;
    use crate::model::{Block, Inline};

    fn blocks(body: &str) -> Vec<Block> {
        read_docx(&docx(body), None).unwrap().blocks
    }

    fn para(text: &str) -> Block {
        Block::Paragraph(vec![Inline::Text(text.to_string())])
    }

    #[test]
    fn read_docx_maps_plain_paragraphs() {
        let body = concat!(
            "<w:p><w:r><w:t>Primeiro.</w:t></w:r></w:p>",
            "<w:p><w:r><w:t xml:space=\"preserve\">Segundo </w:t></w:r>",
            "<w:r><w:t>parágrafo.</w:t></w:r></w:p>",
        );

        assert_eq!(
            blocks(body),
            [para("Primeiro."), para("Segundo parágrafo.")]
        );
    }

    fn heading(level: u8, text: &str) -> Block {
        Block::Heading {
            level,
            content: vec![Inline::Text(text.to_string())],
        }
    }

    /// A paragraph with the style `id` holding `text`.
    fn styled(id: &str, text: &str) -> String {
        format!(r#"<w:p><w:pPr><w:pStyle w:val="{id}"/></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"#)
    }

    #[test]
    fn read_docx_maps_heading_styles_by_name_with_levels() {
        // Word in Portuguese names the style ids after the translated name.
        let styles = styles(concat!(
            r#"<w:style w:type="paragraph" w:styleId="Ttulo1"><w:name w:val="heading 1"/></w:style>"#,
            r#"<w:style w:type="paragraph" w:styleId="Ttulo3"><w:name w:val="heading 3"/></w:style>"#,
        ));
        let body = styled("Ttulo1", "Um") + &styled("Ttulo3", "Três");

        let model = read_docx(
            &docx_with(&body, &[("word/styles.xml", styles.as_bytes())]),
            None,
        )
        .unwrap();

        assert_eq!(model.blocks, [heading(1, "Um"), heading(3, "Três")]);
    }

    #[test]
    fn read_docx_maps_page_break_to_page_break_block() {
        let body = concat!(
            "<w:p><w:r><w:t>Capa</w:t><w:br w:type=\"page\"/></w:r>",
            "<w:r><w:t>Corpo</w:t></w:r></w:p>",
        );

        assert_eq!(
            blocks(body),
            [para("Capa"), Block::PageBreak, para("Corpo")]
        );
    }

    #[test]
    fn read_docx_skips_empty_paragraphs() {
        let body = concat!(
            "<w:p/><w:p><w:r><w:t>Capa</w:t></w:r></w:p>",
            "<w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>",
            "<w:p><w:r><w:rPr><w:b/></w:rPr></w:r><w:r><w:t>Corpo</w:t></w:r></w:p>",
        );

        assert_eq!(
            blocks(body),
            [para("Capa"), Block::PageBreak, para("Corpo")]
        );
    }

    #[test]
    fn read_docx_maps_quote_style_to_quote() {
        let styles = styles(concat!(
            r#"<w:style w:type="paragraph" w:styleId="Citao"><w:name w:val="Quote"/></w:style>"#,
            r#"<w:style w:type="paragraph" w:styleId="CitaoIntensa"><w:name w:val="Intense Quote"/></w:style>"#,
        ));
        let body = styled("Citao", "Ser ou não ser")
            + &styled("CitaoIntensa", "eis a questão")
            + &styled("Normal", "Fim.");

        let model = read_docx(
            &docx_with(&body, &[("word/styles.xml", styles.as_bytes())]),
            None,
        )
        .unwrap();

        let quote = Block::Quote(vec![para("Ser ou não ser"), para("eis a questão")]);
        assert_eq!(model.blocks, [quote, para("Fim.")]);
    }

    #[test]
    fn read_docx_of_document_without_body_has_no_blocks() {
        let document = format!("<w:document {NAMESPACES}/>");

        let model =
            read_docx(&zip_of(&[("word/document.xml", document.as_bytes())]), None).unwrap();

        assert_eq!(model.blocks, []);
    }

    fn failure(bytes: &[u8]) -> ReadError {
        read_docx(bytes, None).unwrap_err()
    }

    #[test]
    fn read_docx_fails_with_read_failed_on_broken_zip() {
        assert!(matches!(
            failure(b"PK\x03\x04 quebrado"),
            ReadError::Failed(_)
        ));
    }

    #[test]
    fn read_docx_fails_with_read_failed_on_missing_document_xml() {
        let zip = zip_of(&[("word/styles.xml", b"<w:styles/>".as_slice())]);

        assert_eq!(
            failure(&zip),
            ReadError::Failed("missing part word/document.xml".to_string())
        );
    }

    #[test]
    fn read_docx_fails_with_read_failed_on_malformed_xml() {
        let zip = zip_of(&[(
            "word/document.xml",
            b"<w:document><w:body></w:document>".as_slice(),
        )]);

        assert!(matches!(failure(&zip), ReadError::Failed(_)));
    }
}
