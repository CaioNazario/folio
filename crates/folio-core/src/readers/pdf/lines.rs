//! Glyphs joined into lines of text, with a space where words stand apart.

use super::glyphs::Glyph;
use super::inlines::{Piece, push};

/// A gap wider than this share of the font size separates two words.
const WORD_GAP: f64 = 0.15;
/// A baseline that moves more than this share of the font size starts a
/// new line.
const LINE_SHIFT: f64 = 0.5;

/// Glyphs that share a baseline, in content order. Never empty.
#[derive(Debug, Clone)]
pub struct Line {
    glyphs: Vec<Glyph>,
}

impl Line {
    /// Where the line starts, from the left of the page.
    pub fn x(&self) -> f64 {
        self.glyphs[0].x
    }

    /// The baseline, from the top of the page.
    pub fn y(&self) -> f64 {
        self.glyphs[0].y
    }

    pub fn size(&self) -> f64 {
        self.glyphs[0].size
    }

    pub fn glyphs(&self) -> &[Glyph] {
        &self.glyphs
    }

    pub fn glyph_count(&self) -> usize {
        self.glyphs.len()
    }

    /// The text of the line in styled pieces, with a space wherever a
    /// gap separates words that have no space glyph between them.
    pub fn pieces(&self) -> Vec<Piece> {
        let mut pieces = Vec::new();
        for (i, glyph) in self.glyphs.iter().enumerate() {
            let gap = i > 0 && is_word_gap(&self.glyphs[i - 1], glyph);
            if gap && !pieces.last().is_some_and(|p: &Piece| p.text.ends_with(' ')) {
                push(&mut pieces, self.glyphs[i - 1].face.style, " ");
            }
            push(&mut pieces, glyph.face.style, &glyph.text);
        }
        pieces
    }
}

/// `glyphs` split into lines where the baseline moves.
pub fn lines_of(glyphs: Vec<Glyph>) -> Vec<Line> {
    let mut lines: Vec<Line> = Vec::new();
    for glyph in glyphs {
        match lines.last_mut() {
            Some(line) if !is_new_line(line.glyphs.last(), &glyph) => line.glyphs.push(glyph),
            _ => lines.push(Line {
                glyphs: vec![glyph],
            }),
        }
    }
    lines
}

fn is_new_line(previous: Option<&Glyph>, glyph: &Glyph) -> bool {
    previous.is_none_or(|p| (glyph.y - p.y).abs() > p.size * LINE_SHIFT)
}

fn is_word_gap(previous: &Glyph, glyph: &Glyph) -> bool {
    glyph.x - previous.end > previous.size * WORD_GAP && !glyph.text.starts_with(' ')
}
