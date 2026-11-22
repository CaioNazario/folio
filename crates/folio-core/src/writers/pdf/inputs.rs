//! The model as the template's `sys.inputs`: dictionaries with the `kind` of
//! each block and inline, and their fields.

use typst::foundations::{Array, Bytes, Dict, IntoValue, Value};

use crate::model::{Block, DocumentModel, Inline, Metadata};
use crate::readers::image_mime;
use crate::writers::image_size::image_size;

/// `sys.inputs` of the template.
pub fn inputs(model: &DocumentModel) -> Dict {
    let mut inputs = metadata(&model.metadata);
    inputs.insert("blocks".into(), blocks(&model.blocks));
    inputs
}

/// The fields of `set document`: the texts that are set, and the lists.
fn metadata(metadata: &Metadata) -> Dict {
    let mut dict = Dict::new();
    let texts = [("title", &metadata.title), ("subject", &metadata.subject)];
    for (key, value) in texts {
        if let Some(value) = value {
            dict.insert(key.into(), value.as_str().into_value());
        }
    }
    let authors: Array = metadata
        .author
        .iter()
        .map(|author| author.as_str().into_value())
        .collect();
    dict.insert("authors".into(), Value::Array(authors));
    let keywords: Array = metadata
        .keywords
        .iter()
        .map(|keyword| keyword.as_str().into_value())
        .collect();
    dict.insert("keywords".into(), Value::Array(keywords));
    dict
}

/// A node of the template: a dictionary with its `kind` and `fields`.
fn node(kind: &str, fields: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    let mut dict = Dict::new();
    dict.insert("kind".into(), kind.into_value());
    for (name, value) in fields {
        dict.insert(name.into(), value);
    }
    Value::Dict(dict)
}

fn blocks(blocks: &[Block]) -> Value {
    Value::Array(blocks.iter().filter_map(block).collect())
}

fn inlines(inlines: &[Inline]) -> Value {
    Value::Array(inlines.iter().map(inline).collect::<Array>())
}

fn block(block: &Block) -> Option<Value> {
    let value = match block {
        Block::Heading { level, content } => node(
            "heading",
            [
                ("level", i64::from(*level).into_value()),
                ("content", inlines(content)),
            ],
        ),
        Block::Paragraph(content) => node("paragraph", [("content", inlines(content))]),
        Block::List { ordered, items } => list(*ordered, items),
        Block::Quote(inner) => node("quote", [("blocks", blocks(inner))]),
        Block::Code { language, text } => code(language.as_deref(), text),
        Block::Table { rows } => table(rows),
        Block::Separator => node("separator", []),
        Block::PageBreak => node("pagebreak", []),
        Block::Image { bytes, alt, .. } => return image(bytes, alt),
    };
    Some(value)
}

fn list(ordered: bool, items: &[Vec<Block>]) -> Value {
    let items: Array = items.iter().map(|item| blocks(item)).collect();
    node(
        "list",
        [
            ("ordered", ordered.into_value()),
            ("items", Value::Array(items)),
        ],
    )
}

fn code(language: Option<&str>, text: &str) -> Value {
    let mut fields = vec![("text", text.into_value())];
    if let Some(language) = language {
        fields.push(("language", language.into_value()));
    }
    node("code", fields)
}

/// Rows padded to the widest one: Typst fills a table cell by cell, so a
/// short row would pull the next one into it.
fn table(rows: &[Vec<Vec<Inline>>]) -> Value {
    let columns = rows.iter().map(Vec::len).max().unwrap_or(0).max(1);
    let rows: Array = rows
        .iter()
        .map(|cells| padded_row(cells, columns))
        .collect();
    let columns = i64::try_from(columns).unwrap_or(1);
    node(
        "table",
        [
            ("columns", columns.into_value()),
            ("rows", Value::Array(rows)),
        ],
    )
}

fn padded_row(cells: &[Vec<Inline>], columns: usize) -> Value {
    let mut row: Array = cells.iter().map(|cell| inlines(cell)).collect();
    (cells.len()..columns).for_each(|_| row.push(Value::Array(Array::new())));
    Value::Array(row)
}

/// An image Typst can draw: SVG, or a raster image whose size Folio can
/// read. Anything else is its alternative text in brackets, as in TXT, and
/// nothing without one.
fn image(bytes: &[u8], alt: &str) -> Option<Value> {
    let drawable = image_mime(bytes) == Some("image/svg+xml") || image_size(bytes).is_some();
    if !drawable {
        if alt.is_empty() {
            return None;
        }
        let text = node("text", [("text", format!("[{alt}]").into_value())]);
        return Some(node(
            "paragraph",
            [("content", Value::Array(std::iter::once(text).collect()))],
        ));
    }
    let fields = [
        ("bytes", Value::Bytes(Bytes::new(bytes.to_vec()))),
        ("alt", alt.into_value()),
    ];
    Some(node("image", fields))
}

fn inline(inline: &Inline) -> Value {
    match inline {
        Inline::Text(text) => node("text", [("text", text.as_str().into_value())]),
        Inline::Bold(content) => node("bold", [("content", inlines(content))]),
        Inline::Italic(content) => node("italic", [("content", inlines(content))]),
        Inline::Code(text) => node("code", [("text", text.as_str().into_value())]),
        Inline::Link { target, content } => node(
            "link",
            [
                ("target", target.as_str().into_value()),
                ("content", inlines(content)),
            ],
        ),
        Inline::LineBreak => node("break", []),
    }
}

#[cfg(test)]
mod tests {
    use super::inputs;
    use crate::model::{Block, DocumentModel};
    use crate::writers::pdf::testing::*;
    use crate::writers::pdf::{drawn, test_png};

    #[test]
    fn write_pdf_unordered_list_reads_back() {
        let blocks = vec![paragraph("Compras:"), list(false, &["Leite", "Pão"])];

        assert_eq!(read_back(blocks.clone()), blocks);
    }

    #[test]
    fn write_pdf_ordered_list_reads_back() {
        let blocks = vec![
            paragraph("Passos:"),
            list(true, &["Abrir", "Ler", "Fechar"]),
        ];

        assert_eq!(read_back(blocks.clone()), blocks);
    }

    #[test]
    fn write_pdf_nested_list_reads_back() {
        let fruits = list(false, &["Maçã", "Pera"]);
        let items = vec![vec![paragraph("Frutas"), fruits], vec![paragraph("Pão")]];
        let blocks = vec![
            paragraph("Compras:"),
            Block::List {
                ordered: true,
                items,
            },
        ];

        assert_eq!(read_back(blocks.clone()), blocks);
    }

    #[test]
    fn write_pdf_writes_quote_text_in_order() {
        let quote = Block::Quote(vec![paragraph("Primeiro."), paragraph("Segundo.")]);

        let text = text_read_back(vec![paragraph("Antes."), quote, paragraph("Depois.")]);

        assert_eq!(text, "Antes. Primeiro. Segundo. Depois.");
    }

    #[test]
    fn write_pdf_writes_code_lines_in_order() {
        let code = Block::Code {
            language: Some("rust".to_string()),
            text: "fn main() {\n    run(\"#x\");\n}\n".to_string(),
        };

        let text = text_read_back(vec![paragraph("Antes."), code, paragraph("Depois.")]);

        assert_eq!(words(&text), r##"Antes. fn main() { run("#x"); } Depois."##);
    }

    /// The number of cells of each row of the first table in `sys.inputs`.
    fn input_row_lengths(model: &DocumentModel) -> Vec<usize> {
        use typst::foundations::{Array, Dict};
        let blocks = inputs(model)
            .get("blocks")
            .unwrap()
            .clone()
            .cast::<Array>()
            .unwrap();
        let table = blocks
            .iter()
            .next()
            .unwrap()
            .clone()
            .cast::<Dict>()
            .unwrap();
        let rows = table.get("rows").unwrap().clone().cast::<Array>().unwrap();
        rows.iter()
            .map(|row| row.clone().cast::<Array>().unwrap().len())
            .collect()
    }

    #[test]
    fn write_pdf_writes_table_cells_in_order() {
        let table = Block::Table {
            rows: vec![row(&["Nome", "Nota"]), row(&["Ana"]), row(&["Bia", "8,0"])],
        };
        let model = DocumentModel {
            blocks: vec![table.clone()],
            ..DocumentModel::default()
        };

        assert_eq!(input_row_lengths(&model), [2, 2, 2]);
        assert_eq!(words(&text_read_back(vec![table])), "Nome Nota Ana Bia 8,0");
    }

    #[test]
    fn write_pdf_separator_adds_no_text() {
        let without = vec![paragraph("Antes."), paragraph("Depois.")];
        let with = vec![paragraph("Antes."), Block::Separator, paragraph("Depois.")];

        assert_eq!(text_read_back(with.clone()), "Antes. Depois.");
        let lines = drawn::drawn(&pdf_of(with)).paths - drawn::drawn(&pdf_of(without)).paths;
        assert_eq!(lines, 1);
    }

    #[test]
    fn write_pdf_page_break_starts_new_page() {
        // A break with nothing before it would only make a blank page.
        let blocks = vec![
            Block::PageBreak,
            paragraph("Página 1."),
            Block::PageBreak,
            paragraph("Página 2."),
        ];

        let read = crate::readers::read_pdf(&pdf_of(blocks), None).unwrap();

        assert_eq!(read.metadata.pages, Some(2));
        assert_eq!(
            read.blocks,
            [
                paragraph("Página 1."),
                Block::PageBreak,
                paragraph("Página 2.")
            ]
        );
    }

    #[test]
    fn write_pdf_draws_raster_image() {
        let blocks = vec![
            paragraph("Antes."),
            image("image/png", test_png::png(64, 32), "Ícone"),
        ];

        assert_eq!(drawn::drawn(&pdf_of(blocks)).image_widths.len(), 1);
    }

    const SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20" fill="red"/></svg>"#;

    #[test]
    fn write_pdf_draws_svg_image() {
        // Typst turns an SVG into vector drawing, not into an image.
        let drawn = drawn::drawn(&pdf_of(vec![image(
            "image/svg+xml",
            SVG.to_vec(),
            "Retângulo",
        )]));

        assert!(drawn.paths >= 1, "{drawn:?}");
    }

    #[test]
    fn write_pdf_keeps_wide_image_within_text_width() {
        let wide = image("image/png", test_png::png(2000, 100), "Panorama");

        let widths = drawn::drawn(&pdf_of(vec![wide])).image_widths;

        // A4 is 595.28 pt wide; each 2.5 cm margin is 70.87 pt.
        let text_width = 595.28 - 2.0 * 70.87;
        assert_eq!(widths.len(), 1);
        assert!(widths[0] <= text_width + 0.5, "{widths:?}");
    }

    #[test]
    fn write_pdf_writes_unknown_image_as_alt_text() {
        let emf = image("image/x-emf", b"\x01\x00\x00\x00 EMF".to_vec(), "Logotipo");
        let cut_png = image("image/png", b"\x89PNG\r\n\x1a\n\0\0".to_vec(), "Foto");

        assert_eq!(text_read_back(vec![emf, cut_png]), "[Logotipo] [Foto]");
    }

    #[test]
    fn write_pdf_drops_unshown_image_without_alt() {
        let emf = image("image/x-emf", b"\x01\x00\x00\x00 EMF".to_vec(), "");

        assert_eq!(
            text_read_back(vec![paragraph("Antes."), emf, paragraph("Depois.")]),
            "Antes. Depois."
        );
    }
}
