//! The glyphs of each page, as hayro draws them, in content order.

use hayro_interpret::font::{Glyph as Drawn, GlyphRun, PositionedGlyph};
use hayro_interpret::hayro_cmap::BfString;
use hayro_interpret::hayro_syntax::Pdf;
use hayro_interpret::hayro_syntax::page::Page;

use super::fonts::{Fonts, Style};
use super::images::{self, PageImage};
use hayro_font::FontCache;
use hayro_interpret::kurbo::{Affine, BezPath, Point, Rect};
use hayro_interpret::{
    BlendMode, ClipPath, Context, Device, DrawMode, DrawProps, Image, ImageDrawProps,
    InterpreterCache, InterpreterSettings, SoftMask, TransformExt, interpret_page,
};

/// A glyph with the text it stands for, placed on the page: `x` and `y`
/// are its origin on the baseline, from the top left; `end` is where the
/// next glyph would start; `size` is the font size; `style` comes from the
/// font dictionary.
#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub end: f64,
    pub size: f64,
    pub style: Style,
}

/// What one page draws: its glyphs and its images, in content order.
#[derive(Debug, Default)]
pub struct PageContent {
    pub glyphs: Vec<Glyph>,
    pub images: Vec<PageImage>,
}

/// The content of every page of `pdf`. After each page, `on_page(read,
/// total)` says whether to go on; `None` when it stopped.
pub fn pages(pdf: &Pdf, on_page: &mut dyn FnMut(usize, usize) -> bool) -> Option<Vec<PageContent>> {
    let cache = InterpreterCache::new();
    let fonts = FontCache::new();
    let settings = InterpreterSettings::default();
    let all = pdf.pages();
    let mut pages = Vec::new();
    for page in all.iter() {
        pages.push(content_of(page, &cache, &fonts, &settings));
        if !on_page(pages.len(), all.len()) {
            return None;
        }
    }
    Some(pages)
}

/// The glyphs and images hayro draws on one page.
fn content_of<'a>(
    page: &Page<'a>,
    cache: &InterpreterCache<'a>,
    fonts: &FontCache<'a>,
    settings: &InterpreterSettings,
) -> PageContent {
    let (width, height) = page.render_dimensions();
    let area = Rect::new(0.0, 0.0, f64::from(width), f64::from(height));
    let transform = page.initial_transform(true).to_kurbo();
    let mut context = Context::new(transform, area, cache, page.xref(), settings.clone());
    let mut collector = Collector {
        glyphs: Vec::new(),
        images: Vec::new(),
        fonts: Fonts::of(page.resources(), fonts, settings),
    };
    interpret_page(page, &mut context, &mut collector);
    PageContent {
        glyphs: collector.glyphs,
        images: collector.images,
    }
}

/// A device that keeps the glyphs and the raster images, and ignores
/// everything else.
struct Collector<'a> {
    glyphs: Vec<Glyph>,
    images: Vec<PageImage>,
    fonts: Fonts<'a>,
}

impl<'a> Device<'a> for Collector<'a> {
    fn draw_glyph_run(&mut self, run: &GlyphRun<'_, 'a>, props: DrawProps<'a>, _: &DrawMode) {
        let placed = run
            .glyphs()
            .iter()
            .filter_map(|glyph| placed(glyph, props.transform, &self.fonts));
        self.glyphs.extend(placed);
    }

    fn draw_path(&mut self, _: &BezPath, _: DrawProps<'a>, _: &DrawMode) {}
    fn push_clip_path(&mut self, _: &ClipPath) {}
    fn push_transparency_group(&mut self, _: f32, _: Option<SoftMask<'a>>, _: BlendMode) {}
    fn draw_image(&mut self, image: Image<'a, '_>, props: ImageDrawProps<'a>) {
        let Image::Raster(raster) = image else { return };
        // The transform maps the pixel grid of the image onto the page.
        let pixels = Rect::new(
            0.0,
            0.0,
            f64::from(raster.width()),
            f64::from(raster.height()),
        );
        let bounds = props.transform.transform_rect_bbox(pixels);
        let png = images::encode(&raster);
        let found = png.map(|png| PageImage {
            after: self.glyphs.len(),
            width: bounds.width(),
            height: bounds.height(),
            png,
        });
        self.images
            .extend(found.filter(|image| !image.is_ornament()));
    }
    fn pop_clip(&mut self) {}
    fn pop_transparency_group(&mut self) {}
}

/// `glyph` on the page through `transform`, or `None` when it has no text.
fn placed(glyph: &PositionedGlyph<'_>, transform: Affine, fonts: &Fonts<'_>) -> Option<Glyph> {
    let text = text_of(glyph.as_unicode()?);
    // Glyph space has 1000 units per em.
    let to_page = transform * glyph.transform();
    let origin = to_page * Point::ORIGIN;
    let [a, b, c, d, _, _] = to_page.as_coeffs();
    Some(Glyph {
        text,
        x: origin.x,
        y: origin.y,
        end: (to_page * Point::new(advance(glyph), 0.0)).x,
        size: (a * d - b * c).abs().sqrt() * 1000.0,
        style: match &**glyph {
            Drawn::Outline(outline) => fonts.style(outline),
            Drawn::Type3(_) => Style::default(),
        },
    })
}

/// How far `glyph` moves the pen, in glyph space.
fn advance(glyph: &Drawn<'_>) -> f64 {
    let width = match glyph {
        Drawn::Outline(outline) => outline.advance_width(),
        Drawn::Type3(type3) => type3.advance_width().map(|w| w * 1000.0),
    };
    f64::from(width.unwrap_or(500.0))
}

fn text_of(unicode: BfString) -> String {
    match unicode {
        BfString::Char(c) => c.to_string(),
        BfString::String(s) => s,
    }
}
