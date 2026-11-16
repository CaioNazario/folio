//! Paragraph styles of `styles.xml`, recognized by their name: the id is
//! translated by Word (`Ttulo1` in Portuguese), the name is not.

use std::collections::HashMap;

use super::xml::Element;

/// What a paragraph style makes of its paragraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleKind {
    /// `heading N`, N from 1 to 6.
    Heading(u8),
    /// `Quote` and `Intense Quote`.
    Quote,
}

/// The paragraph styles the reader recognizes, by style id.
#[derive(Debug, Default)]
pub struct Styles {
    kinds: HashMap<String, StyleKind>,
}

impl Styles {
    /// The styles of the `w:styles` element; empty without the part.
    pub fn of(styles: Option<&Element>) -> Self {
        let all = styles.into_iter().flat_map(Element::elements);
        let kinds = all
            .filter(|element| element.name == "style")
            .filter_map(|style| Some((style.attribute("styleId")?.to_string(), kind_of(style)?)))
            .collect();
        Self { kinds }
    }

    /// What the paragraph style `id` is, if the reader recognizes it.
    pub fn kind(&self, id: &str) -> Option<StyleKind> {
        self.kinds.get(id).copied()
    }
}

fn kind_of(style: &Element) -> Option<StyleKind> {
    let name = style.child("name")?.attribute("val")?.to_lowercase();
    if name == "quote" || name == "intense quote" {
        return Some(StyleKind::Quote);
    }
    let level = name.strip_prefix("heading ")?.parse().ok()?;
    (1..=6)
        .contains(&level)
        .then_some(StyleKind::Heading(level))
}
