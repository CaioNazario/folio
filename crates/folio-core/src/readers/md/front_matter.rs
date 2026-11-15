//! YAML front matter as document metadata (spec 07, "Metadados").

use jiff::Timestamp;
use jiff::civil::Date;
use jiff::tz::TimeZone;
use saphyr::{LoadableYamlNode, Yaml};
use serde_json::Value as JsonValue;

use super::yaml_json::to_json;
use crate::model::Metadata;

/// The metadata in a front matter block, delimiters included.
pub fn metadata_of(front_matter: &str) -> Metadata {
    let mut metadata = Metadata::default();
    let Some(document) = Yaml::load_from_str(&yaml_body(front_matter))
        .ok()
        .and_then(|documents| documents.into_iter().next())
    else {
        return metadata;
    };
    for (key, value) in document.as_mapping().into_iter().flatten() {
        apply(&mut metadata, key.as_str().unwrap_or_default(), value);
    }
    metadata
}

/// The lines between the opening and the closing `---`.
fn yaml_body(front_matter: &str) -> String {
    let lines = front_matter.lines().skip(1);
    let body: Vec<&str> = lines.take_while(|line| line.trim_end() != "---").collect();
    body.join("\n")
}

/// Records the field `key` of the front matter in `metadata`.
fn apply(metadata: &mut Metadata, key: &str, value: &Yaml<'_>) {
    let text = value.as_str().map(str::to_string);
    match key {
        "title" => metadata.title = text,
        "author" => metadata.author = text,
        "tags" | "keywords" => metadata.keywords = strings(value),
        "date" => apply_date(metadata, text),
        _ => _ = metadata.extra.insert(key.to_string(), to_json(value)),
    }
}

/// A date Folio cannot read is kept as written, in `extra`.
fn apply_date(metadata: &mut Metadata, text: Option<String>) {
    let Some(text) = text else { return };
    match date_millis(&text) {
        Some(millis) => metadata.created_at = Some(millis),
        None => {
            _ = metadata
                .extra
                .insert("date".to_string(), JsonValue::String(text))
        }
    }
}

/// The strings of a YAML sequence, or the single string given instead.
fn strings(value: &Yaml<'_>) -> Vec<String> {
    if let Some(single) = value.as_str() {
        return vec![single.to_string()];
    }
    let items = value.as_sequence().into_iter().flatten();
    items.filter_map(Yaml::as_str).map(str::to_string).collect()
}

/// An RFC 3339 instant, or a `YYYY-MM-DD` date as midnight UTC, in ms.
/// The instant goes first: `Date` would accept it and drop the time.
fn date_millis(text: &str) -> Option<i64> {
    if let Ok(instant) = text.parse::<Timestamp>() {
        return Some(instant.as_millisecond());
    }
    let date: Date = text.parse().ok()?;
    let zoned = date.to_zoned(TimeZone::UTC).ok()?;
    Some(zoned.timestamp().as_millisecond())
}

#[cfg(test)]
mod tests {
    use crate::model::{Block, Inline, Metadata};
    use crate::readers::read_md;

    /// 2026-11-15T00:00:00Z, in ms.
    const NOV_15: i64 = 1_794_700_800_000;

    fn metadata(front_matter: &str) -> Metadata {
        read_md(format!("---\n{front_matter}---\n\nCorpo\n").as_bytes()).metadata
    }

    #[test]
    fn md_metadata_reads_title_author_tags_and_date() {
        let read =
            metadata("title: Relatório\nauthor: Ana\ntags: [estudo, rust]\ndate: 2026-11-15\n");

        assert_eq!(
            read,
            Metadata {
                title: Some("Relatório".to_string()),
                author: Some("Ana".to_string()),
                keywords: vec!["estudo".to_string(), "rust".to_string()],
                created_at: Some(NOV_15),
                ..Metadata::default()
            }
        );
    }

    #[test]
    fn md_metadata_accepts_keywords_instead_of_tags() {
        let read = metadata("keywords:\n  - contrato\n  - aluguel\n");

        assert_eq!(read.keywords, ["contrato", "aluguel"]);
    }

    #[test]
    fn md_metadata_accepts_single_string_tag() {
        assert_eq!(metadata("tags: estudo\n").keywords, ["estudo"]);
    }

    #[test]
    fn md_metadata_parses_rfc3339_date() {
        let read = metadata("date: 2026-11-15T14:30:00-03:00\n");

        // 17:30 UTC.
        assert_eq!(read.created_at, Some(NOV_15 + 17 * 3_600_000 + 30 * 60_000));
    }

    #[test]
    fn md_metadata_keeps_unparseable_date_in_extra() {
        let read = metadata("date: semana que vem\n");

        assert_eq!(
            (read.created_at, read.extra.get("date")),
            (None, Some(&serde_json::json!("semana que vem")))
        );
    }

    #[test]
    fn md_metadata_puts_other_fields_in_extra() {
        let read = metadata(
            "title: T\nlang: pt\ndraft: true\nversion: 3\nratio: 1.5\nempty:\nnested:\n  list: [1, x]\n",
        );

        let expected = serde_json::json!({
            "lang": "pt",
            "draft": true,
            "version": 3,
            "ratio": 1.5,
            "empty": null,
            "nested": { "list": [1, "x"] },
        });
        assert_eq!(serde_json::Value::Object(read.extra), expected);
    }

    #[test]
    fn front_matter_with_invalid_yaml_yields_no_metadata() {
        let model = read_md(b"---\ntitle: [aberto\n---\n\nCorpo\n");

        assert_eq!(model.metadata, Metadata::default());
        assert_eq!(model.blocks.len(), 1);
    }

    #[test]
    fn toml_front_matter_is_kept_as_body_text() {
        let model = read_md(b"+++\ntitle = 'T'\n+++\n\nCorpo\n");

        let text = |s: &str| Inline::Text(s.to_string());
        assert_eq!(model.metadata, Metadata::default());
        assert_eq!(
            model.blocks[0],
            Block::Paragraph(vec![
                text("+++"),
                text(" "),
                text("title = 'T'"),
                text(" "),
                text("+++"),
            ])
        );
    }
}
