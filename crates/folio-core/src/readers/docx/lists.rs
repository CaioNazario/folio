//! Numbered paragraphs (`w:numPr`) grouped into model lists, with the kind
//! of each level from `numbering.xml`.

use super::numbering::Numbering;
use super::xml::Element;
use crate::model::Block;

/// A body element on its way to the model: a block of its own, or a
/// numbered paragraph that joins the list around it.
pub enum Piece {
    Blocks(Vec<Block>),
    Item(Item),
}

pub struct Item {
    pub numbered: Numbered,
    pub blocks: Vec<Block>,
}

/// Where a numbered paragraph sits in its list.
#[derive(Debug, Clone, Copy)]
pub struct Numbered {
    pub level: u8,
    pub ordered: bool,
}

/// Where `p` sits in its list, or `None` when it is not numbered: numbered
/// is `w:numPr` with a `w:numId` other than 0, which turns numbering off.
pub fn numbered(p: &Element, numbering: &Numbering) -> Option<Numbered> {
    let num = p.child("pPr")?.child("numPr")?;
    let num_id = num
        .child("numId")?
        .attribute("val")
        .filter(|id| *id != "0")?;
    let level = num
        .child("ilvl")
        .and_then(|lvl| lvl.attribute("val")?.parse().ok())
        .unwrap_or(0);
    Some(Numbered {
        level,
        ordered: numbering.is_ordered(num_id, level),
    })
}

/// The blocks of `pieces`, with consecutive items as one list.
pub fn group(pieces: Vec<Piece>) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut items = Vec::new();
    for piece in pieces {
        match piece {
            Piece::Item(item) => items.push(item),
            Piece::Blocks(more) => {
                flush(&mut blocks, &mut items);
                blocks.extend(more);
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
/// one; deeper items nest under the item before them. The first item says
/// whether it is ordered.
fn list_of(items: Vec<Item>) -> Block {
    let base = items.iter().map(|item| item.numbered.level).min();
    let ordered = items.first().is_some_and(|item| item.numbered.ordered);
    let mut entries = Vec::new();
    let mut deeper = Vec::new();
    for item in items {
        if Some(item.numbered.level) > base {
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

#[cfg(test)]
mod tests {
    use super::super::read_docx;
    use super::super::test_docx::{docx_with, list_item, numbering};
    use crate::model::{Block, Inline};

    fn blocks(body: &str) -> Vec<Block> {
        let numbering = numbering();
        let parts = [("word/numbering.xml", numbering.as_bytes())];
        read_docx(&docx_with(body, &parts)).unwrap().blocks
    }

    fn item(text: &str) -> Vec<Block> {
        vec![Block::Paragraph(vec![Inline::Text(text.to_string())])]
    }

    #[test]
    fn read_docx_maps_bullet_numbering_to_unordered_list() {
        let body = list_item(1, 0, "Um") + &list_item(1, 0, "Dois");

        assert_eq!(
            blocks(&body),
            [Block::List {
                ordered: false,
                items: vec![item("Um"), item("Dois")],
            }]
        );
    }

    #[test]
    fn read_docx_maps_decimal_numbering_to_ordered_list() {
        let body = list_item(2, 0, "Um") + &list_item(2, 0, "Dois");

        assert_eq!(
            blocks(&body),
            [Block::List {
                ordered: true,
                items: vec![item("Um"), item("Dois")],
            }]
        );
    }

    #[test]
    fn read_docx_nests_list_items_by_level() {
        let body = [
            list_item(2, 0, "Um"),
            list_item(1, 1, "A"),
            list_item(1, 1, "B"),
            list_item(2, 0, "Dois"),
        ]
        .concat();
        let nested = Block::List {
            ordered: false,
            items: vec![item("A"), item("B")],
        };

        assert_eq!(
            blocks(&body),
            [Block::List {
                ordered: true,
                items: vec![[item("Um"), vec![nested]].concat(), item("Dois")],
            }]
        );
    }
}
