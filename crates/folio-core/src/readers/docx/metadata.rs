//! The metadata of a DOCX (spec 07, "Metadados"): `docProps/core.xml`, and
//! the pages Word saved in `docProps/app.xml` or, without them, counted from
//! the explicit page breaks.

use jiff::Timestamp;
use serde_json::{Map, Value};

use super::inlines::is_page_break;
use super::package::Package;
use super::xml::Element;
use crate::model::Metadata;
use crate::readers::ReadError;
use crate::readers::keywords::split_keywords;

/// The metadata of a DOCX file, without building its document model.
pub fn read_docx_metadata(bytes: &[u8]) -> Result<Metadata, ReadError> {
    let mut package = Package::open(bytes)?;
    let document = package.xml("word/document.xml")?;
    metadata_of(&mut package, &document)
}

/// The metadata of an open `package` whose `document.xml` is `document`.
pub fn metadata_of(package: &mut Package, document: &Element) -> Result<Metadata, ReadError> {
    let core = package.optional_xml("docProps/core.xml")?;
    let app = package.optional_xml("docProps/app.xml")?;
    let metadata = core.as_ref().map(core_metadata).unwrap_or_default();
    let pages = app
        .as_ref()
        .and_then(saved_pages)
        .unwrap_or_else(|| pages_of(document));
    Ok(Metadata {
        pages: Some(pages),
        ..metadata
    })
}

/// The pages Word counted at its last save (`<Pages>` in `app.xml`);
/// `None` when missing or not a number.
fn saved_pages(app: &Element) -> Option<u32> {
    app.child("Pages")?.text().trim().parse().ok()
}

/// Approximate pages, without Word's count: one, plus one per explicit
/// page break (a DOCX from Google Docs has no `app.xml`).
fn pages_of(document: &Element) -> u32 {
    let breaks = document
        .descendants("br")
        .into_iter()
        .filter(|br| is_page_break(br));
    let breaks = u32::try_from(breaks.count()).unwrap_or(u32::MAX);
    breaks.saturating_add(1)
}

/// The properties of `cp:coreProperties`, by local name.
fn core_metadata(core: &Element) -> Metadata {
    let property = |name: &str| property(core, name);
    Metadata {
        title: property("title"),
        author: property("creator"),
        subject: property("subject"),
        keywords: split_keywords(property("keywords")),
        created_at: property("created").and_then(|text| millis(&text)),
        modified_at: property("modified").and_then(|text| millis(&text)),
        extra: extra(core),
        ..Metadata::default()
    }
}

/// The trimmed text of the property `name`; `None` when missing or blank.
fn property(core: &Element, name: &str) -> Option<String> {
    let text = core.child(name)?.text();
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// A W3CDTF instant (`2026-11-15T10:00:00Z`) in ms UTC.
fn millis(text: &str) -> Option<i64> {
    let instant = text.parse::<Timestamp>().ok()?;
    Some(instant.as_millisecond())
}

/// The properties the model has no field for, by the name Properties
/// shows: `dc:description` is what Word calls comments.
fn extra(core: &Element) -> Map<String, Value> {
    let fields = [("comments", "description"), ("revision", "revision")];
    fields
        .into_iter()
        .filter_map(|(key, name)| Some((key.to_string(), Value::String(property(core, name)?))))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::test_docx::{app, core, docx, docx_with};
    use super::*;

    /// The metadata of an empty document with `core.xml` holding
    /// `properties`.
    fn metadata(properties: &str) -> Metadata {
        let core = core(properties);
        read_docx_metadata(&docx_with("", &[("docProps/core.xml", core.as_bytes())])).unwrap()
    }

    #[test]
    fn docx_metadata_reads_title_author_subject() {
        let properties =
            "<dc:title>Tese</dc:title><dc:creator>Ana</dc:creator><dc:subject>Física</dc:subject>";

        let metadata = metadata(properties);

        assert_eq!(
            (metadata.title, metadata.author, metadata.subject),
            (
                Some("Tese".into()),
                Some("Ana".into()),
                Some("Física".into())
            )
        );
    }

    #[test]
    fn docx_metadata_ignores_empty_properties() {
        let metadata = metadata("<dc:title>  </dc:title><dc:subject></dc:subject>");

        assert_eq!((metadata.title, metadata.subject), (None, None));
    }

    #[test]
    fn docx_metadata_splits_keywords_on_commas_and_semicolons() {
        let metadata = metadata("<cp:keywords>rust, leitura; notas;; </cp:keywords>");

        assert_eq!(metadata.keywords, ["rust", "leitura", "notas"]);
    }

    #[test]
    fn docx_metadata_reads_created_and_modified_dates() {
        let metadata = metadata(concat!(
            r#"<dcterms:created xsi:type="dcterms:W3CDTF">2026-11-15T10:00:00Z</dcterms:created>"#,
            r#"<dcterms:modified xsi:type="dcterms:W3CDTF">2026-11-16T12:30:00Z</dcterms:modified>"#,
        ));

        // 2026-11-15T10:00:00Z and 2026-11-16T12:30:00Z.
        assert_eq!(
            (metadata.created_at, metadata.modified_at),
            (Some(1_794_736_800_000), Some(1_794_832_200_000))
        );
    }

    #[test]
    fn docx_metadata_puts_comments_and_revision_in_extra() {
        let metadata = metadata(
            "<dc:description>Notas do semestre</dc:description><cp:revision>3</cp:revision>",
        );

        let mut extra = serde_json::Map::new();
        extra.insert("comments".to_string(), "Notas do semestre".into());
        extra.insert("revision".to_string(), "3".into());
        assert_eq!(metadata.extra, extra);
    }

    #[test]
    fn docx_metadata_counts_pages_from_explicit_page_breaks() {
        let body = concat!(
            "<w:p><w:r><w:t>Capa</w:t><w:br w:type=\"page\"/></w:r></w:p>",
            "<w:p><w:r><w:t>Um</w:t><w:br/><w:t>Dois</w:t></w:r></w:p>",
            "<w:p><w:r><w:br w:type=\"page\"/></w:r></w:p><w:p><w:r><w:t>Fim</w:t></w:r></w:p>",
        );

        let metadata = read_docx_metadata(&docx(body)).unwrap();

        assert_eq!(metadata.pages, Some(3));
    }

    /// One explicit page break: two pages when counted by breaks.
    const TWO_PAGES_BY_BREAKS: &str = concat!(
        "<w:p><w:r><w:t>Capa</w:t><w:br w:type=\"page\"/></w:r></w:p>",
        "<w:p><w:r><w:t>Fim</w:t></w:r></w:p>",
    );

    #[test]
    fn docx_metadata_pages_come_from_app_properties() {
        let app = app("<Pages>36</Pages><Words>9120</Words>");
        let bytes = docx_with(TWO_PAGES_BY_BREAKS, &[("docProps/app.xml", app.as_bytes())]);

        let metadata = read_docx_metadata(&bytes).unwrap();

        assert_eq!(metadata.pages, Some(36));
    }

    #[test]
    fn docx_metadata_pages_fall_back_to_page_breaks_without_app_properties() {
        let metadata = read_docx_metadata(&docx(TWO_PAGES_BY_BREAKS)).unwrap();

        assert_eq!(metadata.pages, Some(2));
    }

    #[test]
    fn docx_metadata_pages_fall_back_when_app_properties_pages_is_not_a_number() {
        let app = app("<Pages>dez</Pages>");
        let bytes = docx_with(TWO_PAGES_BY_BREAKS, &[("docProps/app.xml", app.as_bytes())]);

        let metadata = read_docx_metadata(&bytes).unwrap();

        assert_eq!(metadata.pages, Some(2));
    }

    #[test]
    fn docx_metadata_is_empty_without_core_xml() {
        let metadata =
            read_docx_metadata(&docx("<w:p><w:r><w:t>Só texto</w:t></w:r></w:p>")).unwrap();

        let expected = Metadata {
            pages: Some(1),
            ..Metadata::default()
        };
        assert_eq!(metadata, expected);
    }

    #[test]
    fn read_docx_fills_model_metadata() {
        let core = core("<dc:title>Tese</dc:title>");
        let bytes = docx_with("", &[("docProps/core.xml", core.as_bytes())]);

        let model = super::super::read_docx(&bytes, None).unwrap();

        assert_eq!(model.metadata, read_docx_metadata(&bytes).unwrap());
    }
}
