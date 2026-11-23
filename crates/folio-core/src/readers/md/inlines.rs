//! Inline nodes of comrak's tree, as model inlines.

use std::cell::RefCell;

use comrak::nodes::{AstNode, NodeValue};

use super::ImageLookup;
use crate::model::{Block, Inline};

/// The image resolver of one reading, shared by the whole translation.
pub struct Images<'r>(RefCell<&'r mut dyn FnMut(&str) -> ImageLookup>);

impl<'r> Images<'r> {
    pub fn new(resolve: &'r mut dyn FnMut(&str) -> ImageLookup) -> Self {
        Self(RefCell::new(resolve))
    }

    fn lookup(&self, url: &str) -> ImageLookup {
        (self.0.borrow_mut())(url)
    }
}

/// The model inlines for the children of `parent`.
pub fn inlines_of<'a>(parent: &'a AstNode<'a>, images: &Images) -> Vec<Inline> {
    parent
        .children()
        .flat_map(|node| inline(node, images))
        .collect()
}

/// A paragraph's blocks: its inlines, split around each image found, which
/// becomes an image block of its own.
pub fn paragraph<'a>(node: &'a AstNode<'a>, images: &Images) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut run = Vec::new();
    for child in node.children() {
        match image_of(child, images) {
            Some(Ok(image)) => {
                flush(&mut run, &mut blocks);
                blocks.push(image);
            }
            Some(Err(inlines)) => run.extend(inlines),
            None => run.extend(inline(child, images)),
        }
    }
    flush(&mut run, &mut blocks);
    blocks
}

/// Ends the paragraph of `run`, unless it is only spaces.
fn flush(run: &mut Vec<Inline>, blocks: &mut Vec<Block>) {
    let inlines = std::mem::take(run);
    let blank = inlines
        .iter()
        .all(|inline| matches!(inline, Inline::Text(text) if text.trim().is_empty()));
    if !blank {
        blocks.push(Block::Paragraph(inlines));
    }
}

/// For an image node: the image block when it is found, or the inlines it
/// becomes otherwise. `None` for any other node.
fn image_of<'a>(node: &'a AstNode<'a>, images: &Images) -> Option<Result<Block, Vec<Inline>>> {
    let NodeValue::Image(link) = &node.data().value else {
        return None;
    };
    let alt = inlines_of(node, images);
    let outcome = match images.lookup(&link.url) {
        ImageLookup::Found { bytes, mime } => Ok(Block::Image {
            bytes,
            mime,
            alt: plain_text(&alt),
        }),
        ImageLookup::Remote => Err(vec![Inline::Link {
            target: link.url.clone(),
            content: alt,
        }]),
        ImageLookup::Unavailable => Err(alt),
    };
    Some(outcome)
}

/// The inlines for `node`: none for what the model leaves out, the content
/// alone for formatting it does not represent. An image not directly in a
/// paragraph keeps only its alt text, or the link of a web image.
fn inline<'a>(node: &'a AstNode<'a>, images: &Images) -> Vec<Inline> {
    let single = match &node.data().value {
        NodeValue::Text(text) => Inline::Text(text.to_string()),
        NodeValue::Strong => Inline::Bold(inlines_of(node, images)),
        NodeValue::Emph => Inline::Italic(inlines_of(node, images)),
        NodeValue::Code(code) => Inline::Code(code.literal.clone()),
        NodeValue::Link(link) => Inline::Link {
            target: link.url.clone(),
            content: inlines_of(node, images),
        },
        NodeValue::SoftBreak => Inline::Text(" ".to_string()),
        NodeValue::LineBreak => Inline::LineBreak,
        NodeValue::Image(_) => {
            return image_of(node, images).map_or_else(Vec::new, inline_fallback);
        }
        NodeValue::Strikethrough => return inlines_of(node, images),
        _ => return Vec::new(),
    };
    vec![single]
}

/// What an image inside other formatting becomes: an image found there has
/// nowhere to go but its alt text.
fn inline_fallback(outcome: Result<Block, Vec<Inline>>) -> Vec<Inline> {
    match outcome {
        Ok(Block::Image { alt, .. }) => vec![Inline::Text(alt)],
        Ok(_) => Vec::new(),
        Err(inlines) => inlines,
    }
}

/// The text of `inlines`, without formatting.
fn plain_text(inlines: &[Inline]) -> String {
    inlines
        .iter()
        .map(|inline| match inline {
            Inline::Text(text) | Inline::Code(text) => text.clone(),
            Inline::Bold(content) | Inline::Italic(content) => plain_text(content),
            Inline::Link { content, .. } => plain_text(content),
            Inline::LineBreak => " ".to_string(),
        })
        .collect()
}
