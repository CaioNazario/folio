//! The typography of a PDF: the face most glyphs of the body and of each
//! heading level are set in.

use std::collections::HashMap;
use std::rc::Rc;

use super::glyphs::Glyph;
use super::headings::Headings;
use super::lines::Line;
use crate::model::{Face, Typography};

/// The typography of the document whose pages have these `lines`, with
/// the levels `headings` gives each size.
pub fn typography_of(pages: &[Vec<Line>], headings: &Headings) -> Typography {
    let mut tallies: Vec<Tally> = Vec::new();
    for line in pages.iter().flatten() {
        let slot = headings.level(line.size()).map_or(0, usize::from);
        if tallies.len() <= slot {
            tallies.resize_with(slot + 1, Tally::default);
        }
        line.glyphs()
            .iter()
            .for_each(|glyph| tallies[slot].add(glyph));
    }
    let mut faces = tallies.into_iter().map(Tally::face);
    Typography {
        body: faces.next().unwrap_or_default(),
        headings: faces.collect(),
    }
}

/// How many glyphs of one kind of text are in each size, family and
/// emphasis.
#[derive(Default)]
struct Tally {
    sizes: HashMap<u16, usize>,
    families: HashMap<Rc<str>, usize>,
    bold: usize,
    italic: usize,
    glyphs: usize,
}

impl Tally {
    fn add(&mut self, glyph: &Glyph) {
        *self.sizes.entry(half_points(glyph.size)).or_default() += 1;
        if let Some(family) = &glyph.face.family {
            *self.families.entry(Rc::clone(family)).or_default() += 1;
        }
        self.bold += usize::from(glyph.face.style.bold);
        self.italic += usize::from(glyph.face.style.italic);
        self.glyphs += 1;
    }

    /// The size and family of most glyphs, bold or italic when most are.
    fn face(self) -> Face {
        Face {
            family: most_common(self.families).map(|family| family.to_string()),
            size: most_common(self.sizes),
            bold: self.bold * 2 > self.glyphs,
            italic: self.italic * 2 > self.glyphs,
        }
    }
}

/// The key with the largest count.
fn most_common<K>(counts: HashMap<K, usize>) -> Option<K> {
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(key, _)| key)
}

/// `size` in points as half-points, the unit of the model.
fn half_points(size: f64) -> u16 {
    (size * 2.0).round() as u16
}

#[cfg(test)]
mod tests {
    use super::super::read_pdf;
    use super::super::test_pdf::{Text, pdf, pdf_with_font_family, text};
    use crate::model::{Face, Typography};

    fn typography(pages: &[Vec<Text>]) -> Typography {
        read_pdf(&pdf(pages), None).unwrap().typography
    }

    fn helvetica(size: u16, bold: bool) -> Face {
        Face {
            family: Some("Helvetica".to_string()),
            size: Some(size),
            bold,
            italic: false,
        }
    }

    #[test]
    fn read_pdf_reports_body_font_and_size() {
        let page = vec![text("Corpo do texto.", 72.0, 700.0)];

        assert_eq!(typography(&[page]).body, helvetica(24, false));
    }

    fn document() -> Vec<Text> {
        vec![
            text("Capítulo", 72.0, 740.0).size(24.0).bold(),
            text("Seção", 72.0, 700.0).size(18.0),
            text("Corpo do texto mais longo que os títulos.", 72.0, 660.0),
        ]
    }

    #[test]
    fn read_pdf_reports_heading_sizes_by_level() {
        let sizes: Vec<_> = typography(&[document()])
            .headings
            .iter()
            .map(|h| h.size)
            .collect();

        assert_eq!(sizes, [Some(48), Some(36)]);
    }

    #[test]
    fn read_pdf_reports_bold_of_heading() {
        let headings = typography(&[document()]).headings;

        assert_eq!((headings[0].bold, headings[1].bold), (true, false));
    }

    #[test]
    fn read_pdf_prefers_font_descriptor_family() {
        let pages = [vec![text("Corpo.", 72.0, 700.0)]];
        let pdf = pdf_with_font_family(&pages, "Computer Modern");

        let body = read_pdf(&pdf, None).unwrap().typography.body;

        assert_eq!(body.family.as_deref(), Some("Computer Modern"));
    }
}
