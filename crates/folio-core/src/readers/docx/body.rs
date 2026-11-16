//! The body of `document.xml`, as model blocks.

use super::context::Context;
use super::images::image_of;
use super::inlines::{Flow, flow_of, merge};
use super::lists::{self, Item, Piece};
use super::styles::StyleKind;
use super::tables::table_of;
use super::xml::Element;
use crate::model::{Block, Inline};

/// The blocks of the `w:body` element.
pub fn blocks_of(body: &Element, context: &Context) -> Vec<Block> {
    let pieces = body
        .elements()
        .filter_map(|element| piece(element, context));
    merge_quotes(lists::group(pieces.collect()))
}

/// Consecutive quotes as one: each quoted paragraph comes as a quote of
/// its own.
fn merge_quotes(blocks: Vec<Block>) -> Vec<Block> {
    let mut merged: Vec<Block> = Vec::new();
    for block in blocks {
        match (merged.last_mut(), block) {
            (Some(Block::Quote(last)), Block::Quote(more)) => last.extend(more),
            (_, block) => merged.push(block),
        }
    }
    merged
}

/// The piece for a paragraph or table; `None` for other elements.
fn piece(element: &Element, context: &Context) -> Option<Piece> {
    match element.name.as_str() {
        "p" => Some(paragraph_piece(element, context)),
        "tbl" => Some(Piece::Blocks(vec![table_of(
            element,
            &context.relationships,
        )])),
        _ => None,
    }
}

/// A numbered paragraph is a list item; any other, blocks of its own.
fn paragraph_piece(p: &Element, context: &Context) -> Piece {
    let blocks = paragraph(p, context);
    match lists::numbered(p, &context.numbering) {
        Some(numbered) => Piece::Item(Item { numbered, blocks }),
        None => Piece::Blocks(blocks),
    }
}

/// The blocks of `p`, inside a quote when its style is one.
fn paragraph(p: &Element, context: &Context) -> Vec<Block> {
    let kind = style_id(p).and_then(|id| context.styles.kind(id));
    let blocks = paragraph_blocks(p, context, kind);
    if kind == Some(StyleKind::Quote) && !blocks.is_empty() {
        return vec![Block::Quote(blocks)];
    }
    blocks
}

/// The blocks of `p` in order: its text (as a heading when its style is
/// one) split by page breaks and pictures. Empty stretches are left out:
/// Word uses empty paragraphs as spacing.
fn paragraph_blocks(p: &Element, context: &Context, kind: Option<StyleKind>) -> Vec<Block> {
    let level = match kind {
        Some(StyleKind::Heading(level)) => Some(level),
        _ => None,
    };
    let mut blocks = Vec::new();
    let mut stretch = Vec::new();
    for item in flow_of(p, &context.relationships) {
        let block = match item {
            Flow::Inline(inline) => {
                stretch.push(inline);
                continue;
            }
            Flow::PageBreak => Some(Block::PageBreak),
            Flow::Drawing(drawing) => image_of(drawing, &context.media),
        };
        end_stretch(&mut blocks, &mut stretch, level);
        blocks.extend(block);
    }
    end_stretch(&mut blocks, &mut stretch, level);
    blocks
}

/// Ends the text gathered in `stretch` as a block, unless it is empty.
fn end_stretch(blocks: &mut Vec<Block>, stretch: &mut Vec<Inline>, level: Option<u8>) {
    let content = merge(std::mem::take(stretch));
    if !content.is_empty() {
        blocks.push(text_block(level, content));
    }
}

/// A heading of `level`, or a paragraph without one.
fn text_block(level: Option<u8>, content: Vec<Inline>) -> Block {
    match level {
        Some(level) => Block::Heading { level, content },
        None => Block::Paragraph(content),
    }
}

/// The paragraph style id from `w:pPr/w:pStyle`.
fn style_id(p: &Element) -> Option<&str> {
    p.child("pPr")?.child("pStyle")?.attribute("val")
}
