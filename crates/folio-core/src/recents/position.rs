//! Where the user stopped reading, to continue from there (spec 06,
//! "Continuar de onde parou").

use rusqlite::OptionalExtension;
use uuid::Uuid;

use crate::documents::Format;
use crate::workspace::Workspace;
use crate::{Error, Result};

/// A place in a document, in the unit its format pages by.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum ReadingPosition {
    /// Page number, from 1 (PDF, DOCX).
    Page { page: u32 },
    /// Fraction of the scroll, from 0 to 1 (MD, TXT).
    Scroll { fraction: f64 },
}

impl ReadingPosition {
    /// Paged formats take a page from 1; the others, a scroll fraction from
    /// 0 to 1.
    fn fits(self, format: Format) -> bool {
        match (self, format) {
            (Self::Page { page }, Format::Pdf | Format::Docx) => page >= 1,
            (Self::Scroll { fraction }, Format::Md | Format::Txt) => {
                (0.0..=1.0).contains(&fraction)
            }
            _ => false,
        }
    }

    /// The stored value: the page number or the fraction.
    fn stored(self) -> f64 {
        match self {
            Self::Page { page } => f64::from(page),
            Self::Scroll { fraction } => fraction,
        }
    }

    /// The stored `value` read back in the unit of `format`.
    fn from_stored(format: Format, value: f64) -> Self {
        match format {
            Format::Pdf | Format::Docx => Self::Page { page: value as u32 },
            Format::Md | Format::Txt => Self::Scroll { fraction: value },
        }
    }
}

impl Workspace {
    /// Keeps `position` as where the user stopped in the document `id`.
    pub fn save_position(&self, id: Uuid, position: ReadingPosition) -> Result<()> {
        if !position.fits(self.document(id)?.format) {
            return Err(Error::InvalidPosition);
        }
        self.conn()
            .execute(SAVE, (id.to_string(), position.stored()))
            .map(drop)
            .map_err(|e| self.db_error(e))
    }

    /// Where the user stopped in the document `id`, if anywhere.
    pub fn position(&self, id: Uuid) -> Result<Option<ReadingPosition>> {
        let row = self
            .conn()
            .query_row(LOAD, [id.to_string()], |row| Ok((row.get(0)?, row.get(1)?)))
            .optional()
            .map_err(|e| self.db_error(e))?;
        Ok(row.map(|(format, value)| ReadingPosition::from_stored(format, value)))
    }
}

const SAVE: &str = "UPDATE recent_documents SET position = ?2 WHERE document_id = ?1";

const LOAD: &str = "SELECT d.format, r.position
     FROM recent_documents r JOIN documents d ON d.id = r.document_id
     WHERE r.document_id = ?1 AND r.position IS NOT NULL";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::test_docs::write_doc;
    use crate::workspace::test_env::TestEnv;

    /// Indexed and opened, as the Reader leaves it before saving a position.
    fn opened(workspace: &Workspace, path: &str, content: &[u8]) -> Uuid {
        let path = write_doc(workspace, path, content);
        let id = workspace.index_file(&path).unwrap().unwrap().id;
        workspace.record_opened(id).unwrap();
        id
    }

    #[test]
    fn saved_position_is_returned_for_document() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let pdf = opened(&workspace, "Livro.pdf", b"%PDF-1.7\n");
        let txt = opened(&workspace, "Nota.txt", b"text");

        workspace
            .save_position(pdf, ReadingPosition::Page { page: 12 })
            .unwrap();
        workspace
            .save_position(txt, ReadingPosition::Scroll { fraction: 0.5 })
            .unwrap();

        assert_eq!(
            (
                workspace.position(pdf).unwrap(),
                workspace.position(txt).unwrap()
            ),
            (
                Some(ReadingPosition::Page { page: 12 }),
                Some(ReadingPosition::Scroll { fraction: 0.5 })
            )
        );
    }

    #[test]
    fn page_position_on_text_document_fails() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let txt = opened(&workspace, "Nota.txt", b"text");

        let result = workspace.save_position(txt, ReadingPosition::Page { page: 3 });

        assert!(matches!(result, Err(Error::InvalidPosition)));
        assert_eq!(workspace.position(txt).unwrap(), None);
    }

    #[test]
    fn scroll_position_on_paged_document_fails() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let pdf = opened(&workspace, "Livro.pdf", b"%PDF-1.7\n");

        let result = workspace.save_position(pdf, ReadingPosition::Scroll { fraction: 0.5 });

        assert!(matches!(result, Err(Error::InvalidPosition)));
    }

    #[test]
    fn scroll_fraction_outside_0_to_1_is_rejected() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let txt = opened(&workspace, "Nota.txt", b"text");

        let rejected = [-0.1, 1.5, f64::NAN]
            .map(|fraction| workspace.save_position(txt, ReadingPosition::Scroll { fraction }))
            .map(|result| matches!(result, Err(Error::InvalidPosition)));

        assert_eq!(rejected, [true, true, true]);
    }

    #[test]
    fn page_zero_is_rejected() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let pdf = opened(&workspace, "Livro.pdf", b"%PDF-1.7\n");

        let result = workspace.save_position(pdf, ReadingPosition::Page { page: 0 });

        assert!(matches!(result, Err(Error::InvalidPosition)));
    }

    #[test]
    fn reading_position_serializes_with_kind_tag() {
        let page = serde_json::to_value(ReadingPosition::Page { page: 3 }).unwrap();
        let scroll = serde_json::to_value(ReadingPosition::Scroll { fraction: 0.5 }).unwrap();

        assert_eq!(page, serde_json::json!({ "kind": "Page", "page": 3 }));
        assert_eq!(
            scroll,
            serde_json::json!({ "kind": "Scroll", "fraction": 0.5 })
        );
    }
}
