//! What the body refers to in the other parts of the package.

use std::collections::HashMap;

use super::numbering::Numbering;
use super::package::Package;
use super::rels::Relationships;
use super::styles::Styles;
use crate::readers::ReadError;

pub struct Context {
    pub styles: Styles,
    pub numbering: Numbering,
    pub relationships: Relationships,
    /// The bytes of each embedded image, by relationship id.
    pub media: HashMap<String, Vec<u8>>,
}

impl Context {
    /// The parts `document.xml` refers to; each one is optional.
    pub fn of(package: &mut Package) -> Result<Self, ReadError> {
        let relationships = Relationships::of(package.optional_xml(RELS)?.as_ref());
        Ok(Self {
            styles: Styles::of(package.optional_xml("word/styles.xml")?.as_ref()),
            numbering: Numbering::of(package.optional_xml("word/numbering.xml")?.as_ref()),
            media: media_of(package, &relationships)?,
            relationships,
        })
    }
}

/// The image parts present in the package; a missing one is left out.
fn media_of(
    package: &mut Package,
    relationships: &Relationships,
) -> Result<HashMap<String, Vec<u8>>, ReadError> {
    let mut media = HashMap::new();
    for (id, part) in relationships.images() {
        if let Some(bytes) = package.bytes(&part)? {
            media.insert(id.to_string(), bytes);
        }
    }
    Ok(media)
}

/// Where the relationships of `document.xml` live.
pub const RELS: &str = "word/_rels/document.xml.rels";
