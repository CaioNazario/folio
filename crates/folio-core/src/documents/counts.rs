//! Counts the UI shows, answered from the index (spec 04). Invalid documents
//! count: they appear in the interface.

use crate::Result;
use crate::paths::RelativePath;
use crate::workspace::Workspace;

/// Documents of the whole workspace, for the sidebar footer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceTotals {
    pub documents: u64,
    /// Sum of the file sizes, in bytes.
    pub size: u64,
}

impl Workspace {
    /// Documents in the project or folder `dir`, counting every folder inside.
    pub fn document_count(&self, dir: &RelativePath) -> Result<u64> {
        let (lower, upper) = dir.descendant_bounds();
        let count: i64 = self
            .conn()
            .query_row(
                "SELECT count(*) FROM documents
                 WHERE missing_since IS NULL AND path > ?1 AND path < ?2",
                (lower, upper),
                |row| row.get(0),
            )
            .map_err(|e| self.db_error(e))?;
        Ok(count as u64)
    }

    /// Count and total size of all documents in the workspace.
    pub fn workspace_totals(&self) -> Result<WorkspaceTotals> {
        let (documents, size): (i64, i64) = self
            .conn()
            .query_row(
                "SELECT count(*), coalesce(sum(size), 0) FROM documents
                 WHERE missing_since IS NULL",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| self.db_error(e))?;
        Ok(WorkspaceTotals {
            documents: documents as u64,
            size: size as u64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::test_docs::write_doc;
    use crate::workspace::test_env::TestEnv;

    fn index_all(workspace: &Workspace, paths: &[&str], content: &[u8]) {
        for path in paths {
            let path = write_doc(workspace, path, content);
            workspace.index_file(&path).unwrap();
        }
    }

    fn rel(text: &str) -> RelativePath {
        RelativePath::from_bytes(text).unwrap()
    }

    #[test]
    fn project_count_includes_documents_in_its_folders() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let paths = [
            "Fac/a.md",
            "Fac/TCC/b.md",
            "Fac/TCC/Fontes/c.md",
            "Faculdade/d.md",
            "e.md",
        ];
        index_all(&workspace, &paths, b"texto");

        assert_eq!(workspace.document_count(&rel("Fac")).unwrap(), 3);
    }

    #[test]
    fn workspace_totals_count_and_size_including_invalid() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        index_all(&workspace, &["leia-me.txt", "Fac/a.md"], b"12345");
        index_all(&workspace, &["Fac/quebrado.pdf"], b"<html>");

        let totals = workspace.workspace_totals().unwrap();

        assert_eq!(
            totals,
            WorkspaceTotals {
                documents: 3,
                size: 16
            }
        );
    }

    #[test]
    fn counts_exclude_missing_documents() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        index_all(&workspace, &["Fac/a.md", "Fac/b.md"], b"12345");
        let gone = workspace.list_documents(Some(&rel("Fac"))).unwrap()[0].id;

        workspace.trash_document(gone).unwrap();

        assert_eq!(workspace.document_count(&rel("Fac")).unwrap(), 1);
        assert_eq!(workspace.workspace_totals().unwrap().documents, 1);
    }
}
