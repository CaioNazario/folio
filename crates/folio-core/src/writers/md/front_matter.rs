//! The metadata as YAML front matter (spec 08, "Escritores").

use std::borrow::Cow;

use jiff::Timestamp;
use saphyr::{Mapping, Scalar, Yaml, YamlEmitter};

use crate::model::Metadata;

/// The front matter of `metadata`, closed by `---` and a blank line; empty
/// when none of the fields it writes is set.
pub fn front_matter(metadata: &Metadata) -> String {
    let fields = fields(metadata);
    if fields.is_empty() {
        return String::new();
    }
    // The emitter opens the block with `---`. Writing to a String never fails.
    let mut yaml = String::new();
    _ = YamlEmitter::new(&mut yaml).dump(&Yaml::Mapping(fields));
    format!("{yaml}\n---\n\n")
}

/// The fields that are set, in a fixed order; `extra` stays out.
fn fields(metadata: &Metadata) -> Mapping<'static> {
    let mut fields = text_fields(metadata);
    if !metadata.keywords.is_empty() {
        let tags = metadata
            .keywords
            .iter()
            .map(|keyword| string(keyword))
            .collect();
        fields.insert(string("tags"), Yaml::Sequence(tags));
    }
    if let Some(date) = metadata.created_at.and_then(rfc3339) {
        fields.insert(string("date"), string(&date));
    }
    fields
}

fn text_fields(metadata: &Metadata) -> Mapping<'static> {
    let texts = [
        ("title", &metadata.title),
        ("author", &metadata.author),
        ("subject", &metadata.subject),
    ];
    let set = texts
        .into_iter()
        .filter_map(|(key, value)| Some((key, value.as_deref()?)));
    set.map(|(key, value)| (string(key), string(value)))
        .collect()
}

fn string(text: &str) -> Yaml<'static> {
    Yaml::Value(Scalar::String(Cow::Owned(text.to_string())))
}

fn rfc3339(millis: i64) -> Option<String> {
    Timestamp::from_millisecond(millis)
        .ok()
        .map(|instant| instant.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Block, DocumentModel, Inline};
    use crate::writers::write_md;

    fn paragraph(content: &str) -> Block {
        Block::Paragraph(vec![Inline::Text(content.to_string())])
    }

    /// 2026-03-18T21:30:00Z.
    const CREATED: i64 = 1_773_869_400_000;

    fn with_metadata(metadata: Metadata) -> String {
        let model = DocumentModel {
            metadata,
            blocks: vec![paragraph("Texto.")],
        };
        write_md(&model, "Doc.assets").text
    }

    #[test]
    fn write_md_writes_metadata_as_front_matter() {
        let metadata = Metadata {
            title: Some("Relatório".to_string()),
            author: Some("Ana".to_string()),
            subject: Some("Física".to_string()),
            keywords: vec!["ótica".to_string(), "luz".to_string()],
            created_at: Some(CREATED),
            ..Metadata::default()
        };

        assert_eq!(
            with_metadata(metadata),
            "---\ntitle: Relatório\nauthor: Ana\nsubject: Física\ntags:\n  - ótica\n  - luz\n\
             date: \"2026-03-18T21:30:00Z\"\n---\n\nTexto.\n"
        );
    }

    #[test]
    fn write_md_omits_absent_metadata_fields() {
        let metadata = Metadata {
            author: Some("Ana".to_string()),
            ..Metadata::default()
        };

        assert_eq!(with_metadata(metadata), "---\nauthor: Ana\n---\n\nTexto.\n");
    }

    #[test]
    fn write_md_has_no_front_matter_without_metadata() {
        let mut metadata = Metadata {
            pages: Some(3),
            modified_at: Some(CREATED),
            ..Metadata::default()
        };
        metadata.extra.insert("producer".to_string(), "Word".into());

        assert_eq!(with_metadata(metadata), "Texto.\n");
    }

    #[test]
    fn write_md_escapes_yaml_in_front_matter() {
        let metadata = Metadata {
            title: Some("Física: uma introdução #1".to_string()),
            author: Some("O'Brien".to_string()),
            keywords: vec!["- lista".to_string(), "true".to_string(), "42".to_string()],
            ..Metadata::default()
        };

        let read = crate::readers::read_md(with_metadata(metadata.clone()).as_bytes()).metadata;

        assert_eq!(
            (read.title, read.author, read.keywords),
            (metadata.title, metadata.author, metadata.keywords)
        );
    }

    #[test]
    fn write_md_front_matter_reads_back_as_same_metadata() {
        let metadata = Metadata {
            title: Some("Relatório".to_string()),
            author: Some("Ana".to_string()),
            subject: Some("Física".to_string()),
            keywords: vec!["ótica".to_string(), "luz".to_string()],
            created_at: Some(CREATED),
            ..Metadata::default()
        };

        let read = crate::readers::read_md(with_metadata(metadata.clone()).as_bytes()).metadata;

        // The Markdown reader keeps `subject` among the other fields (spec 07).
        let mut expected = Metadata {
            subject: None,
            ..metadata
        };
        expected
            .extra
            .insert("subject".to_string(), "Física".into());
        assert_eq!(read, expected);
    }
}
