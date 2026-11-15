//! Inline nodes of comrak's tree, as model inlines.

use comrak::nodes::{AstNode, NodeValue};

use crate::model::Inline;

/// The model inlines for the children of `parent`.
pub fn inlines_of<'a>(parent: &'a AstNode<'a>) -> Vec<Inline> {
    parent.children().flat_map(inline).collect()
}

/// The inlines for `node`: none for what the model leaves out, the content
/// alone for formatting it does not represent.
fn inline<'a>(node: &'a AstNode<'a>) -> Vec<Inline> {
    let single = match &node.data().value {
        NodeValue::Text(text) => Inline::Text(text.to_string()),
        NodeValue::Strong => Inline::Bold(inlines_of(node)),
        NodeValue::Emph => Inline::Italic(inlines_of(node)),
        NodeValue::Code(code) => Inline::Code(code.literal.clone()),
        NodeValue::Link(link) => Inline::Link {
            target: link.url.clone(),
            content: inlines_of(node),
        },
        NodeValue::SoftBreak => Inline::Text(" ".to_string()),
        NodeValue::LineBreak => Inline::LineBreak,
        // Images become their alt text until readers resolve image files.
        NodeValue::Strikethrough | NodeValue::Image(_) => return inlines_of(node),
        _ => return Vec::new(),
    };
    vec![single]
}
