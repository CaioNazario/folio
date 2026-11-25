//! The PDF writer: the model goes to a fixed Typst template as data, and
//! Typst lays out the pages. The document's text is never Typst source
//! (spec 08, "Escritores").

#[cfg(test)]
pub(crate) mod drawn;
mod inputs;
#[cfg(test)]
mod test_png;
#[cfg(test)]
mod testing;

use typst::foundations::Datetime;
use typst_as_lib::TypstEngine;
use typst_layout::PagedDocument;
use typst_pdf::{PdfOptions, Timestamp};

use crate::model::DocumentModel;

const TEMPLATE: &str = include_str!("template.typ");

/// The PDF bytes of `model`; the error is Typst's diagnostic, for the log.
pub fn write_pdf(model: &DocumentModel) -> Result<Vec<u8>, String> {
    let engine = TypstEngine::builder()
        .main_file(TEMPLATE)
        .fonts(typst_assets::fonts())
        .build();
    let document: PagedDocument = engine
        .compile_with_input(inputs::inputs(model))
        .output
        .map_err(|error| format!("{error:?}"))?;
    let options = PdfOptions {
        timestamp: model.metadata.created_at.and_then(timestamp),
        ..PdfOptions::default()
    };
    typst_pdf::pdf(&document, &options).map_err(|error| format!("{error:?}"))
}

/// The creation date in UTC. Without one, the PDF has no date: Typst would
/// otherwise write none either, as the template leaves `document.date` to it.
fn timestamp(millis: i64) -> Option<Timestamp> {
    let utc = jiff::Timestamp::from_millisecond(millis)
        .ok()?
        .to_zoned(jiff::tz::TimeZone::UTC);
    let datetime = Datetime::from_ymd_hms(
        i32::from(utc.year()),
        u8::try_from(utc.month()).ok()?,
        u8::try_from(utc.day()).ok()?,
        u8::try_from(utc.hour()).ok()?,
        u8::try_from(utc.minute()).ok()?,
        u8::try_from(utc.second()).ok()?,
    )?;
    Some(Timestamp::new_utc(datetime))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Block, DocumentModel, Inline};
    use crate::writers::pdf::testing::*;

    #[test]
    fn write_pdf_produces_a_readable_pdf() {
        let pdf = write_pdf(&DocumentModel::default()).unwrap();

        assert!(pdf.starts_with(b"%PDF-"));
        assert!(crate::readers::read_pdf(&pdf, None).is_ok());
    }

    #[test]
    fn write_pdf_of_empty_model_has_one_blank_page() {
        let pdf = write_pdf(&DocumentModel::default()).unwrap();

        let read = crate::readers::read_pdf(&pdf, None).unwrap();
        assert_eq!((read.metadata.pages, read.blocks), (Some(1), Vec::new()));
    }

    #[test]
    fn write_pdf_paragraphs_read_back() {
        let blocks = vec![
            paragraph("Primeiro parágrafo."),
            paragraph("Segundo parágrafo."),
        ];

        assert_eq!(read_back(blocks.clone()), blocks);
    }

    #[test]
    fn write_pdf_does_not_hyphenate_long_paragraph() {
        let sentence = "Interdisciplinaridade, responsabilidade e internacionalização caracterizam \
                        extraordinariamente as universidades contemporâneas brasileiras.";
        let long = vec![paragraph(&[sentence; 12].join(" "))];

        assert_eq!(read_back(long.clone()), long);
    }

    #[test]
    fn write_pdf_headings_read_back_with_levels() {
        let mut blocks = Vec::new();
        for level in 1..=6 {
            let content = vec![text(&format!("Título {level}"))];
            blocks.push(Block::Heading { level, content });
            blocks.push(paragraph(&format!("Texto da seção {level}.")));
        }

        assert_eq!(read_back(blocks.clone()), blocks);
    }

    #[test]
    fn write_pdf_bold_and_italic_read_back() {
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
    fn write_pdf_keeps_markup_characters_literal() {
        let literal = vec![paragraph(
            r#"# título? $x$ *forte* _leve_ @rótulo <tag> [link] `raw` \n "aspas" #set"#,
        )];

        assert_eq!(read_back(literal.clone()), literal);
    }

    #[test]
    fn write_pdf_links_external_targets() {
        let blocks = vec![Block::Paragraph(vec![
            text("Veja o "),
            link("https://example.org/a", "site"),
            text("."),
        ])];

        let pdf = pdf_of(blocks);

        assert!(has_uri(&pdf, "https://example.org/a"));
        let read = crate::readers::read_pdf(&pdf, None).unwrap().blocks;
        assert_eq!(read, [paragraph("Veja o site.")]);
    }

    #[test]
    fn write_pdf_writes_internal_link_as_plain_text() {
        let pdf = pdf_of(vec![Block::Paragraph(vec![
            text("Ver "),
            link("#conclusao", "Conclusão"),
        ])]);

        assert!(!has_uri(&pdf, "#conclusao"));
        let read = crate::readers::read_pdf(&pdf, None).unwrap().blocks;
        assert_eq!(read, [paragraph("Ver Conclusão")]);
    }

    /// 2026-03-18T21:30:00Z.
    const CREATED: i64 = 1_773_869_400_000;

    /// The metadata the PDF reader finds in the PDF of a one-paragraph model
    /// with `metadata`.
    fn metadata_read_back(metadata: crate::model::Metadata) -> crate::model::Metadata {
        let model = DocumentModel {
            metadata,
            blocks: vec![paragraph("Texto.")],
            ..DocumentModel::default()
        };
        crate::readers::read_pdf_metadata(&write_pdf(&model).unwrap()).unwrap()
    }

    #[test]
    fn write_pdf_metadata_reads_back() {
        let metadata = crate::model::Metadata {
            title: Some("Relatório (final) & anexos".to_string()),
            author: Some("Ana".to_string()),
            subject: Some("Física".to_string()),
            keywords: vec!["ótica".to_string(), "luz".to_string()],
            created_at: Some(CREATED),
            ..crate::model::Metadata::default()
        };

        let read = metadata_read_back(metadata.clone());

        let fields =
            |m: crate::model::Metadata| (m.title, m.author, m.subject, m.keywords, m.created_at);
        assert_eq!(fields(read), fields(metadata));
    }

    #[test]
    fn write_pdf_has_no_creation_date_without_one() {
        let metadata = crate::model::Metadata {
            title: Some("Sem data".to_string()),
            ..crate::model::Metadata::default()
        };

        assert_eq!(metadata_read_back(metadata).created_at, None);
    }

    #[test]
    fn write_pdf_reports_typst_error_for_broken_image() {
        let broken = image("image/svg+xml", b"<svg width='10' <<".to_vec(), "Quebrada");
        let model = DocumentModel {
            blocks: vec![broken],
            ..DocumentModel::default()
        };

        let error = write_pdf(&model).unwrap_err();

        // Typst's own diagnostic, for the log.
        assert!(error.contains("unknown image format"), "{error}");
    }

    /// A paragraph with every kind of inline.
    fn rich_paragraph() -> Block {
        Block::Paragraph(vec![
            text("Texto "),
            Inline::Bold(vec![text("forte")]),
            text(", "),
            Inline::Italic(vec![text("leve")]),
            text(", "),
            Inline::Code("código".to_string()),
            text(" e "),
            link("https://example.org", "link"),
            Inline::LineBreak,
            text("outra linha."),
        ])
    }

    /// A bulleted list with a numbered one inside its item.
    fn nested_list() -> Block {
        let items = vec![vec![paragraph("Item"), list(true, &["Um", "Dois"])]];
        Block::List {
            ordered: false,
            items,
        }
    }

    /// A quote, a code block, a table and an image.
    fn other_blocks() -> Vec<Block> {
        let code = Block::Code {
            language: None,
            text: "x = 1\n".to_string(),
        };
        let table = Block::Table {
            rows: vec![row(&["Nome", "Nota"]), row(&["Ana", "9,5"])],
        };
        let picture = image("image/png", test_png::png(40, 20), "Ícone");
        vec![
            Block::Quote(vec![paragraph("Citação.")]),
            code,
            table,
            picture,
        ]
    }

    /// One block of each kind.
    fn every_block() -> Vec<Block> {
        let heading = Block::Heading {
            level: 1,
            content: vec![text("Relatório")],
        };
        let start = vec![heading, rich_paragraph(), nested_list()];
        let end = vec![Block::Separator, Block::PageBreak, paragraph("Fim.")];
        [start, other_blocks(), end].concat()
    }

    #[test]
    fn write_pdf_document_text_reads_back_in_order() {
        let text = text_read_back(every_block());

        let expected = "Relatório Texto forte, leve, código e link outra linha. Item Um Dois \
                        Citação. x = 1 Nome Nota Ana 9,5 Fim.";
        assert_eq!(words(&text), expected);
    }
}
