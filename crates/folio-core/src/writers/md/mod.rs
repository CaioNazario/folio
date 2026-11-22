//! The Markdown writer: the model as comrak's tree, printed by comrak, with
//! the metadata as YAML front matter (spec 08, "Escritores").

mod front_matter;
mod tree;

use comrak::{Arena, format_commonmark};

use crate::model::DocumentModel;
use crate::readers::markdown_options;
use front_matter::front_matter;
use tree::Tree;

/// A Markdown file and the images that go in its `.assets` folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MdDocument {
    pub text: String,
    pub images: Vec<MdImage>,
}

/// An image file of the `.assets` folder: its name there and its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MdImage {
    pub name: String,
    pub bytes: Vec<u8>,
}

/// The Markdown of `model`; its images are written to the folder `assets`,
/// next to the Markdown file.
pub fn write_md(model: &DocumentModel, assets: &str) -> MdDocument {
    let arena = Arena::new();
    let tree = Tree::new(&arena, assets);
    let root = tree.document(&model.blocks);
    let mut text = front_matter(&model.metadata);
    // Writing to a String never fails.
    _ = format_commonmark(root, &markdown_options(), &mut text);
    MdDocument {
        text: trim_quote_marks(&text),
        images: tree.into_images(),
    }
}

/// comrak leaves a space after the `>` of a quote's blank lines.
fn trim_quote_marks(text: &str) -> String {
    let lines = text.split_inclusive('\n').map(|line| {
        let content = line.trim_end();
        let only_marks = !content.is_empty() && content.chars().all(|c| c == '>' || c == ' ');
        if only_marks {
            format!("{content}\n")
        } else {
            line.to_string()
        }
    });
    lines.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Block, Inline};

    fn text(text: &str) -> Inline {
        Inline::Text(text.to_string())
    }

    fn markdown(blocks: Vec<Block>) -> String {
        let model = DocumentModel {
            blocks,
            ..DocumentModel::default()
        };
        write_md(&model, "Doc.assets").text
    }

    #[test]
    fn write_md_of_empty_model_is_empty() {
        let document = write_md(&DocumentModel::default(), "Vazio.assets");

        assert_eq!(
            document,
            MdDocument {
                text: String::new(),
                images: Vec::new()
            }
        );
    }

    fn heading(level: u8, content: &str) -> Block {
        Block::Heading {
            level,
            content: vec![text(content)],
        }
    }

    #[test]
    fn write_md_writes_headings_with_level_marks() {
        assert_eq!(
            markdown(vec![heading(1, "Um"), heading(3, "Três")]),
            "# Um\n\n### Três\n"
        );
    }

    fn paragraph(content: &str) -> Block {
        Block::Paragraph(vec![text(content)])
    }

    #[test]
    fn write_md_separates_paragraphs_with_blank_line() {
        assert_eq!(
            markdown(vec![paragraph("Primeiro."), paragraph("Segundo.")]),
            "Primeiro.\n\nSegundo.\n"
        );
    }

    #[test]
    fn write_md_writes_bold_italic_and_inline_code() {
        let content = vec![
            text("Um "),
            Inline::Bold(vec![text("forte")]),
            text(", um "),
            Inline::Italic(vec![text("leve")]),
            text(" e "),
            Inline::Code("código".to_string()),
            text("."),
        ];

        assert_eq!(
            markdown(vec![Block::Paragraph(content)]),
            "Um **forte**, um *leve* e `código`.\n"
        );
    }

    #[test]
    fn write_md_writes_links() {
        let site = Inline::Link {
            target: "https://example.org".to_string(),
            content: vec![text("site")],
        };
        let content = vec![text("Veja o "), site, text(".")];

        assert_eq!(
            markdown(vec![Block::Paragraph(content)]),
            "Veja o [site](https://example.org).\n"
        );
    }

    #[test]
    fn write_md_writes_line_break_as_hard_break() {
        let address = Block::Paragraph(vec![text("Rua A, 10"), Inline::LineBreak, text("Centro")]);

        assert_eq!(markdown(vec![address]), "Rua A, 10\\\nCentro\n");
    }

    #[test]
    fn write_md_escapes_markup_characters_in_text() {
        let literal = [
            "# não é título",
            "1. nem item",
            "*não* é _ênfase_, [nem](link), <b>tag</b> ou `código`.",
        ];
        let blocks = literal.iter().map(|text| paragraph(text)).collect();

        let read = crate::readers::read_md(markdown(blocks).as_bytes());

        let expected: Vec<Block> = literal.iter().map(|text| paragraph(text)).collect();
        assert_eq!(read.blocks, expected);
    }

    fn list(ordered: bool, items: Vec<Vec<Block>>) -> Block {
        Block::List { ordered, items }
    }

    #[test]
    fn write_md_writes_unordered_and_ordered_lists() {
        let shopping = list(
            false,
            vec![vec![paragraph("Leite")], vec![paragraph("Pão")]],
        );
        let steps = list(true, vec![vec![paragraph("Abrir")], vec![paragraph("Ler")]]);

        assert_eq!(
            markdown(vec![shopping, paragraph("Depois:"), steps]),
            "- Leite\n- Pão\n\nDepois:\n\n1. Abrir\n2. Ler\n"
        );
    }

    #[test]
    fn write_md_nests_lists() {
        let fruits = list(
            false,
            vec![vec![paragraph("Maçã")], vec![paragraph("Pera")]],
        );
        let items = vec![vec![paragraph("Frutas"), fruits], vec![paragraph("Pão")]];

        assert_eq!(
            markdown(vec![list(true, items)]),
            "1. Frutas\n   - Maçã\n   - Pera\n2. Pão\n"
        );
    }

    #[test]
    fn write_md_writes_quote() {
        let quote = Block::Quote(vec![paragraph("Primeiro."), paragraph("Segundo.")]);

        assert_eq!(markdown(vec![quote]), "> Primeiro.\n>\n> Segundo.\n");
    }

    #[test]
    fn write_md_writes_fenced_code_with_language() {
        let code = Block::Code {
            language: Some("rust".to_string()),
            text: "fn main() {\n    run();\n}\n".to_string(),
        };

        assert_eq!(
            markdown(vec![code]),
            "```rust\nfn main() {\n    run();\n}\n```\n"
        );
    }

    fn row(cells: &[&str]) -> Vec<Vec<Inline>> {
        cells.iter().map(|cell| vec![text(cell)]).collect()
    }

    #[test]
    fn write_md_writes_table_with_first_row_as_header() {
        let table = Block::Table {
            rows: vec![row(&["Nome", "Nota"]), row(&["Ana", "9,5"])],
        };

        assert_eq!(
            markdown(vec![table]),
            "| Nome | Nota |\n| --- | --- |\n| Ana | 9,5 |\n"
        );
    }

    #[test]
    fn write_md_pads_short_table_rows() {
        let table = Block::Table {
            rows: vec![row(&["Nome"]), row(&["Ana", "9,5", "Aprovada"])],
        };

        assert_eq!(
            markdown(vec![table]),
            "| Nome |  |  |\n| --- | --- | --- |\n| Ana | 9,5 | Aprovada |\n"
        );
    }

    #[test]
    fn write_md_writes_separator_as_thematic_break() {
        let blocks = vec![paragraph("Antes."), Block::Separator, paragraph("Depois.")];

        assert_eq!(markdown(blocks), "Antes.\n\n-----\n\nDepois.\n");
    }

    #[test]
    fn write_md_drops_page_break() {
        let blocks = vec![
            paragraph("Página 1."),
            Block::PageBreak,
            paragraph("Página 2."),
        ];

        assert_eq!(markdown(blocks), "Página 1.\n\nPágina 2.\n");
    }

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\n";

    fn image(mime: &str, bytes: &[u8], alt: &str) -> Block {
        Block::Image {
            bytes: bytes.to_vec(),
            mime: mime.to_string(),
            alt: alt.to_string(),
        }
    }

    fn write_blocks(blocks: Vec<Block>, assets: &str) -> MdDocument {
        let model = DocumentModel {
            blocks,
            ..DocumentModel::default()
        };
        write_md(&model, assets)
    }

    #[test]
    fn write_md_writes_image_into_assets_folder() {
        let document = write_blocks(vec![image("image/png", PNG, "Diagrama")], "Trabalho.assets");

        let expected = MdDocument {
            text: "![Diagrama](Trabalho.assets/imagem-1.png)\n".to_string(),
            images: vec![MdImage {
                name: "imagem-1.png".to_string(),
                bytes: PNG.to_vec(),
            }],
        };
        assert_eq!(document, expected);
    }

    fn names(document: &MdDocument) -> Vec<&str> {
        document
            .images
            .iter()
            .map(|image| image.name.as_str())
            .collect()
    }

    #[test]
    fn write_md_numbers_images_in_document_order() {
        let blocks = vec![
            image("image/png", PNG, "A"),
            Block::Quote(vec![image("image/png", PNG, "B")]),
            image("image/png", PNG, "C"),
        ];

        let document = write_blocks(blocks, "D.assets");

        assert_eq!(
            names(&document),
            ["imagem-1.png", "imagem-2.png", "imagem-3.png"]
        );
        assert_eq!(
            document.text,
            "![A](D.assets/imagem-1.png)\n\n> ![B](D.assets/imagem-2.png)\n\n![C](D.assets/imagem-3.png)\n"
        );
    }

    #[test]
    fn write_md_names_image_by_its_type() {
        let blocks = vec![
            image("image/jpeg", b"\xFF\xD8\xFF", "Foto"),
            image("image/svg+xml", b"<svg/>", "Ícone"),
            image("image/gif", b"GIF89a", "Animação"),
            image("image/webp", b"RIFF\0\0\0\0WEBP", "Figura"),
        ];

        let document = write_blocks(blocks, "D.assets");

        assert_eq!(
            names(&document),
            [
                "imagem-1.jpg",
                "imagem-2.svg",
                "imagem-3.gif",
                "imagem-4.webp"
            ]
        );
    }

    #[test]
    fn write_md_returns_no_images_when_model_has_none() {
        let document = write_blocks(vec![paragraph("Só texto."), Block::Separator], "D.assets");

        assert_eq!(document.images, []);
    }

    /// A paragraph with every kind of inline.
    fn rich_paragraph() -> Block {
        let link = Inline::Link {
            target: "https://example.org".to_string(),
            content: vec![text("link")],
        };
        Block::Paragraph(vec![
            text("Texto "),
            Inline::Bold(vec![text("forte")]),
            text(", "),
            Inline::Italic(vec![text("leve")]),
            text(", "),
            Inline::Code("código".to_string()),
            text(" e "),
            link,
            Inline::LineBreak,
            text("outra linha."),
        ])
    }

    /// A list whose item has two paragraphs and a sublist.
    fn loose_list() -> Block {
        let steps = list(true, vec![vec![paragraph("Um")], vec![paragraph("Dois")]]);
        list(
            false,
            vec![vec![
                paragraph("Item"),
                paragraph("Segundo parágrafo"),
                steps,
            ]],
        )
    }

    /// One block of each kind the Markdown reader produces.
    fn every_block() -> Vec<Block> {
        let code = Block::Code {
            language: Some("rust".to_string()),
            text: "fn main() {}\n".to_string(),
        };
        let table = Block::Table {
            rows: vec![row(&["Nome", "Nota"]), row(&["Ana", "9,5"])],
        };
        let quote = Block::Quote(vec![paragraph("Citação.")]);
        vec![
            heading(2, "Título"),
            rich_paragraph(),
            loose_list(),
            quote,
            code,
            table,
            Block::Separator,
        ]
    }

    #[test]
    fn write_md_document_reads_back_as_same_model() {
        let blocks = every_block();

        let read = crate::readers::read_md(markdown(blocks.clone()).as_bytes());

        assert_eq!(read.blocks, blocks);
    }
}
