//! One level of the sidebar tree: folders with their document counts, then
//! documents (Design spec 03, "Projetos").

use serde::Serialize;
use ts_rs::TS;

use super::{DirEntry, Workspace};
use crate::Result;
use crate::documents::Document;
use crate::paths::RelativePath;

/// What is directly inside a directory, for the tree to expand lazily.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct DirContents {
    pub folders: Vec<FolderEntry>,
    pub documents: Vec<Document>,
}

/// A project or folder with the documents it holds, subfolders included.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FolderEntry {
    #[serde(flatten)]
    #[ts(flatten)]
    pub entry: DirEntry,
    pub document_count: u64,
}

impl Workspace {
    /// The folders and documents directly inside `parent` (the root for
    /// `None`, where projects and documents without a project are).
    pub fn dir_contents(&self, parent: Option<&RelativePath>) -> Result<DirContents> {
        let folders = self
            .list_dirs(parent)?
            .into_iter()
            .map(|entry| self.folder_entry(entry))
            .collect::<Result<_>>()?;
        Ok(DirContents {
            folders,
            documents: self.list_documents(parent)?,
        })
    }

    fn folder_entry(&self, entry: DirEntry) -> Result<FolderEntry> {
        let document_count = self.document_count(&entry.path)?;
        Ok(FolderEntry {
            entry,
            document_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::test_docs::write_doc;
    use crate::workspace::test_env::TestEnv;

    fn indexed(workspace: &Workspace, paths: &[&str]) {
        for path in paths {
            let path = write_doc(workspace, path, b"text");
            workspace.index_file(&path).unwrap();
        }
    }

    fn name_and_count(folder: &FolderEntry) -> (&str, u64) {
        (&folder.entry.name, folder.document_count)
    }

    #[test]
    fn dir_contents_lists_folders_with_document_counts_and_documents() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        indexed(
            &workspace,
            &[
                "Faculdade/A.txt",
                "Faculdade/Prova/B.txt",
                "Trabalho/C.txt",
                "Solto.txt",
            ],
        );

        let contents = workspace.dir_contents(None).unwrap();

        let folders: Vec<_> = contents.folders.iter().map(name_and_count).collect();
        let documents: Vec<_> = contents.documents.iter().map(|doc| &doc.name).collect();
        assert_eq!(folders, [("Faculdade", 2), ("Trabalho", 1)]);
        assert_eq!(documents, ["Solto.txt"]);
    }
}
