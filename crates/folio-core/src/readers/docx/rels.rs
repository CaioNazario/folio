//! The relationships of `document.xml` (`word/_rels/document.xml.rels`):
//! the targets that hyperlinks and images point to by id.

use std::collections::HashMap;

use super::xml::Element;

#[derive(Debug, Default)]
pub struct Relationships {
    by_id: HashMap<String, Relationship>,
}

#[derive(Debug)]
struct Relationship {
    target: String,
    image: bool,
}

impl Relationships {
    /// The relationships of the `Relationships` element; empty without the
    /// part.
    pub fn of(rels: Option<&Element>) -> Self {
        let all = rels.into_iter().flat_map(Element::elements);
        let by_id = all
            .filter(|element| element.name == "Relationship")
            .filter_map(|rel| Some((rel.attribute("Id")?.to_string(), relationship(rel)?)))
            .collect();
        Self { by_id }
    }

    /// The target of the relationship `id`.
    pub fn target(&self, id: &str) -> Option<&str> {
        self.by_id.get(id).map(|rel| rel.target.as_str())
    }

    /// The id and the package part of each image relationship.
    pub fn images(&self) -> impl Iterator<Item = (&str, String)> {
        let images = self.by_id.iter().filter(|(_, rel)| rel.image);
        images.map(|(id, rel)| (id.as_str(), part_name(&rel.target)))
    }
}

fn relationship(rel: &Element) -> Option<Relationship> {
    Some(Relationship {
        target: rel.attribute("Target")?.to_string(),
        image: rel
            .attribute("Type")
            .is_some_and(|kind| kind.ends_with("/image")),
    })
}

/// The part a target points to: relative to `word/`, or absolute in the
/// package when it starts with `/`.
fn part_name(target: &str) -> String {
    match target.strip_prefix('/') {
        Some(absolute) => absolute.to_string(),
        None => format!("word/{target}"),
    }
}
