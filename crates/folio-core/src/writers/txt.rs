//! The TXT writer: the model's text, without markup (spec 08, "Escritores").

use crate::model::{Block, DocumentModel, Inline};

/// What starts each line of a quote.
const QUOTE: &str = "> ";
/// The indentation of a code block's lines.
const CODE: &str = "    ";

/// The plain text of `model`, in UTF-8 with `\n` line endings.
pub fn write_txt(model: &DocumentModel) -> String {
    let text = blocks(&model.blocks);
    if text.is_empty() {
        return text;
    }
    text + "\n"
}

/// Blocks separated by a blank line.
fn blocks(blocks: &[Block]) -> String {
    let blocks: Vec<String> = blocks.iter().filter_map(block).collect();
    blocks.join("\n\n")
}

/// The block's text, or `None` for a block that has none in plain text.
fn block(block: &Block) -> Option<String> {
    let text = match block {
        Block::Heading { content, .. } | Block::Paragraph(content) => inlines(content),
        Block::List { ordered, items } => list(*ordered, items),
        Block::Quote(inner) => prefix_lines(&blocks(inner), QUOTE, QUOTE),
        Block::Code { text, .. } => prefix_lines(text.trim_end_matches('\n'), CODE, CODE),
        Block::Table { rows } => table(rows),
        Block::Image { alt, .. } if alt.is_empty() => return None,
        Block::Image { alt, .. } => format!("[{alt}]"),
        Block::Separator => "---".to_string(),
        Block::PageBreak => return None,
    };
    Some(text)
}

/// One item per line, without blank lines between them.
fn list(ordered: bool, items: &[Vec<Block>]) -> String {
    let items: Vec<String> = items
        .iter()
        .enumerate()
        .map(|(index, blocks)| item(&marker(ordered, index), blocks))
        .collect();
    items.join("\n")
}

/// `- `, or the item's number counted from 1.
fn marker(ordered: bool, index: usize) -> String {
    if ordered {
        format!("{}. ", index + 1)
    } else {
        "- ".to_string()
    }
}

/// The item's blocks after its marker; the lines after the first are
/// indented to where the item's text starts.
fn item(marker: &str, blocks: &[Block]) -> String {
    let blocks: Vec<String> = blocks.iter().filter_map(block).collect();
    let padding = " ".repeat(marker.chars().count());
    prefix_lines(&blocks.join("\n"), marker, &padding)
}

/// `text` with `first` before its first line and `rest` before each other
/// line. An empty line gets `rest` without its trailing spaces.
fn prefix_lines(text: &str, first: &str, rest: &str) -> String {
    let lines: Vec<String> = text
        .split('\n')
        .enumerate()
        .map(|(index, line)| match (index, line) {
            (0, _) => format!("{first}{line}"),
            (_, "") => rest.trim_end().to_string(),
            _ => format!("{rest}{line}"),
        })
        .collect();
    lines.join("\n")
}

/// One line per row, with the cells separated by tabs.
fn table(rows: &[Vec<Vec<Inline>>]) -> String {
    let rows: Vec<String> = rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|content| cell(content))
                .collect::<Vec<_>>()
                .join("\t")
        })
        .collect();
    rows.join("\n")
}

/// A cell's text on one line, so it stays in its row.
fn cell(content: &[Inline]) -> String {
    inlines(content).replace('\n', " ")
}

fn inlines(content: &[Inline]) -> String {
    content.iter().map(inline).collect()
}

fn inline(inline: &Inline) -> String {
    match inline {
        Inline::Text(text) | Inline::Code(text) => text.clone(),
        Inline::Bold(content) | Inline::Italic(content) => inlines(content),
        Inline::LineBreak => "\n".to_string(),
        Inline::Link { target, content } => link(target, content),
    }
}

/// The link's text, followed by its target when the text does not show it
/// and the target leads out of the document.
fn link(target: &str, content: &[Inline]) -> String {
    let text = inlines(content);
    if text == target || target.starts_with('#') {
        return text;
    }
    format!("{text} ({target})")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Metadata;

    fn text(text: &str) -> Inline {
        Inline::Text(text.to_string())
    }

    fn paragraph(content: &str) -> Block {
        Block::Paragraph(vec![text(content)])
    }

    fn write(blocks: Vec<Block>) -> String {
        write_txt(&DocumentModel {
            blocks,
            ..DocumentModel::default()
        })
    }

    #[test]
    fn write_txt_of_empty_model_is_empty() {
        assert_eq!(write_txt(&DocumentModel::default()), "");
    }

    #[test]
    fn write_txt_separates_blocks_with_blank_line_and_ends_with_newline() {
        let text = write(vec![paragraph("Primeiro."), paragraph("Segundo.")]);

        assert_eq!(text, "Primeiro.\n\nSegundo.\n");
    }

    #[test]
    fn write_txt_writes_heading_as_plain_line() {
        let heading = Block::Heading {
            level: 2,
            content: vec![text("Introdução")],
        };

        assert_eq!(
            write(vec![heading, paragraph("Texto.")]),
            "Introdução\n\nTexto.\n"
        );
    }

    #[test]
    fn write_txt_turns_line_break_into_newline() {
        let address = Block::Paragraph(vec![text("Rua A, 10"), Inline::LineBreak, text("Centro")]);

        assert_eq!(write(vec![address]), "Rua A, 10\nCentro\n");
    }

    #[test]
    fn write_txt_keeps_only_text_of_bold_italic_and_code() {
        let content = vec![
            Inline::Bold(vec![text("Use "), Inline::Italic(vec![text("sempre")])]),
            text(" o comando "),
            Inline::Code("just check".to_string()),
            text("."),
        ];

        assert_eq!(
            write(vec![Block::Paragraph(content)]),
            "Use sempre o comando just check.\n"
        );
    }

    fn link(target: &str, content: &str) -> Inline {
        Inline::Link {
            target: target.to_string(),
            content: vec![text(content)],
        }
    }

    #[test]
    fn write_txt_writes_link_target_in_parentheses_after_text() {
        let content = vec![
            text("Veja o "),
            link("https://example.org", "site"),
            text("."),
        ];

        assert_eq!(
            write(vec![Block::Paragraph(content)]),
            "Veja o site (https://example.org).\n"
        );
    }

    #[test]
    fn write_txt_writes_link_once_when_text_is_target() {
        let content = vec![link("https://example.org", "https://example.org")];

        assert_eq!(
            write(vec![Block::Paragraph(content)]),
            "https://example.org\n"
        );
    }

    #[test]
    fn write_txt_writes_internal_link_as_text_only() {
        let content = vec![text("Ver "), link("#conclusao", "Conclusão"), text(".")];

        assert_eq!(write(vec![Block::Paragraph(content)]), "Ver Conclusão.\n");
    }

    fn list(ordered: bool, items: Vec<Vec<Block>>) -> Block {
        Block::List { ordered, items }
    }

    #[test]
    fn write_txt_marks_unordered_items_with_dash() {
        let items = vec![vec![paragraph("Leite")], vec![paragraph("Pão")]];

        assert_eq!(write(vec![list(false, items)]), "- Leite\n- Pão\n");
    }

    #[test]
    fn write_txt_numbers_ordered_items() {
        let items = vec![
            vec![paragraph("Abrir")],
            vec![paragraph("Ler")],
            vec![paragraph("Fechar")],
        ];

        assert_eq!(
            write(vec![list(true, items)]),
            "1. Abrir\n2. Ler\n3. Fechar\n"
        );
    }

    #[test]
    fn write_txt_indents_nested_list_under_item_text() {
        let fruits = list(
            false,
            vec![vec![paragraph("Maçã")], vec![paragraph("Pera")]],
        );
        let items = vec![vec![paragraph("Frutas"), fruits], vec![paragraph("Pão")]];

        assert_eq!(
            write(vec![list(true, items)]),
            "1. Frutas\n   - Maçã\n   - Pera\n2. Pão\n"
        );
    }

    #[test]
    fn write_txt_indents_item_continuation_under_item_text() {
        let address = Block::Paragraph(vec![text("Rua A, 10"), Inline::LineBreak, text("Centro")]);
        let items = vec![vec![address, paragraph("Sala 2")], vec![paragraph("Outro")]];

        assert_eq!(
            write(vec![list(false, items)]),
            "- Rua A, 10\n  Centro\n  Sala 2\n- Outro\n"
        );
    }

    #[test]
    fn write_txt_prefixes_quote_lines() {
        let quote = Block::Quote(vec![paragraph("Primeiro."), paragraph("Segundo.")]);

        assert_eq!(write(vec![quote]), "> Primeiro.\n>\n> Segundo.\n");
    }

    #[test]
    fn write_txt_indents_code_lines_by_four_spaces() {
        let code = Block::Code {
            language: Some("rust".to_string()),
            text: "fn main() {\n\n    run();\n}\n".to_string(),
        };

        assert_eq!(
            write(vec![code]),
            "    fn main() {\n\n        run();\n    }\n"
        );
    }

    fn table(rows: Vec<Vec<Vec<Inline>>>) -> Block {
        Block::Table { rows }
    }

    #[test]
    fn write_txt_separates_table_cells_with_tabs() {
        let rows = vec![
            vec![vec![text("Nome")], vec![text("Nota")]],
            vec![vec![text("Ana")], vec![text("9,5")]],
        ];

        assert_eq!(write(vec![table(rows)]), "Nome\tNota\nAna\t9,5\n");
    }

    #[test]
    fn write_txt_replaces_line_break_in_table_cell_with_space() {
        let address = vec![text("Rua A"), Inline::LineBreak, text("Centro")];
        let rows = vec![vec![vec![text("Ana")], address]];

        assert_eq!(write(vec![table(rows)]), "Ana\tRua A Centro\n");
    }

    fn image(alt: &str) -> Block {
        Block::Image {
            bytes: vec![0x89, b'P', b'N', b'G'],
            mime: "image/png".to_string(),
            alt: alt.to_string(),
        }
    }

    #[test]
    fn write_txt_writes_image_alt_in_brackets() {
        assert_eq!(
            write(vec![image("Diagrama da arquitetura")]),
            "[Diagrama da arquitetura]\n"
        );
    }

    #[test]
    fn write_txt_drops_image_without_alt() {
        let blocks = vec![paragraph("Antes."), image(""), paragraph("Depois.")];

        assert_eq!(write(blocks), "Antes.\n\nDepois.\n");
    }

    #[test]
    fn write_txt_writes_separator_as_dashes() {
        let blocks = vec![paragraph("Antes."), Block::Separator, paragraph("Depois.")];

        assert_eq!(write(blocks), "Antes.\n\n---\n\nDepois.\n");
    }

    #[test]
    fn write_txt_drops_page_break_and_metadata() {
        let model = DocumentModel {
            metadata: Metadata {
                title: Some("Relatório".to_string()),
                author: Some("Ana".to_string()),
                ..Metadata::default()
            },
            blocks: vec![
                paragraph("Página 1."),
                Block::PageBreak,
                paragraph("Página 2."),
            ],
        };

        assert_eq!(write_txt(&model), "Página 1.\n\nPágina 2.\n");
    }

    #[test]
    fn write_txt_paragraphs_read_back_unchanged() {
        let address = Block::Paragraph(vec![text("Rua A, 10"), Inline::LineBreak, text("Centro")]);
        let blocks = vec![paragraph("Primeiro."), address, paragraph("Fim.")];

        let read = crate::readers::read_txt(write(blocks.clone()).as_bytes());

        assert_eq!(read.blocks, blocks);
    }
}
