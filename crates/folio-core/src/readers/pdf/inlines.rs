//! Styled text turned into the inlines of the document model.

use super::fonts::Style;
use crate::model::Inline;

/// Text in one style.
#[derive(Debug, Clone, PartialEq)]
pub struct Piece {
    pub style: Style,
    pub text: String,
}

/// Appends `text` in `style` to `pieces`, joining it to the last piece
/// when the style is the same.
pub fn push(pieces: &mut Vec<Piece>, style: Style, text: &str) {
    match pieces.last_mut() {
        Some(last) if last.style == style => last.text.push_str(text),
        _ => pieces.push(Piece {
            style,
            text: text.to_string(),
        }),
    }
}

/// `pieces` without bold when all their text is bold, as in a heading
/// set in a bold font; spaces do not count.
pub fn unbold_whole(pieces: Vec<Piece>) -> Vec<Piece> {
    let whole = pieces
        .iter()
        .all(|piece| piece.style.bold || piece.text.trim().is_empty());
    if !whole {
        return pieces;
    }
    let mut plain = Vec::new();
    for piece in pieces {
        let style = Style {
            bold: false,
            ..piece.style
        };
        push(&mut plain, style, &piece.text);
    }
    plain
}

/// The inlines of `pieces`, emphasized as their style says.
pub fn inlines_of(pieces: Vec<Piece>) -> Vec<Inline> {
    pieces.into_iter().map(inline_of).collect()
}

fn inline_of(piece: Piece) -> Inline {
    let mut inline = Inline::Text(piece.text);
    if piece.style.italic {
        inline = Inline::Italic(vec![inline]);
    }
    if piece.style.bold {
        inline = Inline::Bold(vec![inline]);
    }
    inline
}
