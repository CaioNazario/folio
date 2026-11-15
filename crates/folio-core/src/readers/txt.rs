//! The TXT reader: decoded text, one paragraph per run of non-blank lines
//! (spec 07, "Leitores").

use super::decode_text;
use crate::model::{Block, DocumentModel, Inline};

/// The document model of a plain text file.
pub fn read_txt(bytes: &[u8]) -> DocumentModel {
    let text = decode_text(bytes);
    // `lines` also drops the `\r` of Windows line endings.
    let lines: Vec<&str> = text.lines().collect();
    let blocks = lines
        .split(|line| line.trim().is_empty())
        .filter(|group| !group.is_empty())
        .map(paragraph)
        .collect();
    DocumentModel {
        blocks,
        ..DocumentModel::default()
    }
}

/// Consecutive non-blank lines as one paragraph, joined by line breaks.
fn paragraph(lines: &[&str]) -> Block {
    let mut content = Vec::new();
    for line in lines {
        if !content.is_empty() {
            content.push(Inline::LineBreak);
        }
        content.push(Inline::Text(line.to_string()));
    }
    Block::Paragraph(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paragraph(text: &str) -> Block {
        Block::Paragraph(vec![Inline::Text(text.to_string())])
    }

    #[test]
    fn read_txt_splits_paragraphs_on_blank_lines() {
        let model = read_txt(b"Primeiro.\n\nSegundo.\n\n\nTerceiro.\n");

        assert_eq!(
            model.blocks,
            [
                paragraph("Primeiro."),
                paragraph("Segundo."),
                paragraph("Terceiro.")
            ]
        );
    }

    #[test]
    fn read_txt_keeps_single_line_breaks_inside_paragraph() {
        let model = read_txt(b"Rua A, 10\nCentro");

        assert_eq!(
            model.blocks,
            [Block::Paragraph(vec![
                Inline::Text("Rua A, 10".to_string()),
                Inline::LineBreak,
                Inline::Text("Centro".to_string()),
            ])]
        );
    }

    #[test]
    fn read_txt_treats_whitespace_only_lines_as_blank() {
        let model = read_txt(b"Um.\n  \t \nDois.\n \n");

        assert_eq!(model.blocks, [paragraph("Um."), paragraph("Dois.")]);
    }

    #[test]
    fn read_txt_normalizes_crlf() {
        let model = read_txt(b"Rua A\r\nCentro\r\n\r\nFim.\r\n");

        assert_eq!(
            model.blocks,
            [
                Block::Paragraph(vec![
                    Inline::Text("Rua A".to_string()),
                    Inline::LineBreak,
                    Inline::Text("Centro".to_string()),
                ]),
                paragraph("Fim."),
            ]
        );
    }

    #[test]
    fn read_txt_of_empty_file_has_no_blocks() {
        assert_eq!(read_txt(b"").blocks, []);
    }
}
