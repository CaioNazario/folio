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
