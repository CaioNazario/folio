//! A small XML tree of the parts of a package. Element and attribute names
//! keep only their local name (`w:p` is `p`): the readers look at one
//! vocabulary per part, so prefixes add nothing.

use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesRef, BytesStart, Event};
use quick_xml::{Reader, XmlVersion};

use crate::readers::ReadError;

/// An element with its attributes and children.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Element {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Element(Element),
    Text(String),
}

impl Element {
    /// The value of the attribute with local name `name`.
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// The child elements, in order.
    pub fn elements(&self) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(|node| match node {
            Node::Element(element) => Some(element),
            Node::Text(_) => None,
        })
    }

    /// The first child element named `name`.
    pub fn child(&self, name: &str) -> Option<&Element> {
        self.elements().find(|element| element.name == name)
    }

    /// Every element named `name` below this one, in document order.
    pub fn descendants(&self, name: &str) -> Vec<&Element> {
        let mut found = Vec::new();
        for child in self.elements() {
            if child.name == name {
                found.push(child);
            }
            found.extend(child.descendants(name));
        }
        found
    }

    /// The text directly inside this element.
    pub fn text(&self) -> String {
        self.children
            .iter()
            .filter_map(|node| match node {
                Node::Text(text) => Some(text.as_str()),
                Node::Element(_) => None,
            })
            .collect()
    }
}

/// The root element of `xml`.
pub fn parse(xml: &str) -> Result<Element, ReadError> {
    let mut reader = Reader::from_str(xml);
    // The synthetic document node holds the root element.
    let mut stack = vec![Element::default()];
    loop {
        match reader.read_event().map_err(failed)? {
            Event::Start(start) => stack.push(element_of(&start)?),
            Event::Empty(start) => push_child(&mut stack, Node::Element(element_of(&start)?)),
            Event::End(_) => close(&mut stack)?,
            Event::Text(text) => push_text(&mut stack, &text.xml10_content()),
            Event::CData(data) => push_text(&mut stack, &data.xml10_content()),
            Event::GeneralRef(reference) => push_text(&mut stack, &resolve(&reference)?),
            Event::Eof => return root_of(stack),
            _ => {}
        }
    }
}

fn element_of(start: &BytesStart) -> Result<Element, ReadError> {
    let mut attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(failed)?;
        let value = attribute
            .normalized_value(XmlVersion::Implicit1_0)
            .map_err(failed)?;
        let key = attribute.key.local_name();
        attributes.push((key.as_ref().to_string(), value.into_owned()));
    }
    Ok(Element {
        name: start.local_name().as_ref().to_string(),
        attributes,
        children: Vec::new(),
    })
}

fn push_child(stack: &mut [Element], node: Node) {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(node);
    }
}

/// Text next to text is merged, so entities do not split it.
fn push_text(stack: &mut [Element], text: &str) {
    let Some(parent) = stack.last_mut() else {
        return;
    };
    if let Some(Node::Text(last)) = parent.children.last_mut() {
        last.push_str(text);
        return;
    }
    parent.children.push(Node::Text(text.to_string()));
}

fn close(stack: &mut Vec<Element>) -> Result<(), ReadError> {
    let element = stack.pop().filter(|_| !stack.is_empty());
    let element = element.ok_or_else(|| ReadError::Failed("unbalanced end tag".into()))?;
    push_child(stack, Node::Element(element));
    Ok(())
}

fn resolve(reference: &BytesRef) -> Result<String, ReadError> {
    if let Some(ch) = reference.resolve_char_ref().map_err(failed)? {
        return Ok(ch.to_string());
    }
    resolve_predefined_entity(reference)
        .map(str::to_string)
        .ok_or_else(|| ReadError::Failed(format!("unknown entity &{};", &**reference)))
}

fn root_of(mut stack: Vec<Element>) -> Result<Element, ReadError> {
    if stack.len() != 1 {
        return Err(ReadError::Failed("unclosed element".into()));
    }
    let document = stack.remove(0);
    document
        .elements()
        .next()
        .cloned()
        .ok_or_else(|| ReadError::Failed("no root element".into()))
}

fn failed(error: impl std::fmt::Display) -> ReadError {
    ReadError::Failed(error.to_string())
}
