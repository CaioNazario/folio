//! Document formats of 0.1.0, decided by extension (spec 04).

use std::ffi::OsStr;
use std::path::Path;

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

use crate::paths::os_bytes;

/// A format Folio reads in 0.1.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    Pdf,
    Docx,
    Md,
    Txt,
}

impl Format {
    /// The format of a file named `name`, by extension and ignoring case;
    /// `None` when it is not a document.
    pub fn from_name(name: &OsStr) -> Option<Self> {
        // Read from the bytes: a non-UTF-8 name with `.pdf` is still a PDF.
        let extension = os_bytes::as_bytes(Path::new(name).extension()?).to_ascii_lowercase();
        match extension.as_slice() {
            b"pdf" => Some(Self::Pdf),
            b"docx" => Some(Self::Docx),
            b"md" | b"markdown" => Some(Self::Md),
            b"txt" => Some(Self::Txt),
            _ => None,
        }
    }

    /// The MIME type, derived from the format and never stored.
    pub fn mime_type(self) -> &'static str {
        match self {
            Self::Pdf => "application/pdf",
            Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Self::Md => "text/markdown",
            Self::Txt => "text/plain",
        }
    }
}

/// Files that never appear even with a document extension: hidden ones
/// (which covers `.folio-tmp-*` and LibreOffice's `.~lock.*#`) and Word's
/// `~$` locks.
pub fn is_ignored_name(name: &OsStr) -> bool {
    let bytes = os_bytes::as_bytes(name);
    bytes.starts_with(b".") || bytes.starts_with(b"~$")
}

impl Format {
    /// The name stored in the index.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Docx => "docx",
            Self::Md => "md",
            Self::Txt => "txt",
        }
    }
}

impl ToSql for Format {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

impl FromSql for Format {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_str()? {
            "pdf" => Ok(Self::Pdf),
            "docx" => Ok(Self::Docx),
            "md" => Ok(Self::Md),
            "txt" => Ok(Self::Txt),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format_of(name: &str) -> Option<Format> {
        Format::from_name(OsStr::new(name))
    }

    #[test]
    fn format_comes_from_extension_ignoring_case() {
        assert_eq!(format_of("Trabalho.PDF"), Some(Format::Pdf));
        assert_eq!(format_of("Contrato.Docx"), Some(Format::Docx));
        assert_eq!(format_of("notas.md"), Some(Format::Md));
        assert_eq!(format_of("README.MARKDOWN"), Some(Format::Md));
        assert_eq!(format_of("lista.txt"), Some(Format::Txt));
    }

    #[test]
    fn unknown_extension_is_not_a_document() {
        for name in ["antigo.doc", "planilha.xlsx", "foto.png", "Makefile"] {
            assert_eq!(format_of(name), None, "{name}");
        }
    }

    #[test]
    fn non_utf8_name_with_pdf_extension_is_pdf() {
        use std::os::unix::ffi::OsStrExt;
        let name = OsStr::from_bytes(b"relat\xf3rio.pdf");

        assert_eq!(Format::from_name(name), Some(Format::Pdf));
    }

    #[test]
    fn hidden_files_and_word_locks_are_ignored() {
        for name in [
            ".rascunho.pdf",
            ".folio-tmp-a1b2",
            ".~lock.Contrato.docx#",
            "~$ntrato.docx",
        ] {
            assert!(is_ignored_name(OsStr::new(name)), "{name}");
        }
        assert!(!is_ignored_name(OsStr::new("Contrato.docx")));
    }

    #[test]
    fn mime_type_comes_from_format() {
        let docx = "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
        assert_eq!(Format::Pdf.mime_type(), "application/pdf");
        assert_eq!(Format::Docx.mime_type(), docx);
        assert_eq!(Format::Md.mime_type(), "text/markdown");
        assert_eq!(Format::Txt.mime_type(), "text/plain");
    }
}
