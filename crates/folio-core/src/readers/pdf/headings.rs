//! Headings told apart from the body by font size: the body size is the
//! most common one, and each larger size is a heading level.

use std::collections::BTreeMap;

use super::lines::Line;

/// A size at least this many times the body size is a heading.
const HEADING_SCALE: f64 = 1.15;
/// The deepest heading level of the model.
const MAX_LEVEL: usize = 6;

/// The heading sizes of a document, largest first.
pub struct Headings {
    sizes: Vec<i64>,
}

impl Headings {
    /// The heading sizes of the document whose pages have these lines.
    pub fn of(pages: &[Vec<Line>]) -> Self {
        let lines = || pages.iter().flatten();
        let Some(body) = body_size(lines()) else {
            return Self { sizes: Vec::new() };
        };
        let threshold = body as f64 * HEADING_SCALE;
        let mut sizes: Vec<i64> = lines()
            .map(|line| key(line.size()))
            .filter(|size| *size as f64 >= threshold)
            .collect();
        sizes.sort_unstable_by(|a, b| b.cmp(a));
        sizes.dedup();
        Self { sizes }
    }

    /// The heading level of text in `size`, or `None` for body text.
    /// Sizes past the sixth share the sixth level.
    pub fn level(&self, size: f64) -> Option<u8> {
        let index = self.sizes.iter().position(|s| *s == key(size))?;
        u8::try_from(index.min(MAX_LEVEL - 1) + 1).ok()
    }
}

/// `size` rounded to tenths of a point, so sizes compare as equal.
pub fn key(size: f64) -> i64 {
    (size * 10.0).round() as i64
}

/// The size most glyphs of `lines` have, as a [`key`].
fn body_size<'l>(lines: impl Iterator<Item = &'l Line>) -> Option<i64> {
    let mut glyphs: BTreeMap<i64, usize> = BTreeMap::new();
    for line in lines {
        *glyphs.entry(key(line.size())).or_default() += line.glyph_count();
    }
    glyphs
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(size, _)| size)
}
