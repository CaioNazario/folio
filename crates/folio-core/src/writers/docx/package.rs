//! The package around the document: content types, relationships and the
//! ZIP that holds the parts.

use std::io::{Cursor, Write};

use quick_xml::escape::escape;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

/// The namespace of package relationships.
const RELATIONSHIPS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
/// The prefix of the relationship types of a Word document.
pub const RELATIONSHIP_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// The prefix of the content types of WordprocessingML parts.
const WORD: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml";

/// The relationship type of the core properties, which is the package's.
const CORE_PROPERTIES: &str =
    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties";

/// A file of the package, with its content type when the extension does
/// not give it.
pub struct Part {
    pub name: String,
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
}

impl Part {
    pub fn new(name: &str, text: String) -> Self {
        Self {
            name: name.to_string(),
            bytes: text.into_bytes(),
            content_type: None,
        }
    }

    /// A WordprocessingML part of the type `{WORD}.{kind}+xml`.
    pub fn word(name: &str, kind: &str, text: String) -> Self {
        Self {
            content_type: Some(format!("{WORD}.{kind}+xml")),
            ..Self::new(name, text)
        }
    }
}

/// A relationship of a part: its id, type and target, inside the package
/// or outside it.
pub struct Relationship {
    pub id: String,
    pub kind: String,
    pub target: String,
    pub external: bool,
}

impl Relationship {
    /// A relationship of a Word document's type `kind` (the last part of
    /// the type's URI).
    pub fn new(id: &str, kind: &'static str, target: &str) -> Self {
        Self {
            id: id.to_string(),
            kind: format!("{RELATIONSHIP_TYPE}/{kind}"),
            target: target.to_string(),
            external: false,
        }
    }

    /// The package's relationship to its core properties.
    pub fn core_properties(id: &str, target: &str) -> Self {
        Self {
            kind: CORE_PROPERTIES.to_string(),
            ..Self::new(id, "", target)
        }
    }

    pub fn external(id: &str, kind: &'static str, target: &str) -> Self {
        Self {
            external: true,
            ..Self::new(id, kind, target)
        }
    }
}

/// A `.rels` part listing `relationships`.
pub fn relationships_part(name: &str, relationships: &[Relationship]) -> Part {
    let items: String = relationships.iter().map(relationship_xml).collect();
    Part::new(
        name,
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="{RELATIONSHIPS}">{items}</Relationships>"#
        ),
    )
}

fn relationship_xml(relationship: &Relationship) -> String {
    let mode = if relationship.external {
        r#" TargetMode="External""#
    } else {
        ""
    };
    format!(
        r#"<Relationship Id="{}" Type="{}" Target="{}"{mode}/>"#,
        relationship.id,
        relationship.kind,
        escape(relationship.target.as_str())
    )
}

/// `[Content_Types].xml`: the type of each part, by its name when it has
/// one of its own, by its extension otherwise.
pub fn content_types(parts: &[Part]) -> Part {
    let overrides: String = parts
        .iter()
        .filter_map(|part| Some((&part.name, part.content_type.as_ref()?)))
        .map(|(name, kind)| format!(r#"<Override PartName="/{name}" ContentType="{kind}"/>"#))
        .collect();
    Part::new(
        "[Content_Types].xml",
        format!(
            concat!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
                r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">"#,
                r#"<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>"#,
                r#"<Default Extension="xml" ContentType="application/xml"/>"#,
                "{}</Types>"
            ),
            overrides
        ),
    )
}

/// The ZIP holding `parts`.
pub fn zip(parts: &[Part]) -> Result<Vec<u8>, String> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    for part in parts {
        zip.start_file(part.name.as_str(), SimpleFileOptions::default())
            .map_err(|error| error.to_string())?;
        zip.write_all(&part.bytes)
            .map_err(|error| error.to_string())?;
    }
    let cursor = zip.finish().map_err(|error| error.to_string())?;
    Ok(cursor.into_inner())
}
