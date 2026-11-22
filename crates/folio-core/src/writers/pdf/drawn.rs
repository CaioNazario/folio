//! For the tests: what a PDF draws besides text, as the PDF reader's
//! interpreter sees it.

use hayro_interpret::font::GlyphRun;
use hayro_interpret::hayro_syntax::Pdf;
use hayro_interpret::kurbo::{Affine, BezPath, Rect};
use hayro_interpret::{
    BlendMode, ClipPath, Context, Device, DrawMode, DrawProps, Image, ImageDrawProps,
    InterpreterCache, InterpreterSettings, SoftMask, interpret_page,
};

/// The paths drawn (lines, borders, fills) and the width on the page, in
/// points, of each image drawn, over all pages.
#[derive(Debug, Default)]
pub struct Drawn {
    pub paths: usize,
    pub image_widths: Vec<f64>,
}

pub fn drawn(pdf: &[u8]) -> Drawn {
    let pdf = Pdf::new(pdf.to_vec()).unwrap();
    let cache = InterpreterCache::new();
    let mut drawn = Drawn::default();
    for page in pdf.pages().iter() {
        let (width, height) = page.render_dimensions();
        let area = Rect::new(0.0, 0.0, f64::from(width), f64::from(height));
        let settings = InterpreterSettings::default();
        let mut context = Context::new(Affine::IDENTITY, area, &cache, page.xref(), settings);
        interpret_page(page, &mut context, &mut drawn);
    }
    drawn
}

impl<'a> Device<'a> for Drawn {
    fn draw_path(&mut self, _: &BezPath, _: DrawProps<'a>, _: &DrawMode) {
        self.paths += 1;
    }

    /// The transform works in the image's pixels, so the width on the page
    /// is the pixel width scaled by the length of the transformed x axis.
    fn draw_image(&mut self, image: Image<'a, '_>, props: ImageDrawProps<'a>) {
        let [a, b, ..] = props.transform.as_coeffs();
        self.image_widths
            .push(f64::from(image.width()) * a.hypot(b));
    }

    fn draw_glyph_run(&mut self, _: &GlyphRun<'_, 'a>, _: DrawProps<'a>, _: &DrawMode) {}
    fn push_clip_path(&mut self, _: &ClipPath) {}
    fn push_transparency_group(&mut self, _: f32, _: Option<SoftMask<'a>>, _: BlendMode) {}
    fn pop_clip(&mut self) {}
    fn pop_transparency_group(&mut self) {}
}
