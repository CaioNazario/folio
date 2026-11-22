//! The DOCX writer: a Word package built from the model with `zip` and
//! `quick-xml`, holding only the parts it needs (spec 08, "Escritores").

mod body;
mod core;
mod numbering;
mod package;
mod styles;
#[cfg(test)]
mod testing;

use crate::model::DocumentModel;
use body::{Body, Media, Written};
use package::{Part, Relationship, content_types, relationships_part, zip};

/// The namespaces of a WordprocessingML part.
const NAMESPACES: &str = concat!(
    r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" "#,
    r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" "#,
    r#"xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" "#,
    r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" "#,
    r#"xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture""#,
);
/// An A4 page, in twips.
const PAGE: (u32, u32) = (11906, 16838);
/// The page margins, 2.5 cm in twips.
const MARGIN: u32 = 1417;
/// The width of the text between the margins, in twips.
pub const TEXT_WIDTH: u32 = PAGE.0 - 2 * MARGIN;

/// The DOCX bytes of `model`; the error is the technical reason, for the log.
pub fn write_docx(model: &DocumentModel) -> Result<Vec<u8>, String> {
    let mut body = Body::default();
    body.blocks(&model.blocks);
    let mut parts = document_parts(body.finish());
    let core = core::core_properties(&model.metadata);
    parts.push(package_relationships(core.is_some()));
    parts.extend(core);
    parts.push(content_types(&parts));
    zip(&parts)
}

/// `document.xml`, the parts it points to and its relationships.
fn document_parts(written: Written) -> Vec<Part> {
    let mut linked = Linked {
        parts: Vec::new(),
        relationships: written.relationships,
    };
    linked.add("styles", xml_part("styles", &styles::styles()));
    if !written.lists.is_empty() {
        linked.add(
            "numbering",
            xml_part("numbering", &numbering::numbering(&written.lists)),
        );
    }
    linked.media(written.media);
    let document = format!("<w:body>{}{}</w:body>", written.xml, section());
    linked.finish(xml_part("document", &document))
}

/// The parts `document.xml` points to, with its relationships to them.
struct Linked {
    parts: Vec<Part>,
    relationships: Vec<Relationship>,
}

impl Linked {
    /// The part `word/{kind}.xml`, of the type `kind`.
    fn add(&mut self, kind: &'static str, text: String) {
        let target = format!("{kind}.xml");
        let id = format!("rId_{kind}");
        self.relationships
            .push(Relationship::new(&id, kind, &target));
        self.parts
            .push(Part::word(&format!("word/{target}"), kind, text));
    }

    /// The image files; their relationships come with the body.
    fn media(&mut self, media: Vec<Media>) {
        let parts = media.into_iter().map(|image| Part {
            name: format!("word/media/{}", image.name),
            bytes: image.bytes,
            content_type: Some(image.mime),
        });
        self.parts.extend(parts);
    }

    /// The parts, with `document.xml` and its relationships at the end.
    fn finish(mut self, document: String) -> Vec<Part> {
        self.parts
            .push(Part::word("word/document.xml", "document.main", document));
        let relationships = relationships_part("word/_rels/document.xml.rels", &self.relationships);
        self.parts.push(relationships);
        self.parts
    }
}

/// The page: A4 with the margins on all sides.
fn section() -> String {
    let (width, height) = PAGE;
    format!(
        concat!(
            r#"<w:sectPr><w:pgSz w:w="{0}" w:h="{1}"/>"#,
            r#"<w:pgMar w:top="{2}" w:right="{2}" w:bottom="{2}" w:left="{2}" "#,
            r#"w:header="709" w:footer="709" w:gutter="0"/></w:sectPr>"#,
        ),
        width, height, MARGIN
    )
}

/// The package's relationships: to the document, and to the core
/// properties when there are any.
fn package_relationships(has_core: bool) -> Part {
    let mut relationships = vec![Relationship::new(
        "rId1",
        "officeDocument",
        "word/document.xml",
    )];
    if has_core {
        relationships.push(Relationship::core_properties("rId2", core::CORE_PROPERTIES));
    }
    relationships_part("_rels/.rels", &relationships)
}

/// A WordprocessingML part whose root element `w:{root}` holds `content`.
fn xml_part(root: &str, content: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:{root} {NAMESPACES}>{content}</w:{root}>"#
    )
}

#[cfg(test)]
mod tests {
    use crate::model::{Block, DocumentModel, Inline};
    use crate::writers::docx::testing::*;
    use crate::writers::docx::write_docx;

    #[test]
    fn write_docx_produces_package_with_required_parts() {
        let docx = write_docx(&DocumentModel::default()).unwrap();

        let mut names = part_names(&docx);
        names.sort();
        assert_eq!(
            names,
            [
                "[Content_Types].xml",
                "_rels/.rels",
                "word/_rels/document.xml.rels",
                "word/document.xml",
                "word/styles.xml",
            ]
        );
    }

    #[test]
    fn write_docx_of_empty_model_reads_back_empty() {
        assert_eq!(read_back(Vec::new()), []);
    }

    #[test]
    fn write_docx_sets_a4_page_with_margins() {
        let xml = document_xml(Vec::new());

        // A4 is 11906 × 16838 twips; 2.5 cm is 1417 twips.
        assert!(
            xml.contains(r#"<w:pgSz w:w="11906" w:h="16838"/>"#),
            "{xml}"
        );
        assert!(
            xml.contains(r#"<w:pgMar w:top="1417" w:right="1417" w:bottom="1417" w:left="1417""#),
            "{xml}"
        );
    }

    #[test]
    fn write_docx_paragraphs_read_back() {
        let blocks = vec![paragraph("Primeiro."), paragraph("Segundo.")];

        assert_eq!(read_back(blocks.clone()), blocks);
    }

    #[test]
    fn write_docx_headings_read_back_with_levels() {
        let headings: Vec<Block> = (1..=6)
            .map(|level| Block::Heading {
                level,
                content: vec![text(&format!("Nível {level}"))],
            })
            .collect();

        assert_eq!(read_back(headings.clone()), headings);
    }

    #[test]
    fn write_docx_bold_and_italic_read_back() {
        let content = vec![
            text("Um "),
            Inline::Bold(vec![text("forte")]),
            text(" e um "),
            Inline::Italic(vec![text("leve")]),
            text("."),
        ];
        let blocks = vec![Block::Paragraph(content)];

        assert_eq!(read_back(blocks.clone()), blocks);
    }

    #[test]
    fn write_docx_line_break_reads_back() {
        let blocks = vec![Block::Paragraph(vec![
            text("Rua A, 10"),
            Inline::LineBreak,
            text("Centro"),
        ])];

        assert_eq!(read_back(blocks.clone()), blocks);
    }

    #[test]
    fn write_docx_external_link_reads_back() {
        let content = vec![
            text("Veja o "),
            link("https://example.org/a?b=1&c=2", "site"),
            text("."),
        ];
        let blocks = vec![Block::Paragraph(content)];

        assert_eq!(read_back(blocks.clone()), blocks);
    }

    #[test]
    fn write_docx_internal_link_reads_back_as_anchor() {
        let blocks = vec![Block::Paragraph(vec![
            text("Ver "),
            link("#conclusao", "Conclusão"),
        ])];

        assert!(document_xml(blocks.clone()).contains(r#"<w:hyperlink w:anchor="conclusao">"#));
        assert_eq!(read_back(blocks.clone()), blocks);
    }

    #[test]
    fn write_docx_writes_inline_code_in_monospace_font() {
        let content = vec![
            text("Rode "),
            Inline::Code("just check".to_string()),
            text("."),
        ];

        let xml = document_xml(vec![Block::Paragraph(content)]);

        let code_run = concat!(
            r#"<w:r><w:rPr><w:rFonts w:ascii="Courier New" w:hAnsi="Courier New" w:cs="Courier New"/></w:rPr>"#,
            r#"<w:t xml:space="preserve">just check</w:t></w:r>"#,
        );
        assert!(xml.contains(code_run), "{xml}");
    }

    #[test]
    fn write_docx_escapes_xml_characters_in_text() {
        let blocks = vec![paragraph(
            r#"Fé & razão: 1 < 2 > 0, "aspas" e 'apóstrofos'"#,
        )];

        assert_eq!(read_back(blocks.clone()), blocks);
    }

    /// One block of each kind the DOCX reader produces.
    /// A paragraph with every kind of inline the DOCX reader produces.
    fn rich_paragraph() -> Block {
        Block::Paragraph(vec![
            text("Texto "),
            Inline::Bold(vec![text("forte")]),
            text(", "),
            Inline::Italic(vec![text("leve")]),
            text(" e "),
            link("https://example.org", "link"),
            Inline::LineBreak,
            text("outra linha."),
        ])
    }

    /// A bulleted list with a numbered one inside its first item.
    fn nested_list() -> Block {
        let items = vec![
            vec![paragraph("Item"), list(true, &["Um", "Dois"])],
            vec![paragraph("Outro")],
        ];
        Block::List {
            ordered: false,
            items,
        }
    }

    fn every_block() -> Vec<Block> {
        let heading = Block::Heading {
            level: 2,
            content: vec![text("Título")],
        };
        let quote = Block::Quote(vec![paragraph("Citação.")]);
        let table = table(vec![row(&["Nome", "Nota"]), row(&["Ana", "9,5"])]);
        let image = image("image/png", png(64, 32), "Ícone");
        vec![
            heading,
            rich_paragraph(),
            nested_list(),
            quote,
            table,
            image,
            Block::PageBreak,
        ]
    }

    #[test]
    fn write_docx_document_reads_back_as_same_model() {
        let blocks = every_block();

        assert_eq!(read_back(blocks.clone()), blocks);
    }

    #[test]
    fn write_docx_styles_links_as_hyperlink() {
        let blocks = vec![Block::Paragraph(vec![link("https://example.org", "site")])];
        let docx = docx_of(blocks);

        let link_run = r#"<w:r><w:rPr><w:rStyle w:val="Hyperlink"/></w:rPr><w:t xml:space="preserve">site</w:t></w:r>"#;
        assert!(part(&docx, "word/document.xml").contains(link_run));
        let styles = part(&docx, "word/styles.xml");
        assert!(
            styles.contains(
                r#"<w:style w:type="character" w:styleId="Hyperlink"><w:name w:val="Hyperlink"/>"#
            ),
            "{styles}"
        );
    }
}
