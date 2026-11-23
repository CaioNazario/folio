//! Block nodes of comrak's tree, as model blocks.

use comrak::nodes::{AstNode, ListType, NodeCodeBlock, NodeList, NodeValue};

use super::inlines::{Images, inlines_of, paragraph};
use crate::model::{Block, Inline};

/// The model blocks for the children of `parent`.
pub fn blocks_of<'a>(parent: &'a AstNode<'a>, images: &Images) -> Vec<Block> {
    parent
        .children()
        .flat_map(|node| blocks(node, images))
        .collect()
}

/// The blocks for `node`: a paragraph may split around its images; none
/// for nodes the model leaves out.
fn blocks<'a>(node: &'a AstNode<'a>, images: &Images) -> Vec<Block> {
    let block = match &node.data().value {
        NodeValue::Paragraph => return paragraph(node, images),
        NodeValue::Heading(heading) => Block::Heading {
            level: heading.level,
            content: inlines_of(node, images),
        },
        NodeValue::List(list) => list_block(node, list, images),
        NodeValue::BlockQuote => Block::Quote(blocks_of(node, images)),
        NodeValue::CodeBlock(code) => code_block(code),
        NodeValue::Table(_) => Block::Table {
            rows: node.children().map(|row| cells(row, images)).collect(),
        },
        NodeValue::ThematicBreak => Block::Separator,
        _ => return Vec::new(),
    };
    vec![block]
}

/// Each item (plain or task) holds blocks, so lists nest.
fn list_block<'a>(node: &'a AstNode<'a>, list: &NodeList, images: &Images) -> Block {
    Block::List {
        ordered: list.list_type == ListType::Ordered,
        items: node
            .children()
            .map(|item| blocks_of(item, images))
            .collect(),
    }
}

fn code_block(code: &NodeCodeBlock) -> Block {
    Block::Code {
        language: language(&code.info),
        text: code.literal.clone(),
    }
}

/// The inlines of each cell of a table row.
fn cells<'a>(row: &'a AstNode<'a>, images: &Images) -> Vec<Vec<Inline>> {
    row.children()
        .map(|cell| inlines_of(cell, images))
        .collect()
}

/// The first word of a fenced block's info string (```` ```rust ````).
fn language(info: &str) -> Option<String> {
    info.split_whitespace().next().map(str::to_string)
}
