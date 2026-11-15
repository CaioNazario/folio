//! Images of a Markdown document: each address is swapped for a marker
//! before sanitizing, and each marker for what the core decided after it, so
//! only `data:` URLs the core generated survive (spec 07).

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use comrak::nodes::{AstNode, NodeValue};

use super::markdown::{ImageOutcome, Unshown};

/// The scheme of the markers; the sanitizer is told to keep it in `src`.
pub const MARKER_SCHEME: &str = "folio-image";

/// An image found in the document.
pub struct MarkedImage {
    pub url: String,
    pub alt: String,
}

/// Swaps the address of every image under `root` for `folio-image:N`, N
/// being its position in the returned list.
pub fn mark_images<'a>(root: &'a AstNode<'a>) -> Vec<MarkedImage> {
    let mut images = Vec::new();
    for node in root.descendants() {
        let mut data = node.data_mut();
        let NodeValue::Image(link) = &mut data.value else {
            continue;
        };
        let url = std::mem::replace(&mut link.url, marker(images.len()));
        images.push(MarkedImage {
            url,
            alt: alt_text(node),
        });
    }
    images
}

fn marker(index: usize) -> String {
    format!("{MARKER_SCHEME}:{index}")
}

/// The plain text inside an image node: its alt text.
fn alt_text<'a>(node: &'a AstNode<'a>) -> String {
    node.descendants()
        .skip(1)
        .filter_map(|child| match &child.data().value {
            NodeValue::Text(text) => Some(text.to_string()),
            NodeValue::Code(code) => Some(code.literal.clone()),
            _ => None,
        })
        .collect()
}

/// `html` with each marked `<img>` replaced by what `outcomes` says, in the
/// order of `images`.
pub fn replace_markers(
    mut html: String,
    images: &[MarkedImage],
    outcomes: &[ImageOutcome],
) -> String {
    for (index, (image, outcome)) in images.iter().zip(outcomes).enumerate() {
        let Some((start, end)) = img_tag(&html, index) else {
            continue;
        };
        html.replace_range(start..end, &image_html(image, outcome));
    }
    html
}

/// Byte range of the `<img>` tag whose `src` is marker `index`.
fn img_tag(html: &str, index: usize) -> Option<(usize, usize)> {
    let src = html.find(&format!("src=\"{}\"", marker(index)))?;
    let start = html[..src].rfind("<img")?;
    Some((start, tag_end(html, src)?))
}

/// The byte after the `>` closing the tag that contains `from`, skipping
/// `>` inside quoted attribute values.
fn tag_end(html: &str, from: usize) -> Option<usize> {
    let mut quoted = false;
    for (offset, byte) in html.bytes().enumerate().skip(from) {
        match byte {
            b'"' => quoted = !quoted,
            b'>' if !quoted => return Some(offset + 1),
            _ => {}
        }
    }
    None
}

fn image_html(image: &MarkedImage, outcome: &ImageOutcome) -> String {
    let alt = escape(&image.alt);
    match outcome {
        ImageOutcome::Embedded { mime, bytes } => {
            format!(
                r#"<img src="data:{mime};base64,{}" alt="{alt}">"#,
                STANDARD.encode(bytes)
            )
        }
        ImageOutcome::Placeholder { reason, link } => placeholder(*reason, &alt, link.as_deref()),
    }
}

/// The alt text in a span the UI styles and labels by `data-folio-image`.
fn placeholder(reason: Unshown, alt: &str, link: Option<&str>) -> String {
    let link = link.map_or_else(String::new, |url| {
        let url = escape(url);
        format!(r#" <a href="{url}">{url}</a>"#)
    });
    format!(
        r#"<span data-folio-image="{}">{alt}{link}</span>"#,
        reason.as_str()
    )
}

/// `text` safe inside an HTML element or a double-quoted attribute.
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
