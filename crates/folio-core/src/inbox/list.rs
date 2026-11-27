//! Listing the inbox folder (spec 09, "O que aparece").

use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use crate::documents::file_facts::FileFacts;
use crate::documents::index::document_format;
use crate::documents::{Format, check_signature};
use crate::paths::os_bytes;
use crate::{Error, Result};

/// A document in the inbox folder. It is in no index and has no id: the
/// UI names it back to the core by its file name, unique in the folder.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct InboxDocument {
    /// The exact bytes of the file name, to name it back to the core.
    pub file_name: Vec<u8>,
    /// File name for the UI; invalid UTF-8 bytes show as U+FFFD.
    pub name: String,
    pub format: Format,
    pub valid: bool,
    pub protected: bool,
    pub size: u64,
    /// Filesystem mtime, in ms since the Unix epoch.
    pub modified_at: i64,
}

/// The documents directly inside the inbox folder `dir`.
pub fn list_inbox(dir: &Path) -> Result<Vec<InboxDocument>> {
    let entries = fs::read_dir(dir).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => Error::InboxNotFound(dir.to_path_buf()),
        _ => Error::io(dir, e),
    })?;
    let mut documents = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| Error::io(dir, e))?;
        let path = entry.path();
        if path.is_dir() || is_being_downloaded(&path) {
            continue;
        }
        documents.extend(inbox_document(&path, &entry.file_name())?);
    }
    // What just finished downloading is what the user looks for.
    documents.sort_by_key(|doc| std::cmp::Reverse(doc.modified_at));
    Ok(documents)
}

/// Extensions of the partial file a browser downloads into while the final
/// file already exists: Firefox's and Chromium's.
const PARTIAL_BESIDE: [&str; 2] = [".part", ".crdownload"];

/// Whether the browser is still downloading into `path`: it has a partial
/// file beside it. Firefox creates the final file empty at the start.
fn is_being_downloaded(path: &Path) -> bool {
    PARTIAL_BESIDE.iter().any(|extension| {
        let mut partial = path.as_os_str().to_owned();
        partial.push(extension);
        fs::symlink_metadata(partial).is_ok()
    })
}

/// The file at `path` as an inbox document; `None` when it is not one.
fn inbox_document(path: &Path, file_name: &OsStr) -> Result<Option<InboxDocument>> {
    let Some(format) = document_format(file_name) else {
        return Ok(None);
    };
    let signature = check_signature(format, path);
    let facts = FileFacts::read(path)?;
    Ok(Some(InboxDocument {
        file_name: os_bytes::as_bytes(file_name).to_vec(),
        name: file_name.to_string_lossy().into_owned(),
        format,
        valid: signature.valid,
        protected: signature.protected,
        size: facts.size,
        modified_at: facts.modified_at,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The names `list_inbox` shows for `dir`, in order.
    fn names(dir: &Path) -> Vec<String> {
        list_inbox(dir)
            .unwrap()
            .into_iter()
            .map(|doc| doc.name)
            .collect()
    }

    /// Writes `content` to `dir/name`, modified `secs` after the epoch.
    fn write_at(dir: &Path, name: &str, content: &[u8], secs: u64) {
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
        let mtime = std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs);
        let file = fs::File::options().write(true).open(&path).unwrap();
        file.set_modified(mtime).unwrap();
    }

    #[test]
    fn list_inbox_orders_newest_first() {
        let dir = tempfile::tempdir().unwrap();
        write_at(dir.path(), "antigo.txt", b"a", 1_000);
        write_at(dir.path(), "novo.txt", b"b", 3_000);
        write_at(dir.path(), "meio.txt", b"c", 2_000);

        assert_eq!(names(dir.path()), ["novo.txt", "meio.txt", "antigo.txt"]);
    }

    #[test]
    fn list_inbox_skips_non_documents_hidden_files_and_locks() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["foto.jpg", "setup.zip", ".oculto.txt", "~$Trabalho.docx"] {
            fs::write(dir.path().join(name), "x").unwrap();
        }
        fs::write(dir.path().join(".~lock.Trabalho.docx#"), "x").unwrap();
        fs::write(dir.path().join("Aula.txt"), "aula").unwrap();

        assert_eq!(names(dir.path()), ["Aula.txt"]);
    }

    /// The single document `list_inbox` shows for `dir`.
    fn only(dir: &Path) -> InboxDocument {
        let mut documents = list_inbox(dir).unwrap();
        assert_eq!(documents.len(), 1, "{documents:?}");
        documents.remove(0)
    }

    #[test]
    fn list_inbox_shows_invalid_document_as_invalid() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("falso.pdf"), "não sou um PDF").unwrap();

        let doc = only(dir.path());

        assert_eq!((doc.name.as_str(), doc.valid), ("falso.pdf", false));
    }

    #[test]
    fn list_inbox_marks_protected_document() {
        let dir = tempfile::tempdir().unwrap();
        let fixture = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/docx/protegido.docx"
        );
        fs::copy(fixture, dir.path().join("protegido.docx")).unwrap();

        let doc = only(dir.path());

        assert_eq!((doc.valid, doc.protected), (true, true));
    }

    #[test]
    fn list_inbox_follows_symlink_to_file() {
        let dir = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        fs::write(elsewhere.path().join("real.txt"), "doze bytes!!").unwrap();
        std::os::unix::fs::symlink(
            elsewhere.path().join("real.txt"),
            dir.path().join("link.txt"),
        )
        .unwrap();
        std::os::unix::fs::symlink(elsewhere.path(), dir.path().join("pasta.txt")).unwrap();

        let doc = only(dir.path());

        assert_eq!((doc.name.as_str(), doc.size), ("link.txt", 12));
    }

    #[test]
    fn list_inbox_returns_documents_of_first_level_only() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Aula.txt"), "aula").unwrap();
        fs::create_dir(dir.path().join("extraido")).unwrap();
        fs::write(dir.path().join("extraido/Notas.md"), "# notas").unwrap();

        assert_eq!(names(dir.path()), ["Aula.txt"]);
    }

    #[test]
    fn list_inbox_hides_in_progress_download_extensions() {
        let dir = tempfile::tempdir().unwrap();
        let in_progress = [
            "a.pdf.part",
            "b.pdf.crdownload",
            "c.pdf.download",
            "d.pdf.partial",
            "e.pdf.opdownload",
        ];
        for name in in_progress {
            fs::write(dir.path().join(name), "%PDF-1.7").unwrap();
        }

        assert_eq!(names(dir.path()), Vec::<String>::new());
    }

    #[test]
    fn list_inbox_hides_final_file_while_part_exists() {
        let dir = tempfile::tempdir().unwrap();
        // Firefox creates the final file empty before the download ends.
        fs::write(dir.path().join("Edital.pdf"), "").unwrap();
        fs::write(dir.path().join("Edital.pdf.part"), "%PDF-1.7").unwrap();

        assert_eq!(names(dir.path()), Vec::<String>::new());
    }

    #[test]
    fn list_inbox_hides_final_file_while_crdownload_exists() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Edital.pdf"), "").unwrap();
        fs::write(dir.path().join("Edital.pdf.crdownload"), "%PDF-1.7").unwrap();

        assert_eq!(names(dir.path()), Vec::<String>::new());
    }

    #[test]
    fn list_inbox_writes_nothing_into_the_folder() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Aula.txt"), "aula").unwrap();
        fs::write(dir.path().join("falso.pdf"), "x").unwrap();
        fs::create_dir(dir.path().join("extraido")).unwrap();
        let before = crate::integrity::test_support::snapshot(dir.path());

        list_inbox(dir.path()).unwrap();

        assert_eq!(crate::integrity::test_support::snapshot(dir.path()), before);
        assert!(!dir.path().join(".folio").exists());
    }

    #[test]
    fn list_inbox_fails_with_inbox_not_found_when_folder_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("Downloads");

        let error = list_inbox(&missing).unwrap_err();

        assert!(
            matches!(&error, Error::InboxNotFound(path) if *path == missing),
            "{error:?}"
        );
    }
}
