//! Links between a document and the documents converted from it (spec 04).
//! Spec 08 records them; this module answers about them. A link stays hidden
//! while the other side is missing (spec 05).

use rusqlite::{OptionalExtension, Row};
use uuid::Uuid;

use super::file_facts::FileFacts;
use super::index::document_row;
use super::{Document, Format};
use crate::Result;
use crate::workspace::Workspace;

/// One side of a conversion link, seen from the other side.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ConversionLink {
    /// The source (when asking where a document came from) or the
    /// derivative (when asking for a document's derivatives).
    pub document: Document,
    pub source_format: Format,
    pub target_format: Format,
    pub converted_at: i64,
    /// The source changed after the conversion: the derivative may be stale.
    pub source_changed: bool,
    /// The derivative changed after the conversion: it is no longer the exact
    /// result of it.
    pub derivative_changed: bool,
}

impl Workspace {
    /// Records that `target` was just converted from `source`, with both
    /// files' mtimes as they are now. The derivative is already on disk, so
    /// a failure here is only logged (spec 01, guarantee 5).
    pub(crate) fn record_conversion(&self, source: &Document, target: &Document) {
        let id = self.ids.new_id().to_string();
        let converted_at = self.clock.now_millis();
        self.update_index_after_disk(|conn| {
            conn.execute(
                "INSERT INTO conversions VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                (
                    id,
                    source.id.to_string(),
                    target.id.to_string(),
                    source.format,
                    target.format,
                    converted_at,
                    source.modified_at,
                    target.modified_at,
                ),
            )
            .map(drop)
        });
    }

    /// The document `id` was converted from, if any.
    pub fn converted_from(&self, id: Uuid) -> Result<Option<ConversionLink>> {
        let sql = format!(
            "SELECT {LINK_COLUMNS} FROM conversions c JOIN documents d ON d.id = c.source_id
             WHERE c.target_id = ?1 AND d.missing_since IS NULL
             ORDER BY c.converted_at DESC LIMIT 1"
        );
        let raw = self
            .conn()
            .query_row(&sql, [id.to_string()], raw_link)
            .optional()
            .map_err(|e| self.db_error(e))?;
        let Some(raw) = raw else {
            return Ok(None);
        };
        let (source, derivative) = (raw.other.clone(), self.document(id)?);
        Ok(Some(self.link(raw, &source, &derivative)))
    }

    /// The documents converted from `id`, oldest conversion first.
    pub fn derivatives(&self, id: Uuid) -> Result<Vec<ConversionLink>> {
        let sql = format!(
            "SELECT {LINK_COLUMNS} FROM conversions c JOIN documents d ON d.id = c.target_id
             WHERE c.source_id = ?1 AND d.missing_since IS NULL ORDER BY c.converted_at"
        );
        let conn = self.conn();
        let query = || -> rusqlite::Result<Vec<RawLink>> {
            conn.prepare(&sql)?
                .query_map([id.to_string()], raw_link)?
                .collect()
        };
        let raws = query().map_err(|e| self.db_error(e))?;
        drop(conn);
        let source = self.document(id)?;
        let links = raws.into_iter().map(|raw| {
            let derivative = raw.other.clone();
            self.link(raw, &source, &derivative)
        });
        Ok(links.collect())
    }

    /// Builds the link, comparing each file's mtime on disk now with the one
    /// recorded right after the conversion.
    fn link(&self, raw: RawLink, source: &Document, derivative: &Document) -> ConversionLink {
        let changed = |doc: &Document, recorded: i64| self.mtime_on_disk(doc) != Some(recorded);
        ConversionLink {
            source_changed: changed(source, raw.source_modified_at),
            derivative_changed: changed(derivative, raw.target_modified_at),
            document: raw.other,
            source_format: raw.source_format,
            target_format: raw.target_format,
            converted_at: raw.converted_at,
        }
    }

    fn mtime_on_disk(&self, doc: &Document) -> Option<i64> {
        FileFacts::read(&doc.path.resolve(self.root()))
            .ok()
            .map(|facts| facts.modified_at)
    }
}

/// A row of `conversions` joined with the document on the other side.
struct RawLink {
    other: Document,
    source_format: Format,
    target_format: Format,
    converted_at: i64,
    source_modified_at: i64,
    target_modified_at: i64,
}

/// Document columns of `d` (read by `document_row`), then the link's own.
const LINK_COLUMNS: &str = "d.id, d.path, d.format, d.valid, d.protected, d.size,
     d.modified_at, d.created_at, c.source_format, c.target_format, c.converted_at,
     c.source_modified_at, c.target_modified_at";

fn raw_link(row: &Row<'_>) -> rusqlite::Result<RawLink> {
    Ok(RawLink {
        other: document_row(row)?,
        source_format: row.get(8)?,
        target_format: row.get(9)?,
        converted_at: row.get(10)?,
        source_modified_at: row.get(11)?,
        target_modified_at: row.get(12)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::test_docs::{MTIME, write_doc};
    use crate::workspace::test_env::TestEnv;

    fn indexed(workspace: &Workspace, path: &str, content: &[u8]) -> Uuid {
        let path = write_doc(workspace, path, content);
        workspace.index_file(&path).unwrap().unwrap().id
    }

    /// Records `source` → `target` as spec 08 does, with both files as they
    /// are now.
    fn record(workspace: &Workspace, source: Uuid, target: Uuid, converted_at: i64) {
        let (s, t) = (
            workspace.document(source).unwrap(),
            workspace.document(target).unwrap(),
        );
        workspace
            .conn()
            .execute(
                "INSERT INTO conversions VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                (
                    workspace.ids.new_id().to_string(),
                    source.to_string(),
                    target.to_string(),
                    s.format,
                    t.format,
                    converted_at,
                    s.modified_at,
                    t.modified_at,
                ),
            )
            .unwrap();
    }

    #[test]
    fn document_reports_its_source_with_format_and_date() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let pdf = indexed(&workspace, "Trabalho.pdf", b"%PDF-1.7\n");
        let md = indexed(&workspace, "Trabalho.md", b"# Trabalho");
        record(&workspace, pdf, md, 5_000);

        let link = workspace.converted_from(md).unwrap().unwrap();

        assert_eq!(
            (link.document.id, link.source_format, link.target_format),
            (pdf, Format::Pdf, Format::Md)
        );
        assert_eq!(
            (
                link.converted_at,
                link.source_changed,
                link.derivative_changed
            ),
            (5_000, false, false)
        );
        assert_eq!(link.document.modified_at, MTIME);
    }

    #[test]
    fn document_reports_all_its_derivatives() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let pdf = indexed(&workspace, "Trabalho.pdf", b"%PDF-1.7\n");
        let md = indexed(&workspace, "Trabalho.md", b"# Trabalho");
        let txt = indexed(&workspace, "Trabalho.txt", b"Trabalho");
        record(&workspace, pdf, md, 5_000);
        record(&workspace, pdf, txt, 6_000);

        let derivatives = workspace.derivatives(pdf).unwrap();

        let ids: Vec<_> = derivatives.iter().map(|link| link.document.id).collect();
        assert_eq!(ids, [md, txt]);
    }

    /// A converted pair, `Trabalho.pdf` → `Trabalho.md`, recorded at 5000.
    fn converted_pair(workspace: &Workspace) -> (Uuid, Uuid) {
        let pdf = indexed(workspace, "Trabalho.pdf", b"%PDF-1.7\n");
        let md = indexed(workspace, "Trabalho.md", b"# Trabalho");
        record(workspace, pdf, md, 5_000);
        (pdf, md)
    }

    /// Edits a file as a user would: new content and a later mtime.
    fn edit(workspace: &Workspace, path: &str) {
        let full = workspace.root().join(path);
        std::fs::write(&full, b"editado").unwrap();
        let later = std::time::SystemTime::UNIX_EPOCH
            + std::time::Duration::from_millis(MTIME as u64 + 60_000);
        std::fs::File::options()
            .write(true)
            .open(&full)
            .unwrap()
            .set_modified(later)
            .unwrap();
    }

    #[test]
    fn editing_source_after_conversion_flags_source_changed() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let (_pdf, md) = converted_pair(&workspace);
        edit(&workspace, "Trabalho.pdf");

        let link = workspace.converted_from(md).unwrap().unwrap();

        assert_eq!(
            (link.source_changed, link.derivative_changed),
            (true, false)
        );
    }

    #[test]
    fn editing_derivative_after_conversion_flags_derivative_changed() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let (pdf, _md) = converted_pair(&workspace);
        edit(&workspace, "Trabalho.md");

        let links = workspace.derivatives(pdf).unwrap();

        assert_eq!(
            (links[0].source_changed, links[0].derivative_changed),
            (false, true)
        );
    }

    #[test]
    fn relation_survives_renaming_either_side() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let (pdf, md) = converted_pair(&workspace);

        workspace.rename_document(pdf, "Original").unwrap();
        workspace.rename_document(md, "Convertido").unwrap();

        let link = workspace.converted_from(md).unwrap().unwrap();
        assert_eq!(
            (link.document.id, link.document.name.as_str()),
            (pdf, "Original.pdf")
        );
    }

    #[test]
    fn relation_is_hidden_while_one_side_is_missing() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let (pdf, md) = converted_pair(&workspace);

        workspace.trash_document(pdf).unwrap();

        assert_eq!(workspace.converted_from(md).unwrap(), None);
    }
}
