//! List items told apart by the marker that starts their first line, and
//! grouped into lists.

use super::inlines::Piece;
use crate::model::Block;

/// Glyphs that mark an item of an unordered list, before a space.
const BULLETS: [char; 6] = ['•', '◦', '▪', '–', '-', '*'];
/// The most digits an item number has.
const MAX_DIGITS: usize = 3;
/// An item indented more than this, in points, past the shallowest item
/// of its list nests under the item before it.
const INDENT_STEP: f64 = 2.0;

/// A block on its way to the model, or a list item that joins the list
/// around it.
pub enum Part {
    Block(Block),
    Item(Item),
}

/// A list item: whether it is numbered, how far its marker is indented,
/// and its content.
pub struct Item {
    pub ordered: bool,
    pub indent: f64,
    pub blocks: Vec<Block>,
}

/// The item marker at the start of a line.
#[derive(Debug, Clone, Copy)]
pub struct Marker {
    /// In bytes, with the spaces after the marker.
    pub len: usize,
    /// A number (`1.` or `1)`) rather than a bullet.
    pub ordered: bool,
}

/// The item marker that starts `text`: a bullet or a number, then a space.
pub fn marker_of(text: &str) -> Option<Marker> {
    let bullet = text.chars().next().filter(|c| BULLETS.contains(c));
    let (len, ordered) = match bullet {
        Some(bullet) => (bullet.len_utf8(), false),
        None => (number_len(text)?, true),
    };
    let rest = text[len..].strip_prefix(' ')?;
    let len = text.len() - rest.trim_start().len();
    Some(Marker { len, ordered })
}

/// The length of the item number that starts `text`, with its `.` or `)`.
fn number_len(text: &str) -> Option<usize> {
    let digits = text.chars().take_while(char::is_ascii_digit).count();
    let closed = matches!(text[digits..].chars().next(), Some('.' | ')'));
    ((1..=MAX_DIGITS).contains(&digits) && closed).then_some(digits + 1)
}

/// `pieces` without the item marker at their start.
pub fn strip_marker(mut pieces: Vec<Piece>) -> Vec<Piece> {
    let Some(first) = pieces.first_mut() else {
        return pieces;
    };
    if let Some(marker) = marker_of(&first.text) {
        first.text.replace_range(..marker.len, "");
    }
    if first.text.is_empty() {
        pieces.remove(0);
    }
    pieces
}

/// The blocks of `parts`, with consecutive items as one list.
pub fn group(parts: Vec<Part>) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut items = Vec::new();
    for part in parts {
        match part {
            Part::Item(item) => items.push(item),
            Part::Block(block) => {
                flush(&mut blocks, &mut items);
                blocks.push(block);
            }
        }
    }
    flush(&mut blocks, &mut items);
    blocks
}

/// Ends the list being gathered in `items`, if any.
fn flush(blocks: &mut Vec<Block>, items: &mut Vec<Item>) {
    if !items.is_empty() {
        blocks.push(list_of(std::mem::take(items)));
    }
}

/// The list of `items`, which is not empty. Its level is the shallowest
/// indent; deeper items nest under the item before them. The first item
/// says whether it is ordered.
fn list_of(items: Vec<Item>) -> Block {
    let base = items
        .iter()
        .map(|item| item.indent)
        .fold(f64::INFINITY, f64::min);
    let ordered = items.first().is_some_and(|item| item.ordered);
    let mut entries = Vec::new();
    let mut deeper = Vec::new();
    for item in items {
        if item.indent > base + INDENT_STEP {
            deeper.push(item);
            continue;
        }
        nest(&mut entries, &mut deeper);
        entries.push(item.blocks);
    }
    nest(&mut entries, &mut deeper);
    Block::List {
        ordered,
        items: entries,
    }
}

/// The `deeper` items as a list inside the last entry.
fn nest(entries: &mut Vec<Vec<Block>>, deeper: &mut Vec<Item>) {
    if deeper.is_empty() {
        return;
    }
    let nested = list_of(std::mem::take(deeper));
    match entries.last_mut() {
        Some(last) => last.push(nested),
        None => entries.push(vec![nested]),
    }
}
