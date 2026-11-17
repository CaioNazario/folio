//! The PDF reader: the glyphs hayro draws on each page, turned into the
//! document model by deducing lines, paragraphs, headings and lists from
//! their position and font (spec 07, "Leitores").

mod blocks;
mod fonts;
mod glyphs;
mod headings;
mod inlines;
mod lines;
mod lists;
mod metadata;
mod open;
#[cfg(test)]
mod test_crypt;
#[cfg(test)]
pub(crate) mod test_pdf;
mod text_string;

pub use metadata::read_pdf_metadata;
pub use open::needs_password;

use super::ReadError;
use crate::model::DocumentModel;

/// The document model of a PDF file; `password` opens a protected one.
pub fn read_pdf(bytes: &[u8], password: Option<&str>) -> Result<DocumentModel, ReadError> {
    let pdf = open::open(bytes, password)?;
    let pages = glyphs::pages(&pdf)
        .into_iter()
        .map(lines::lines_of)
        .collect();
    Ok(DocumentModel {
        metadata: metadata::metadata_of(&pdf),
        blocks: blocks::blocks_of(pages),
    })
}

#[cfg(test)]
mod tests {
    use super::test_pdf::{
        pdf, pdf_with_form, pdf_with_image, pdf_with_info, pdf_with_latex_fonts, pdf_with_password,
        pdf_without_unicode, text,
    };
    use super::*;
    use crate::model::{Block, Inline};

    fn blocks(pages: &[Vec<test_pdf::Text>]) -> Vec<Block> {
        read_pdf(&pdf(pages), None).unwrap().blocks
    }

    fn para(text: &str) -> Block {
        Block::Paragraph(vec![Inline::Text(text.to_string())])
    }

    #[test]
    fn read_pdf_reads_single_line_as_paragraph() {
        let page = vec![text("Ata da reunião", 72.0, 700.0)];

        assert_eq!(blocks(&[page]), [para("Ata da reunião")]);
    }

    #[test]
    fn read_pdf_inserts_space_between_words_by_gap() {
        // Words placed apart, without a space glyph between them.
        let page = vec![text("Ata", 72.0, 700.0), text("final", 100.0, 700.0)];

        assert_eq!(blocks(&[page]), [para("Ata final")]);
    }

    #[test]
    fn read_pdf_joins_lines_of_paragraph_with_space() {
        let page = vec![
            text("Aos dez dias", 72.0, 700.0),
            text("do mês de março.", 72.0, 686.0),
        ];

        assert_eq!(blocks(&[page]), [para("Aos dez dias do mês de março.")]);
    }

    #[test]
    fn read_pdf_splits_paragraphs_by_vertical_gap() {
        let page = vec![
            text("Foi aprovada", 72.0, 700.0),
            text("a troca do portão.", 72.0, 686.0),
            text("Nada mais havendo", 72.0, 650.0),
            text("a tratar.", 72.0, 636.0),
        ];

        assert_eq!(
            blocks(&[page]),
            [
                para("Foi aprovada a troca do portão."),
                para("Nada mais havendo a tratar.")
            ]
        );
    }

    #[test]
    fn read_pdf_joins_word_hyphenated_across_lines() {
        let page = vec![
            text("Levou o guarda-", 72.0, 700.0),
            text("chuva ao sair.", 72.0, 686.0),
        ];

        assert_eq!(blocks(&[page]), [para("Levou o guarda-chuva ao sair.")]);
    }

    #[test]
    fn read_pdf_marks_bold_and_italic_from_font_dictionary() {
        let page = vec![
            text("O valor total é", 72.0, 700.0),
            text("R$ 4.500,00", 72.0, 686.0).bold(),
            text("em três parcelas.", 72.0, 672.0).italic(),
        ];

        assert_eq!(
            blocks(&[page]),
            [Block::Paragraph(vec![
                Inline::Text("O valor total é ".to_string()),
                Inline::Bold(vec![Inline::Text("R$ 4.500,00".to_string())]),
                Inline::Text(" ".to_string()),
                Inline::Italic(vec![Inline::Text("em três parcelas.".to_string())]),
            ])]
        );
    }

    #[test]
    fn read_pdf_finds_fonts_of_form_xobjects() {
        let model = read_pdf(&pdf_with_form("Cláusula primeira"), None).unwrap();

        assert_eq!(
            model.blocks,
            [Block::Paragraph(vec![Inline::Bold(vec![Inline::Text(
                "Cláusula primeira".to_string()
            )])])]
        );
    }

    fn heading(level: u8, text: &str) -> Block {
        Block::Heading {
            level,
            content: vec![Inline::Text(text.to_string())],
        }
    }

    #[test]
    fn read_pdf_maps_larger_sizes_to_heading_levels() {
        let page = vec![
            text("Contrato", 72.0, 740.0).size(18.0),
            text("Cláusula 1", 72.0, 710.0).size(14.0),
            text("O objeto é a revisão", 72.0, 690.0),
            text("dos documentos.", 72.0, 676.0),
            text("Cláusula 2", 72.0, 640.0).size(14.0),
            text("O prazo é de um mês.", 72.0, 620.0),
        ];

        assert_eq!(
            blocks(&[page]),
            [
                heading(1, "Contrato"),
                heading(2, "Cláusula 1"),
                para("O objeto é a revisão dos documentos."),
                heading(2, "Cláusula 2"),
                para("O prazo é de um mês."),
            ]
        );
    }

    fn list(ordered: bool, items: &[&str]) -> Block {
        Block::List {
            ordered,
            items: items.iter().map(|item| vec![para(item)]).collect(),
        }
    }

    #[test]
    fn read_pdf_recognizes_bullet_lines_as_unordered_list() {
        let page = vec![
            text("Parcelas:", 72.0, 714.0),
            text("• Primeira, na assinatura.", 72.0, 700.0),
            text("• Segunda, na entrega", 72.0, 686.0),
            text("parcial.", 82.0, 672.0),
        ];

        assert_eq!(
            blocks(&[page]),
            [
                para("Parcelas:"),
                list(
                    false,
                    &["Primeira, na assinatura.", "Segunda, na entrega parcial."]
                ),
            ]
        );
    }

    #[test]
    fn read_pdf_recognizes_numbered_lines_as_ordered_list() {
        let page = vec![
            text("1. Entregar no prazo.", 72.0, 700.0),
            text("2. Manter sigilo.", 72.0, 686.0),
            text("10) Avisar antes.", 72.0, 672.0),
        ];

        assert_eq!(
            blocks(&[page]),
            [list(
                true,
                &["Entregar no prazo.", "Manter sigilo.", "Avisar antes."]
            )]
        );
    }

    #[test]
    fn read_pdf_nests_list_items_by_indentation() {
        let page = vec![
            text("1. Pagamento", 72.0, 700.0),
            text("• Primeira parcela.", 90.0, 686.0),
            text("• Segunda parcela.", 90.0, 672.0),
            text("2. Prazo", 72.0, 658.0),
        ];
        let nested = list(false, &["Primeira parcela.", "Segunda parcela."]);

        assert_eq!(
            blocks(&[page]),
            [Block::List {
                ordered: true,
                items: vec![vec![para("Pagamento"), nested], vec![para("Prazo")]],
            }]
        );
    }

    #[test]
    fn read_pdf_separates_pages_with_page_break() {
        let pages = [
            vec![text("Página um.", 72.0, 700.0)],
            vec![text("Página dois.", 72.0, 700.0)],
        ];

        assert_eq!(
            blocks(&pages),
            [para("Página um."), Block::PageBreak, para("Página dois.")]
        );
    }

    #[test]
    fn read_pdf_skips_glyphs_without_unicode() {
        let model = read_pdf(&pdf_without_unicode("abba"), None).unwrap();

        assert_eq!(model.blocks, []);
    }

    #[test]
    fn read_pdf_of_page_without_text_is_empty() {
        let model = read_pdf(&pdf_with_image(), None).unwrap();

        assert_eq!(model.blocks, []);
    }

    #[test]
    fn read_pdf_fails_on_invalid_pdf() {
        let result = read_pdf(b"%PDF-1.4\nnada mais", None);

        assert!(matches!(result, Err(ReadError::Failed(_))));
    }

    fn protected() -> Vec<u8> {
        pdf_with_password(&[vec![text("Sigiloso.", 72.0, 700.0)]], "folio", "dono")
    }

    #[test]
    fn read_pdf_requires_password_for_protected_pdf() {
        let result = read_pdf(&protected(), None);

        assert_eq!(result, Err(ReadError::PasswordRequired));
    }

    #[test]
    fn read_pdf_rejects_wrong_password() {
        let result = read_pdf(&protected(), Some("errada"));

        assert_eq!(result, Err(ReadError::WrongPassword));
    }

    #[test]
    fn read_pdf_opens_protected_pdf_with_right_password() {
        let model = read_pdf(&protected(), Some("folio")).unwrap();

        assert_eq!(model.blocks, [para("Sigiloso.")]);
    }

    #[test]
    fn read_pdf_ignores_password_when_only_owner_password_is_set() {
        let pages = [vec![text("Aberto.", 72.0, 700.0)]];
        let owner_only = pdf_with_password(&pages, "", "dono");

        let model = read_pdf(&owner_only, Some("qualquer")).unwrap();

        assert_eq!(model.blocks, [para("Aberto.")]);
    }

    #[test]
    fn read_pdf_drops_bold_that_covers_whole_heading() {
        let page = vec![
            text("Cláusula 1", 72.0, 710.0).size(14.0).bold(),
            text("O objeto é a revisão.", 72.0, 690.0),
        ];

        assert_eq!(
            blocks(&[page]),
            [heading(1, "Cláusula 1"), para("O objeto é a revisão.")]
        );
    }

    #[test]
    fn read_pdf_reads_bold_of_embedded_fonts_from_stem_width() {
        let page = vec![
            text("Definição", 72.0, 700.0).bold(),
            text("Um conjunto é", 72.0, 686.0),
            text("uma coleção.", 72.0, 672.0).italic(),
        ];

        let model = read_pdf(&pdf_with_latex_fonts(&[page]), None).unwrap();

        assert_eq!(
            model.blocks,
            [Block::Paragraph(vec![
                Inline::Bold(vec![Inline::Text("Definição".to_string())]),
                Inline::Text(" Um conjunto é ".to_string()),
                Inline::Italic(vec![Inline::Text("uma coleção.".to_string())]),
            ])]
        );
    }

    #[test]
    fn read_pdf_carries_metadata() {
        let pages = [vec![text("Corpo.", 72.0, 700.0)]];

        let model = read_pdf(&pdf_with_info(&pages, "/Title (Ata)"), None).unwrap();

        assert_eq!(model.metadata.title.as_deref(), Some("Ata"));
    }
}
