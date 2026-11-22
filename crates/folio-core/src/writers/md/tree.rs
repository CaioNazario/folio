//! The document model as comrak's tree, for comrak to print.

use std::cell::RefCell;

use comrak::Arena;
use comrak::nodes::{
    AstNode, ListType, NodeCode, NodeCodeBlock, NodeHeading, NodeLink, NodeList, NodeTable,
    NodeValue, TableAlignment,
};

use super::MdImage;
use crate::model::{Block, Inline};
use crate::writers::image_extension;

type Node<'a> = &'a AstNode<'a>;

/// Builds the tree's nodes in one arena, and collects the images that go
/// in the folder `assets`.
pub struct Tree<'a> {
    arena: &'a Arena<'a>,
    assets: &'a str,
    images: RefCell<Vec<MdImage>>,
}

impl<'a> Tree<'a> {
    pub fn new(arena: &'a Arena<'a>, assets: &'a str) -> Self {
        Self {
            arena,
            assets,
            images: RefCell::new(Vec::new()),
        }
    }

    /// The images the tree points to, in document order.
    pub fn into_images(self) -> Vec<MdImage> {
        self.images.into_inner()
    }

    /// The document node holding `blocks`.
    pub fn document(&self, blocks: &[Block]) -> Node<'a> {
        let root = self.node(NodeValue::Document);
        self.append_blocks(root, blocks);
        root
    }

    fn append_blocks(&self, parent: Node<'a>, blocks: &[Block]) {
        for block in blocks.iter().filter_map(|block| self.block(block)) {
            parent.append(block);
        }
    }

    fn node(&self, value: NodeValue) -> Node<'a> {
        self.arena.alloc(value.into())
    }

    fn block(&self, block: &Block) -> Option<Node<'a>> {
        let node = match block {
            Block::Heading { level, content } => self.with_inlines(heading(*level), content),
            Block::Paragraph(content) => self.with_inlines(NodeValue::Paragraph, content),
            Block::List { ordered, items } => self.list(*ordered, items),
            Block::Quote(inner) => self.with_blocks(NodeValue::BlockQuote, inner),
            Block::Code { language, text } => self.node(code_block(language.as_deref(), text)),
            Block::Table { rows } => self.table(rows),
            Block::Separator => self.node(NodeValue::ThematicBreak),
            Block::Image { bytes, mime, alt } => self.image(bytes, mime, alt),
            Block::PageBreak => return None,
        };
        Some(node)
    }

    /// A paragraph with the image alone, pointing to its file in `assets`.
    fn image(&self, bytes: &[u8], mime: &str, alt: &str) -> Node<'a> {
        let mut images = self.images.borrow_mut();
        let name = format!("imagem-{}.{}", images.len() + 1, image_extension(mime));
        let url = format!("{}/{name}", self.assets);
        images.push(MdImage {
            name,
            bytes: bytes.to_vec(),
        });
        let image = self.node(NodeValue::Image(Box::new(NodeLink {
            url,
            title: String::new(),
        })));
        image.append(self.node(NodeValue::Text(alt.to_string().into())));
        let paragraph = self.node(NodeValue::Paragraph);
        paragraph.append(image);
        paragraph
    }

    /// A node of `value` holding `blocks`.
    fn with_blocks(&self, value: NodeValue, blocks: &[Block]) -> Node<'a> {
        let node = self.node(value);
        self.append_blocks(node, blocks);
        node
    }

    fn list(&self, ordered: bool, items: &[Vec<Block>]) -> Node<'a> {
        let list = list_node(ordered, is_tight(items));
        let node = self.node(NodeValue::List(list));
        for blocks in items {
            node.append(self.with_blocks(NodeValue::Item(list), blocks));
        }
        node
    }

    /// The first row is the header. Every row gets as many cells as the
    /// widest one: a GitHub table has the same columns in all rows.
    fn table(&self, rows: &[Vec<Vec<Inline>>]) -> Node<'a> {
        let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
        let node = self.node(table_node(columns, rows.len()));
        for (index, cells) in rows.iter().enumerate() {
            node.append(self.row(index == 0, cells, columns));
        }
        node
    }

    fn row(&self, header: bool, cells: &[Vec<Inline>], columns: usize) -> Node<'a> {
        let row = self.node(NodeValue::TableRow(header));
        for content in cells {
            row.append(self.with_inlines(NodeValue::TableCell, content));
        }
        for _ in cells.len()..columns {
            row.append(self.node(NodeValue::TableCell));
        }
        row
    }

    /// A node of `value` holding `content`.
    fn with_inlines(&self, value: NodeValue, content: &[Inline]) -> Node<'a> {
        let node = self.node(value);
        for inline in content {
            node.append(self.inline(inline));
        }
        node
    }

    fn inline(&self, inline: &Inline) -> Node<'a> {
        match inline {
            Inline::Text(text) => self.node(NodeValue::Text(text.clone().into())),
            Inline::Bold(content) => self.with_inlines(NodeValue::Strong, content),
            Inline::Italic(content) => self.with_inlines(NodeValue::Emph, content),
            Inline::Code(text) => self.node(code(text)),
            Inline::Link { target, content } => self.with_inlines(link(target), content),
            Inline::LineBreak => self.node(NodeValue::LineBreak),
        }
    }
}

fn heading(level: u8) -> NodeValue {
    NodeValue::Heading(NodeHeading {
        level,
        setext: false,
        closed: false,
    })
}

fn code(text: &str) -> NodeValue {
    NodeValue::Code(NodeCode {
        num_backticks: 1,
        literal: text.to_string(),
    })
}

fn link(target: &str) -> NodeValue {
    NodeValue::Link(Box::new(NodeLink {
        url: target.to_string(),
        title: String::new(),
    }))
}

/// A list is printed tight (no blank lines inside) only when no item has
/// more than its first block besides sublists: two paragraphs of one item
/// in a tight list would be read back as one.
fn is_tight(items: &[Vec<Block>]) -> bool {
    let rest_are_lists = |item: &Vec<Block>| {
        let mut rest = item.iter().skip(1);
        rest.all(|block| matches!(block, Block::List { .. }))
    };
    items.iter().all(rest_are_lists)
}

fn list_node(ordered: bool, tight: bool) -> NodeList {
    NodeList {
        list_type: if ordered {
            ListType::Ordered
        } else {
            ListType::Bullet
        },
        start: 1,
        tight,
        ..NodeList::default()
    }
}

fn code_block(language: Option<&str>, text: &str) -> NodeValue {
    NodeValue::CodeBlock(Box::new(NodeCodeBlock {
        fenced: true,
        fence_char: b'`',
        fence_length: 3,
        info: language.unwrap_or_default().to_string(),
        literal: text.to_string(),
        closed: true,
        ..NodeCodeBlock::default()
    }))
}

fn table_node(columns: usize, rows: usize) -> NodeValue {
    NodeValue::Table(Box::new(NodeTable {
        alignments: vec![TableAlignment::None; columns],
        num_columns: columns,
        num_rows: rows,
        num_nonempty_cells: 0,
    }))
}
