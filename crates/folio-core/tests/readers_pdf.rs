//! The PDF reader against realistic documents (spec 07, "Critérios de
//! aceite"). No PDF file is versioned: krilla, the PDF library of Typst,
//! writes each one in the test with the Libertinus Serif fonts of
//! `typst-assets`, embedded and subset as real producers do.

use std::sync::Arc;

use folio_core::model::{Block, Inline};
use folio_core::readers::read_pdf;
use krilla::Document;
use krilla::geom::Point;
use krilla::page::PageSettings;
use krilla::text::{Font, TextDirection};

/// The Libertinus Serif styles, by their place in `typst_assets::fonts()`.
#[derive(Clone, Copy)]
enum Style {
    Regular = 0,
    Bold = 1,
    Italic = 2,
}

/// A line of text: style, size, left and baseline from the top, text.
type Line = (Style, f32, f32, f32, &'static str);

fn font(style: Style) -> Font {
    let bytes = typst_assets::fonts().nth(style as usize).unwrap();
    Font::new(Arc::new(bytes.to_vec()).into(), 0).unwrap()
}

/// An A4 PDF with one page that shows `lines` in order.
fn pdf(lines: &[Line]) -> Vec<u8> {
    let mut document = Document::new();
    let mut page = document.start_page_with(PageSettings::from_wh(595.0, 842.0).unwrap());
    let mut surface = page.surface();
    for &(style, size, x, y, text) in lines {
        let start = Point::from_xy(x, y);
        surface.draw_text(start, font(style), size, text, false, TextDirection::Auto);
    }
    surface.finish();
    page.finish();
    document.finish().unwrap()
}

fn text(text: &str) -> Inline {
    Inline::Text(text.to_string())
}

fn para(content: &str) -> Block {
    Block::Paragraph(vec![text(content)])
}

fn heading(level: u8, content: &str) -> Block {
    Block::Heading {
        level,
        content: vec![text(content)],
    }
}

fn list(ordered: bool, items: &[&str]) -> Block {
    Block::List {
        ordered,
        items: items.iter().map(|item| vec![para(item)]).collect(),
    }
}

const CONTRACT: &[Line] = &[
    (
        Style::Bold,
        20.0,
        72.0,
        80.0,
        "Contrato de Prestação de Serviços",
    ),
    (Style::Bold, 14.0, 72.0, 120.0, "Cláusula 1 · Do objeto"),
    (
        Style::Regular,
        11.0,
        72.0,
        140.0,
        "O CONTRATADO prestará serviços de",
    ),
    (Style::Italic, 11.0, 72.0, 154.0, "revisão técnica"),
    (Style::Regular, 11.0, 72.0, 168.0, "de documentos."),
    (
        Style::Bold,
        14.0,
        72.0,
        200.0,
        "Cláusula 2 · Das obrigações",
    ),
    (Style::Regular, 11.0, 72.0, 220.0, "1. Entregar no prazo."),
    (Style::Regular, 11.0, 72.0, 234.0, "2. Manter sigilo."),
    (Style::Bold, 14.0, 72.0, 270.0, "Cláusula 3 · Do pagamento"),
    (
        Style::Regular,
        11.0,
        72.0,
        290.0,
        "• Primeira parcela na assinatura.",
    ),
    (
        Style::Regular,
        11.0,
        72.0,
        304.0,
        "• Segunda parcela na entrega.",
    ),
];

/// What `CONTRACT` holds, in the document model.
fn contract_model() -> Vec<Block> {
    let object = Block::Paragraph(vec![
        text("O CONTRATADO prestará serviços de "),
        Inline::Italic(vec![text("revisão técnica")]),
        text(" de documentos."),
    ]);
    let installments = [
        "Primeira parcela na assinatura.",
        "Segunda parcela na entrega.",
    ];
    vec![
        heading(1, "Contrato de Prestação de Serviços"),
        heading(2, "Cláusula 1 · Do objeto"),
        object,
        heading(2, "Cláusula 2 · Das obrigações"),
        list(true, &["Entregar no prazo.", "Manter sigilo."]),
        heading(2, "Cláusula 3 · Do pagamento"),
        list(false, &installments),
    ]
}

#[test]
fn generated_contract_reads_headings_emphasis_and_lists() {
    let model = read_pdf(&pdf(CONTRACT), None).unwrap();

    assert_eq!(model.blocks, contract_model());
}

/// Two columns, drawn column by column, as text processors do.
const ARTICLE: &[Line] = &[
    (
        Style::Regular,
        10.0,
        50.0,
        100.0,
        "A primeira coluna começa",
    ),
    (Style::Regular, 10.0, 50.0, 113.0, "com este parágrafo."),
    (Style::Regular, 10.0, 50.0, 140.0, "Depois vem o segundo"),
    (Style::Regular, 10.0, 50.0, 153.0, "parágrafo da coluna."),
    (
        Style::Regular,
        10.0,
        310.0,
        100.0,
        "A segunda coluna continua",
    ),
    (
        Style::Regular,
        10.0,
        310.0,
        113.0,
        "o texto no alto da página.",
    ),
    (Style::Regular, 10.0, 310.0, 140.0, "E termina aqui."),
];

#[test]
fn generated_two_column_article_reads_column_after_column() {
    let model = read_pdf(&pdf(ARTICLE), None).unwrap();

    assert_eq!(
        model.blocks,
        [
            para("A primeira coluna começa com este parágrafo."),
            para("Depois vem o segundo parágrafo da coluna."),
            para("A segunda coluna continua o texto no alto da página."),
            para("E termina aqui."),
        ]
    );
}
