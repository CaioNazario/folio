//! Raster images drawn on a page, encoded as PNG.

use hayro_interpret::{ImageData, LumaData, RasterImage};
use png::{BitDepth, ColorType, Encoder};

/// An image on a page: how many glyphs the page drew before it, its size
/// on the page and its PNG bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct PageImage {
    pub after: usize,
    pub width: f64,
    pub height: f64,
    pub png: Vec<u8>,
}

/// An image drawn smaller than this, in points on both sides, is a
/// bullet or an ornament, not content.
const MIN_SIDE: f64 = 24.0;

impl PageImage {
    /// Whether the image is too small to be content.
    pub fn is_ornament(&self) -> bool {
        self.width < MIN_SIDE && self.height < MIN_SIDE
    }
}

/// An image with more pixels than this is not decoded, to keep a huge
/// scan from exhausting memory.
const MAX_PIXELS: u64 = 25_000_000;

/// `image` as PNG, or `None` when it is too large or cannot be decoded.
pub fn encode(image: &RasterImage) -> Option<Vec<u8>> {
    if u64::from(image.width()) * u64::from(image.height()) > MAX_PIXELS {
        return None;
    }
    let mut png = None;
    image.with_rgba(|data, alpha| png = png_of(&data, alpha.as_ref()), None);
    png
}

fn png_of(data: &ImageData, alpha: Option<&LumaData>) -> Option<Vec<u8>> {
    let (color, pixels) = pixels_of(data, alpha);
    let mut out = Vec::new();
    let mut encoder = Encoder::new(&mut out, data.width(), data.height());
    encoder.set_color(color);
    encoder.set_depth(BitDepth::Eight);
    encoder
        .write_header()
        .ok()?
        .write_image_data(&pixels)
        .ok()?;
    Some(out)
}

/// The PNG color type and the interleaved samples of `data`, with the
/// `alpha` channel when it covers the same pixels.
fn pixels_of(data: &ImageData, alpha: Option<&LumaData>) -> (ColorType, Vec<u8>) {
    let (color, channels, samples) = match data {
        ImageData::Rgb(rgb) => (ColorType::Rgb, 3, &rgb.data),
        ImageData::Luma(luma) => (ColorType::Grayscale, 1, &luma.data),
    };
    let covering = alpha.filter(|a| (a.width, a.height) == (data.width(), data.height()));
    let Some(alpha) = covering else {
        return (color, samples.clone());
    };
    let with_alpha = match color {
        ColorType::Rgb => ColorType::Rgba,
        _ => ColorType::GrayscaleAlpha,
    };
    let pixels = samples.chunks(channels).zip(&alpha.data);
    (
        with_alpha,
        pixels
            .flat_map(|(px, a)| px.iter().chain([a]).copied())
            .collect(),
    )
}
