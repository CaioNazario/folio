//! The Markdown reader: comrak's tree translated into the document model,
//! and the YAML front matter into metadata (spec 07, "Leitores").

mod blocks;
mod front_matter;
mod inlines;
mod yaml_json;

use comrak::nodes::{AstNode, NodeValue};
use comrak::{Arena, Options, parse_document};
use front_matter::metadata_of;

use super::decode_text;
use crate::model::{DocumentModel, Metadata};

/// The document model of a Markdown file.
pub fn read_md(bytes: &[u8]) -> DocumentModel {
    let text = decode_text(bytes);
    let arena = Arena::new();
    let root = parse_document(&arena, &text, &options());
    DocumentModel {
        metadata: front_matter_of(root).map_or_else(Metadata::default, |yaml| metadata_of(&yaml)),
        blocks: blocks::blocks_of(root),
    }
}

/// The raw front matter block, when the document starts with one.
fn front_matter_of<'a>(root: &'a AstNode<'a>) -> Option<String> {
    let first = root.first_child()?;
    match &first.data().value {
        NodeValue::FrontMatter(raw) => Some(raw.clone()),
        _ => None,
    }
}

/// The GitHub extensions the model can represent, and YAML front matter
/// between `---` lines.
fn options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some("---".to_string());
    options.extension.tasklist = true;
    options.extension.table = true;
    options.extension.strikethrough = true;
    options
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Block, Inline};

    fn blocks(markdown: &str) -> Vec<Block> {
        read_md(markdown.as_bytes()).blocks
    }

    fn text(text: &str) -> Inline {
        Inline::Text(text.to_string())
    }

    #[test]
    fn read_md_maps_headings_with_levels() {
        assert_eq!(
            blocks("# Um\n\n### Três\n"),
            [
                Block::Heading {
                    level: 1,
                    content: vec![text("Um")]
                },
                Block::Heading {
                    level: 3,
                    content: vec![text("Três")]
                },
            ]
        );
    }

    #[test]
    fn read_md_maps_bold_italic_and_inline_code() {
        assert_eq!(
            blocks("Um **forte** e _leve_ com `código`.\n"),
            [Block::Paragraph(vec![
                text("Um "),
                Inline::Bold(vec![text("forte")]),
                text(" e "),
                Inline::Italic(vec![text("leve")]),
                text(" com "),
                Inline::Code("código".to_string()),
                text("."),
            ])]
        );
    }

    #[test]
    fn read_md_maps_links() {
        assert_eq!(
            blocks("Veja [as **notas**](../Notas.md).\n"),
            [Block::Paragraph(vec![
                text("Veja "),
                Inline::Link {
                    target: "../Notas.md".to_string(),
                    content: vec![text("as "), Inline::Bold(vec![text("notas")])],
                },
                text("."),
            ])]
        );
    }

    #[test]
    fn read_md_maps_soft_break_to_space_and_hard_break_to_line_break() {
        assert_eq!(
            blocks("um\ndois  \ntrês\n"),
            [Block::Paragraph(vec![
                text("um"),
                text(" "),
                text("dois"),
                Inline::LineBreak,
                text("três"),
            ])]
        );
    }

    fn para(content: &str) -> Block {
        Block::Paragraph(vec![text(content)])
    }

    #[test]
    fn read_md_maps_nested_ordered_and_unordered_lists() {
        assert_eq!(
            blocks("1. Um\n   - A\n   - B\n2. Dois\n"),
            [Block::List {
                ordered: true,
                items: vec![
                    vec![
                        para("Um"),
                        Block::List {
                            ordered: false,
                            items: vec![vec![para("A")], vec![para("B")]],
                        },
                    ],
                    vec![para("Dois")],
                ],
            }]
        );
    }

    #[test]
    fn read_md_maps_task_list_items_to_list_items() {
        assert_eq!(
            blocks("- [ ] fazer\n- [x] feito\n"),
            [Block::List {
                ordered: false,
                items: vec![vec![para("fazer")], vec![para("feito")]],
            }]
        );
    }

    #[test]
    fn read_md_maps_block_quote() {
        assert_eq!(
            blocks("> Citado\n>\n> Segundo\n"),
            [Block::Quote(vec![para("Citado"), para("Segundo")])]
        );
    }

    #[test]
    fn read_md_maps_code_block_with_language() {
        assert_eq!(
            blocks("```rust title=x\nfn main() {}\n```\n\n    indentado\n"),
            [
                Block::Code {
                    language: Some("rust".to_string()),
                    text: "fn main() {}\n".to_string(),
                },
                Block::Code {
                    language: None,
                    text: "indentado\n".to_string(),
                },
            ]
        );
    }

    #[test]
    fn read_md_maps_table() {
        assert_eq!(
            blocks("| Nome | Nota |\n|---|---:|\n| Ana | **10** |\n"),
            [Block::Table {
                rows: vec![
                    vec![vec![text("Nome")], vec![text("Nota")]],
                    vec![vec![text("Ana")], vec![Inline::Bold(vec![text("10")])]],
                ],
            }]
        );
    }

    #[test]
    fn read_md_maps_thematic_break_to_separator() {
        assert_eq!(
            blocks("Antes\n\n***\n\nDepois\n"),
            [para("Antes"), Block::Separator, para("Depois")]
        );
    }

    #[test]
    fn read_md_keeps_text_of_strikethrough() {
        assert_eq!(
            blocks("um ~~riscado~~ fim\n"),
            [Block::Paragraph(vec![
                text("um "),
                text("riscado"),
                text(" fim")
            ])]
        );
    }

    #[test]
    fn read_md_drops_html_tags_keeping_text() {
        assert_eq!(
            blocks("<div>\nbloco cru\n</div>\n\nUm <b>forte</b>.\n"),
            [Block::Paragraph(vec![
                text("Um "),
                text("forte"),
                text(".")
            ])]
        );
    }

    #[test]
    fn read_md_maps_image_to_alt_text_until_images_are_supported() {
        assert_eq!(
            blocks("Veja ![o gráfico](g.png) aqui\n"),
            [Block::Paragraph(vec![
                text("Veja "),
                text("o gráfico"),
                text(" aqui")
            ])]
        );
    }

    #[test]
    fn read_md_separates_yaml_front_matter_from_body() {
        assert_eq!(
            blocks("---\ntitle: Notas\n---\n\n# Corpo\n"),
            [Block::Heading {
                level: 1,
                content: vec![text("Corpo")]
            }]
        );
    }
}
