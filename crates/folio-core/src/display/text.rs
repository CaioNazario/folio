//! Text shown by the TXT and MD views, cut when the file is too large
//! (spec 07, "Arquivos grandes").

use std::io::{self, Read};

use crate::readers::{decode_text, decode_text_start};

/// MD and TXT files show at most this many bytes.
pub const DISPLAY_TEXT_LIMIT: usize = 10 * 1024 * 1024;

/// Decoded text for the stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayText {
    pub text: String,
    /// The file was larger than the limit and only its start is here.
    pub truncated: bool,
}

/// The text of `source`, reading at most `limit` bytes of it.
pub fn display_text(source: impl Read, limit: usize) -> io::Result<DisplayText> {
    // One byte past the limit tells whether the file goes on.
    let mut bytes = Vec::new();
    source.take(limit as u64 + 1).read_to_end(&mut bytes)?;
    let truncated = bytes.len() > limit;
    bytes.truncate(limit);
    let text = if truncated {
        cut_at_line_end(decode_text_start(&bytes))
    } else {
        decode_text(&bytes)
    };
    Ok(DisplayText { text, truncated })
}

/// `text` up to and including its last line break.
fn cut_at_line_end(mut text: String) -> String {
    if let Some(end) = text.rfind('\n') {
        text.truncate(end + 1);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_text_returns_whole_text_under_limit() {
        let shown = display_text("Linha 1\nLinha 2\n".as_bytes(), 100).unwrap();

        assert_eq!(
            shown,
            DisplayText {
                text: "Linha 1\nLinha 2\n".to_string(),
                truncated: false,
            }
        );
    }

    #[test]
    fn display_text_cuts_large_text_at_end_of_line_and_flags_it() {
        let shown = display_text("Linha 1\nLinha 2\nLinha 3\n".as_bytes(), 12).unwrap();

        assert_eq!(
            shown,
            DisplayText {
                text: "Linha 1\n".to_string(),
                truncated: true,
            }
        );
    }

    #[test]
    fn display_text_cut_utf8_does_not_fall_back_to_windows_1252() {
        // Each line is 7 bytes; byte 9 is the first half of the second "ç".
        let shown = display_text("ação\nação\n".as_bytes(), 9).unwrap();

        assert_eq!(shown.text, "ação\n");
    }

    #[test]
    fn display_text_of_single_line_longer_than_limit_cuts_at_char_boundary() {
        // Byte 4 is the first half of "ã": the cut keeps "aç" whole.
        let shown = display_text("ação sem quebra".as_bytes(), 4).unwrap();

        assert_eq!(
            shown,
            DisplayText {
                text: "aç".to_string(),
                truncated: true,
            }
        );
    }
}
