//! The metadata of a PDF: its Info dictionary, header version and page
//! count (spec 07, "Metadados").

use hayro_interpret::hayro_syntax::object::DateTime;
use hayro_interpret::hayro_syntax::{Pdf, PdfVersion};
use jiff::civil;
use jiff::tz::{Offset, TimeZone};
use serde_json::{Map, Value};

use super::super::ReadError;
use super::super::keywords::split_keywords;
use super::open::open;
use super::text_string;
use crate::model::Metadata;

/// The metadata of the PDF in `bytes`, without reading its pages.
pub fn read_pdf_metadata(bytes: &[u8]) -> Result<Metadata, ReadError> {
    Ok(metadata_of(&open(bytes, None)?))
}

/// The metadata of `pdf`.
pub fn metadata_of(pdf: &Pdf) -> Metadata {
    let info = pdf.metadata();
    Metadata {
        title: text(&info.title),
        author: text(&info.author),
        subject: text(&info.subject),
        keywords: split_keywords(text(&info.keywords)),
        created_at: info.creation_date.as_ref().and_then(millis),
        modified_at: info.modification_date.as_ref().and_then(millis),
        pages: u32::try_from(pdf.pages().len()).ok(),
        extra: extra_of(pdf),
    }
}

/// What the model has no field for: the producing application, the
/// converter that wrote the PDF and the header version.
fn extra_of(pdf: &Pdf) -> Map<String, Value> {
    let info = pdf.metadata();
    let mut extra = Map::new();
    for (key, value) in [("creator", &info.creator), ("producer", &info.producer)] {
        if let Some(text) = text(value) {
            extra.insert(key.to_string(), Value::String(text));
        }
    }
    extra.insert("version".to_string(), Value::String(version(pdf.version())));
    extra
}

/// An Info entry as text, trimmed; `None` when blank.
fn text(value: &Option<Box<[u8]>>) -> Option<String> {
    let text = text_string::decode(value.as_deref()?);
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

fn version(version: PdfVersion) -> String {
    let text = match version {
        PdfVersion::Pdf10 => "1.0",
        PdfVersion::Pdf11 => "1.1",
        PdfVersion::Pdf12 => "1.2",
        PdfVersion::Pdf13 => "1.3",
        PdfVersion::Pdf14 => "1.4",
        PdfVersion::Pdf15 => "1.5",
        PdfVersion::Pdf16 => "1.6",
        PdfVersion::Pdf17 => "1.7",
        PdfVersion::Pdf20 => "2.0",
    };
    text.to_string()
}

/// `date` in ms UTC; `None` when it is not a real date.
fn millis(date: &DateTime) -> Option<i64> {
    let sign = if date.utc_offset_hour < 0 { -1 } else { 1 };
    let offset =
        i32::from(date.utc_offset_hour) * 3600 + sign * i32::from(date.utc_offset_minute) * 60;
    let civil = civil::DateTime::new(
        i16::try_from(date.year).ok()?,
        i8::try_from(date.month).ok()?,
        i8::try_from(date.day).ok()?,
        i8::try_from(date.hour).ok()?,
        i8::try_from(date.minute).ok()?,
        i8::try_from(date.second).ok()?,
        0,
    )
    .ok()?;
    let zone = TimeZone::fixed(Offset::from_seconds(offset).ok()?);
    Some(civil.to_zoned(zone).ok()?.timestamp().as_millisecond())
}

#[cfg(test)]
mod tests {
    use super::super::test_pdf::{pdf_with_info, text};
    use super::*;

    fn metadata(info: &str) -> Metadata {
        let pages = [vec![text("Corpo.", 72.0, 700.0)]];
        read_pdf_metadata(&pdf_with_info(&pages, info)).unwrap()
    }

    #[test]
    fn pdf_metadata_reads_title_author_and_subject() {
        let metadata =
            metadata("/Title (Ata da reuniao) /Author (Ana Souza) /Subject (Condominio)");

        assert_eq!(
            (
                metadata.title.as_deref(),
                metadata.author.as_deref(),
                metadata.subject.as_deref()
            ),
            (
                Some("Ata da reuniao"),
                Some("Ana Souza"),
                Some("Condominio")
            )
        );
    }

    #[test]
    fn pdf_metadata_decodes_utf16_text_strings() {
        // "Ata da reunião" in UTF-16BE with its byte order mark.
        let metadata =
            metadata("/Title <FEFF0041007400610020006400610020007200650075006E006900E3006F>");

        assert_eq!(metadata.title.as_deref(), Some("Ata da reunião"));
    }

    #[test]
    fn pdf_metadata_decodes_pdfdoc_text_strings() {
        // PDFDocEncoding: \343 is "ã", as in Latin-1; \204 is an em dash.
        let metadata = metadata(r"/Title (Reuni\343o \204 ata)");

        assert_eq!(metadata.title.as_deref(), Some("Reunião — ata"));
    }

    #[test]
    fn pdf_metadata_splits_keywords_on_comma_or_semicolon() {
        let metadata = metadata("/Keywords (contrato, servicos;prazo ; )");

        assert_eq!(metadata.keywords, ["contrato", "servicos", "prazo"]);
    }

    #[test]
    fn pdf_metadata_converts_dates_with_offset() {
        let metadata =
            metadata("/CreationDate (D:20240310143000-03'00') /ModDate (D:20240311090000Z)");

        // 2024-03-10T17:30:00Z and 2024-03-11T09:00:00Z.
        assert_eq!(
            (metadata.created_at, metadata.modified_at),
            (Some(1_710_091_800_000), Some(1_710_147_600_000))
        );
    }

    #[test]
    fn pdf_metadata_counts_pages_exactly() {
        let page = || vec![text("Corpo.", 72.0, 700.0)];
        let three = pdf_with_info(&[page(), page(), page()], "");

        assert_eq!(read_pdf_metadata(&three).unwrap().pages, Some(3));
    }

    #[test]
    fn pdf_metadata_keeps_creator_producer_and_version_in_extra() {
        let metadata = metadata("/Creator (Writer) /Producer (LibreOffice 26.8)");

        let expected = serde_json::json!({
            "creator": "Writer",
            "producer": "LibreOffice 26.8",
            "version": "1.4",
        });
        assert_eq!(serde_json::Value::Object(metadata.extra), expected);
    }

    #[test]
    fn pdf_metadata_ignores_blank_properties() {
        let metadata = metadata("/Title (  ) /Author () /Producer ( )");

        assert_eq!((metadata.title, metadata.author), (None, None));
        assert!(!metadata.extra.contains_key("producer"));
    }
}
