//! Block nodes of comrak's tree, as model blocks.

use comrak::nodes::{AstNode, ListType, NodeCodeBlock, NodeList, NodeValue};

use super::inlines::inlines_of;
use crate::model::{Block, Inline};

/// The model blocks for the children of `parent`.
pub fn blocks_of<'a>(parent: &'a AstNode<'a>) -> Vec<Block> {
    parent.children().filter_map(block).collect()
}

/// The block for `node`, or `None` for nodes the model leaves out.
fn block<'a>(node: &'a AstNode<'a>) -> Option<Block> {
    let block = match &node.data().value {
        NodeValue::Heading(heading) => Block::Heading {
            level: heading.level,
            content: inlines_of(node),
        },
        NodeValue::Paragraph => Block::Paragraph(inlines_of(node)),
        NodeValue::List(list) => list_block(node, list),
        NodeValue::BlockQuote => Block::Quote(blocks_of(node)),
        NodeValue::CodeBlock(code) => code_block(code),
        NodeValue::Table(_) => Block::Table {
            rows: node.children().map(cells).collect(),
        },
        NodeValue::ThematicBreak => Block::Separator,
        _ => return None,
    };
    Some(block)
}

/// Each item (plain or task) holds blocks, so lists nest.
fn list_block<'a>(node: &'a AstNode<'a>, list: &NodeList) -> Block {
    Block::List {
        ordered: list.list_type == ListType::Ordered,
        items: node.children().map(blocks_of).collect(),
    }
}

fn code_block(code: &NodeCodeBlock) -> Block {
    Block::Code {
        language: language(&code.info),
        text: code.literal.clone(),
    }
}

/// The inlines of each cell of a table row.
fn cells<'a>(row: &'a AstNode<'a>) -> Vec<Vec<Inline>> {
    row.children().map(inlines_of).collect()
}

/// The first word of a fenced block's info string (```` ```rust ````).
fn language(info: &str) -> Option<String> {
    info.split_whitespace().next().map(str::to_string)
}
