//! `docProps/core.xml`: the document's own metadata (spec 08, "Escritores").

use jiff::Timestamp;
use quick_xml::escape::escape;

use super::package::Part;
use crate::model::Metadata;

const NAMESPACES: &str = concat!(
    r#"xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" "#,
    r#"xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" "#,
    r#"xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance""#,
);
pub const CORE_PROPERTIES: &str = "docProps/core.xml";
const CONTENT_TYPE: &str = "application/vnd.openxmlformats-package.core-properties+xml";

/// The core properties part, with the fields of `metadata` that are set;
/// `None` when none is.
pub fn core_properties(metadata: &Metadata) -> Option<Part> {
    let properties: String = [texts(metadata), dates(metadata)].concat().concat();
    if properties.is_empty() {
        return None;
    }
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><cp:coreProperties {NAMESPACES}>{properties}</cp:coreProperties>"#
    );
    Some(Part {
        content_type: Some(CONTENT_TYPE.to_string()),
        ..Part::new(CORE_PROPERTIES, xml)
    })
}

/// Title, subject, author and keywords (joined by `; `).
fn texts(metadata: &Metadata) -> Vec<String> {
    let keywords = (!metadata.keywords.is_empty()).then(|| metadata.keywords.join("; "));
    let fields = [
        ("dc:title", metadata.title.clone()),
        ("dc:subject", metadata.subject.clone()),
        ("dc:creator", metadata.author.clone()),
        ("cp:keywords", keywords),
    ];
    let set = fields
        .into_iter()
        .filter_map(|(name, value)| Some((name, value?)));
    set.map(|(name, value)| format!("<{name}>{}</{name}>", escape(value.as_str())))
        .collect()
}

/// Creation and modification, in W3CDTF (UTC).
fn dates(metadata: &Metadata) -> Vec<String> {
    let fields = [
        ("dcterms:created", metadata.created_at),
        ("dcterms:modified", metadata.modified_at),
    ];
    let set = fields
        .into_iter()
        .filter_map(|(name, millis)| Some((name, w3cdtf(millis?)?)));
    set.map(|(name, date)| format!(r#"<{name} xsi:type="dcterms:W3CDTF">{date}</{name}>"#))
        .collect()
}

fn w3cdtf(millis: i64) -> Option<String> {
    let instant = Timestamp::from_millisecond(millis).ok()?;
    Some(instant.strftime("%Y-%m-%dT%H:%M:%SZ").to_string())
}

#[cfg(test)]
mod tests {
    use crate::model::DocumentModel;
    use crate::writers::docx::testing::*;
    use crate::writers::docx::write_docx;

    /// 2026-03-18T21:30:00Z and 2026-03-19T08:15:00Z.
    const CREATED: i64 = 1_773_869_400_000;
    const MODIFIED: i64 = 1_773_908_100_000;

    /// The DOCX of a one-paragraph model with `metadata`.
    fn docx_with_metadata(metadata: crate::model::Metadata) -> Vec<u8> {
        let model = DocumentModel {
            metadata,
            blocks: vec![paragraph("Texto.")],
            ..DocumentModel::default()
        };
        write_docx(&model).unwrap()
    }

    #[test]
    fn write_docx_metadata_reads_back() {
        let metadata = crate::model::Metadata {
            title: Some("Relatório & análise".to_string()),
            author: Some("Ana".to_string()),
            subject: Some("Física".to_string()),
            keywords: vec!["ótica".to_string(), "luz".to_string()],
            created_at: Some(CREATED),
            modified_at: Some(MODIFIED),
            ..crate::model::Metadata::default()
        };

        let read = crate::readers::read_docx(&docx_with_metadata(metadata.clone()), None).unwrap();

        // Without `app.xml`, the reader counts the pages itself.
        let expected = crate::model::Metadata {
            pages: Some(1),
            ..metadata
        };
        assert_eq!(read.metadata, expected);
    }

    #[test]
    fn write_docx_has_no_core_properties_without_metadata() {
        let mut metadata = crate::model::Metadata {
            pages: Some(12),
            ..crate::model::Metadata::default()
        };
        metadata.extra.insert("producer".to_string(), "Word".into());

        let docx = docx_with_metadata(metadata);

        assert!(!part_names(&docx).contains(&"docProps/core.xml".to_string()));
        assert!(!part(&docx, "_rels/.rels").contains("core-properties"));
    }
}
