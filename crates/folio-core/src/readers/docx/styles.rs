//! Paragraph styles of `styles.xml`, recognized by their name: the id is
//! translated by Word (`Ttulo1` in Portuguese), the name is not.

use std::collections::HashMap;

use super::page_breaks::page_break_before;
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
    /// The `w:pageBreakBefore` each style sets, on or off.
    page_breaks: HashMap<String, bool>,
    /// The style each style is based on (`w:basedOn`).
    based_on: HashMap<String, String>,
}

impl Styles {
    /// The styles of the `w:styles` element; empty without the part.
    pub fn of(styles: Option<&Element>) -> Self {
        let all = styles.into_iter().flat_map(Element::elements);
        let styles: Vec<&Element> = all.filter(|element| element.name == "style").collect();
        Self {
            kinds: by_id(&styles, kind_of),
            page_breaks: by_id(&styles, |style| page_break_before(style.child("pPr")?)),
            based_on: by_id(&styles, |style| {
                Some(style.child("basedOn")?.attribute("val")?.to_string())
            }),
        }
    }

    /// What the paragraph style `id` is, if the reader recognizes it.
    pub fn kind(&self, id: &str) -> Option<StyleKind> {
        self.kinds.get(id).copied()
    }

    /// Whether paragraphs of the style `id` start on a new page.
    pub fn starts_page(&self, id: &str) -> bool {
        let mut id = id;
        // The depth bounds a `w:basedOn` cycle in a broken file.
        for _ in 0..MAX_BASED_ON_DEPTH {
            if let Some(set) = self.page_breaks.get(id) {
                return *set;
            }
            let Some(parent) = self.based_on.get(id) else {
                return false;
            };
            id = parent;
        }
        false
    }
}

/// What `value` finds in each of `styles`, by style id; a style without an
/// id or without a value is left out.
fn by_id<T>(styles: &[&Element], value: impl Fn(&Element) -> Option<T>) -> HashMap<String, T> {
    styles
        .iter()
        .filter_map(|style| Some((style.attribute("styleId")?.to_string(), value(style)?)))
        .collect()
}

/// How many `w:basedOn` links are followed from a style.
const MAX_BASED_ON_DEPTH: usize = 16;

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
