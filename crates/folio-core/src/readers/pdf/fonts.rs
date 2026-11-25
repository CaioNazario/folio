//! The style of each glyph, from the dictionary of its font: hayro gives
//! a glyph the identity of its font program, which is matched against the
//! fonts of the page resources (spec 07, spike S1).

use super::family::family_of;
use super::text_string;
use hayro_font::{Font, FontCache, FontGlyph, OutlineGlyph};
use hayro_interpret::InterpreterSettings;
use std::rc::Rc;

use hayro_interpret::hayro_syntax::object::{Dict, Name, Stream, String as PdfString};
use hayro_interpret::hayro_syntax::page::Resources;

/// A font weight from this one up is bold.
const BOLD_WEIGHT: f32 = 600.0;
/// Without a weight, a vertical stem this wide or wider is bold: 114 in
/// pdflatex's CMBX10 against 69 in CMR10, 139 in Times-Bold against 85.
const BOLD_STEM: f32 = 110.0;
/// Font descriptor flags (PDF 32000, 9.8.2).
const ITALIC_FLAG: u32 = 1 << 6;
const FORCE_BOLD_FLAG: u32 = 1 << 18;
const BOLD_NAMES: [&str; 5] = ["bold", "black", "heavy", "semibold", "demi"];
const ITALIC_NAMES: [&str; 2] = ["italic", "oblique"];

/// The emphasis of a glyph.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Style {
    pub bold: bool,
    pub italic: bool,
}

/// The family and emphasis of the font that drew a glyph; the family is
/// shared by every glyph of the font.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FontFace {
    pub family: Option<Rc<str>>,
    pub style: Style,
}

/// Form XObjects nested deeper than this are not searched for fonts,
/// which also stops a form that draws itself.
const MAX_FORM_DEPTH: usize = 8;

/// The fonts a page declares, in its resources or in those of the forms
/// it draws, each with its style.
pub struct Fonts<'a> {
    known: Vec<(Font<'a>, FontFace)>,
}

impl<'a> Fonts<'a> {
    /// The fonts of `resources`, loaded through `cache`.
    pub fn of(
        resources: &Resources<'a>,
        cache: &FontCache<'a>,
        settings: &InterpreterSettings,
    ) -> Self {
        let mut fonts = Self { known: Vec::new() };
        fonts.add(resources, &Loader { cache, settings }, 0);
        fonts
    }

    /// Adds the fonts of `resources` and of the forms in them.
    fn add(&mut self, resources: &Resources<'a>, loader: &Loader<'_, 'a>, depth: usize) {
        for name in resources.fonts.keys() {
            let dict = resources.fonts.get::<Dict<'a>>(&*name);
            if let Some(known) = dict.and_then(|dict| loader.load(&dict)) {
                self.known.push(known);
            }
        }
        if depth < MAX_FORM_DEPTH {
            for forms in form_resources(resources) {
                self.add(&forms, loader, depth + 1);
            }
        }
    }

    /// The face of the font that drew `glyph`; plain and without a family
    /// when no declared font has its program.
    pub fn face(&self, glyph: &OutlineGlyph) -> FontFace {
        let same_program = |font: &Font<'a>| match font.glyph(glyph.glyph_id(), glyph.char_code()) {
            FontGlyph::Outline(candidate) => candidate.identifier() == glyph.identifier(),
            FontGlyph::Type3(_) => false,
        };
        self.known
            .iter()
            .find(|(font, _)| same_program(font))
            .map(|(_, face)| face.clone())
            .unwrap_or_default()
    }
}

/// Loads fonts from their dictionaries.
struct Loader<'l, 'a> {
    cache: &'l FontCache<'a>,
    settings: &'l InterpreterSettings,
}

impl<'a> Loader<'_, 'a> {
    fn load(&self, dict: &Dict<'a>) -> Option<(Font<'a>, FontFace)> {
        let resolvers = (&self.settings.font_resolver, &self.settings.cmap_resolver);
        let font = self.cache.font(dict, resolvers.0, resolvers.1)?;
        let face = FontFace {
            family: family_of_font(dict).map(Rc::from),
            style: style_of(dict),
        };
        Some((font, face))
    }
}

/// The resources of the form XObjects in `resources`.
fn form_resources<'a>(resources: &Resources<'a>) -> Vec<Resources<'a>> {
    let x_objects = &resources.x_objects;
    x_objects
        .keys()
        .filter_map(|name| x_objects.get::<Stream<'a>>(&*name))
        .filter(|stream| {
            stream
                .dict()
                .get::<Name<'_>>(b"Subtype")
                .is_some_and(|s| s.as_str() == "Form")
        })
        .filter_map(|stream| stream.dict().get::<Dict<'a>>(b"Resources"))
        .map(Resources::new)
        .collect()
}

/// The family of a font: the descriptor's `FontFamily`, or the one its
/// `BaseFont` names.
fn family_of_font(font: &Dict<'_>) -> Option<String> {
    let descriptor = font.get::<Dict<'_>>(b"FontDescriptor").unwrap_or_default();
    if let Some(family) = descriptor.get::<PdfString<'_>>(b"FontFamily") {
        return Some(text_string::decode(family.as_bytes()));
    }
    family_of(font.get::<Name<'_>>(b"BaseFont")?.as_str())
}

/// What a font dictionary says about its weight and slant: the font
/// descriptor's weight, flags and angle, then the font name, then the
/// stem width.
fn style_of(font: &Dict<'_>) -> Style {
    let name = font
        .get::<Name<'_>>(b"BaseFont")
        .map(|name| name.as_str().to_lowercase())
        .unwrap_or_default();
    let named = |words: &[&str]| words.iter().any(|word| name.contains(word));
    let descriptor = Descriptor::of(font);
    Style {
        bold: descriptor.weight.map_or_else(
            || named(&BOLD_NAMES) || descriptor.stem.is_some_and(|s| s >= BOLD_STEM),
            |w| w >= BOLD_WEIGHT,
        ) || descriptor.flags & FORCE_BOLD_FLAG != 0,
        italic: descriptor.flags & ITALIC_FLAG != 0
            || descriptor.angle.is_some_and(|a| a != 0.0)
            || named(&ITALIC_NAMES),
    }
}

/// The entries of a font descriptor that tell the style.
struct Descriptor {
    flags: u32,
    weight: Option<f32>,
    angle: Option<f32>,
    stem: Option<f32>,
}

impl Descriptor {
    fn of(font: &Dict<'_>) -> Self {
        let descriptor = font.get::<Dict<'_>>(b"FontDescriptor").unwrap_or_default();
        Self {
            flags: descriptor.get::<u32>(b"Flags").unwrap_or(0),
            weight: descriptor.get::<f32>(b"FontWeight"),
            angle: descriptor.get::<f32>(b"ItalicAngle"),
            stem: descriptor.get::<f32>(b"StemV"),
        }
    }
}
