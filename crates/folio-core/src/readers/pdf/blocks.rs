//! Lines grouped into the blocks of the document model.

use super::fonts::Style;
use super::glyphs::PageContent;
use super::headings::{Headings, key};
use super::images::PageImage;
use super::inlines::{Piece, inlines_of, push, unbold_whole};
use super::lines::{Line, lines_of};
use super::lists::{Item, Marker, Part, group, marker_of, strip_marker};
use crate::model::Block;
use std::iter::{self, Peekable};
use std::vec;

/// Baselines farther apart than this share of the font size (1.5 times
/// the usual leading of 1.2) start a new paragraph.
const PARAGRAPH_GAP: f64 = 1.8;

/// Lines that make one block, or one list item when the first line
/// starts with a marker.
struct Group {
    lines: Vec<Line>,
    marker: Option<Marker>,
}

impl Group {
    fn glyph_count(&self) -> usize {
        self.lines.iter().map(Line::glyph_count).sum()
    }
}

/// The blocks of the lines of each page, in order, with a page break
/// between pages.
pub fn blocks_of(pages: Vec<PageContent>) -> Vec<Block> {
    let (lines, images): (Vec<_>, Vec<_>) = pages
        .into_iter()
        .map(|page| (lines_of(page.glyphs), page.images))
        .unzip();
    let headings = Headings::of(&lines);
    let mut blocks = Vec::new();
    for (i, (lines, images)) in lines.into_iter().zip(images).enumerate() {
        if i > 0 {
            blocks.push(Block::PageBreak);
        }
        blocks.extend(page_blocks(lines, images, &headings));
    }
    blocks
}

fn image_block(image: PageImage) -> Block {
    Block::Image {
        bytes: image.png,
        mime: "image/png".to_string(),
        alt: String::new(),
    }
}

/// The blocks of the `lines` and `images` of one page, in content order:
/// an image comes after the block whose text the page drew before it.
fn page_blocks(lines: Vec<Line>, images: Vec<PageImage>, headings: &Headings) -> Vec<Block> {
    let mut images = images.into_iter().peekable();
    let mut parts: Vec<Part> = drawn_by(&mut images, 0).collect();
    let mut drawn = 0;
    for group in groups_of(lines) {
        drawn += group.glyph_count();
        parts.push(part_of(group, headings));
        parts.extend(drawn_by(&mut images, drawn));
    }
    group(parts)
}

/// The next of `images` that the page drew by its `glyphs`-th glyph.
fn drawn_by(
    images: &mut Peekable<vec::IntoIter<PageImage>>,
    glyphs: usize,
) -> impl Iterator<Item = Part> + '_ {
    iter::from_fn(move || images.next_if(|image| image.after <= glyphs))
        .map(|image| Part::Block(image_block(image)))
}

/// `lines` split where a block ends; a marker always starts a new one.
fn groups_of(lines: Vec<Line>) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    for line in lines {
        let marker = marker_at_start(&line);
        match groups.last_mut() {
            Some(group) if marker.is_none() && continues(&group.lines, &line) => {
                group.lines.push(line);
            }
            _ => groups.push(Group {
                lines: vec![line],
                marker,
            }),
        }
    }
    groups
}

fn marker_at_start(line: &Line) -> Option<Marker> {
    marker_of(&line.pieces().first()?.text)
}

/// Whether `line` goes on the block of `group`: same size, and close
/// below its last line.
fn continues(group: &[Line], line: &Line) -> bool {
    group.last().is_some_and(|last| {
        let gap = line.y() - last.y();
        key(line.size()) == key(last.size()) && gap > 0.0 && gap <= last.size() * PARAGRAPH_GAP
    })
}

/// The heading, paragraph or list item of `group`.
fn part_of(group: Group, headings: &Headings) -> Part {
    let pieces = joined(&group.lines);
    if let Some(marker) = group.marker {
        let item = Block::Paragraph(inlines_of(strip_marker(pieces)));
        return Part::Item(Item {
            ordered: marker.ordered,
            indent: group.lines[0].x(),
            blocks: vec![item],
        });
    }
    match headings.level(group.lines[0].size()) {
        Some(level) => Part::Block(Block::Heading {
            level,
            content: inlines_of(unbold_whole(pieces)),
        }),
        None => Part::Block(Block::Paragraph(inlines_of(pieces))),
    }
}

/// The pieces of `lines`, joined by a plain space, or by nothing after a
/// line that ends in a hyphen or already in a space.
fn joined(lines: &[Line]) -> Vec<Piece> {
    let mut pieces: Vec<Piece> = Vec::new();
    for line in lines {
        if pieces
            .last()
            .is_some_and(|last| !last.text.ends_with(['-', ' ']))
        {
            push(&mut pieces, Style::default(), " ");
        }
        for piece in line.pieces() {
            push(&mut pieces, piece.style, &piece.text);
        }
    }
    pieces
}
