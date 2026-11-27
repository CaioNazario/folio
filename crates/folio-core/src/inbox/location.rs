//! Where the inbox folder is (spec 09, "Pasta da Entrada").

use std::path::{Path, PathBuf};

use crate::Result;
use crate::db::GlobalDb;
use crate::paths::os_bytes;

/// The preference with the inbox folder: the exact bytes of its path, as a
/// JSON array, so a name that is not UTF-8 survives (spec 02).
const INBOX_LOCATION: &str = "inbox.location";

impl GlobalDb {
    /// The inbox folder the user chose, or `default` before any choice. A
    /// stored value that cannot be read counts as no choice.
    pub fn inbox_location(&self, default: &Path) -> Result<PathBuf> {
        let stored = self.preference(INBOX_LOCATION)?;
        let bytes = stored.and_then(|value| serde_json::from_value::<Vec<u8>>(value).ok());
        Ok(bytes.map_or_else(
            || default.to_path_buf(),
            |bytes| PathBuf::from(os_bytes::from_bytes(&bytes)),
        ))
    }

    /// Stores `dir` as the inbox folder.
    pub fn set_inbox_location(&self, dir: &Path) -> Result<()> {
        let bytes = os_bytes::as_bytes(dir.as_os_str());
        self.set_preference(INBOX_LOCATION, &serde_json::json!(bytes))
    }
}

/// The inbox folder on first run: the system downloads dir
/// (`XDG_DOWNLOAD_DIR`, which follows the system language), or
/// `~/Downloads` without it.
pub fn default_inbox_dir(download_dir: Option<&Path>, home: &Path) -> PathBuf {
    download_dir.map_or_else(|| home.join("Downloads"), Path::to_path_buf)
}

/// Whether the inbox leaves the sidebar: its folder is in the open
/// workspace, whose documents are already shown there.
pub fn inbox_is_hidden(inbox: &Path, workspace_root: &Path) -> bool {
    // By components: "Folio Downloads" is not inside "Folio".
    inbox.starts_with(workspace_root)
}

#[cfg(test)]
mod tests {
    use std::os::unix::ffi::OsStrExt;

    use super::*;

    #[test]
    fn default_inbox_dir_is_the_system_download_dir() {
        let dir = default_inbox_dir(
            Some(Path::new("/home/ana/Transferências")),
            Path::new("/home/ana"),
        );

        assert_eq!(dir, Path::new("/home/ana/Transferências"));
    }

    #[test]
    fn default_inbox_dir_falls_back_to_downloads_in_home() {
        let dir = default_inbox_dir(None, Path::new("/home/ana"));

        assert_eq!(dir, Path::new("/home/ana/Downloads"));
    }

    fn open(dir: &tempfile::TempDir) -> GlobalDb {
        let clock = std::sync::Arc::new(crate::testing::ManualClock::at(1_000));
        let ids = std::sync::Arc::new(crate::testing::SequentialIds::default());
        GlobalDb::open(&dir.path().join("folio.db"), clock, ids).unwrap()
    }

    #[test]
    fn inbox_location_is_the_default_when_not_stored() {
        let dir = tempfile::tempdir().unwrap();

        let location = open(&dir).inbox_location(Path::new("/home/ana/Downloads"));

        assert_eq!(location.unwrap(), Path::new("/home/ana/Downloads"));
    }

    #[test]
    fn set_inbox_location_is_kept_in_the_global_db() {
        let dir = tempfile::tempdir().unwrap();
        // Not UTF-8: the exact bytes must survive (spec 02, `Location`).
        let chosen = Path::new(std::ffi::OsStr::from_bytes(
            b"/media/ana/Pendrive/baixados\xe7",
        ));
        open(&dir).set_inbox_location(chosen).unwrap();

        let location = open(&dir).inbox_location(Path::new("/home/ana/Downloads"));

        assert_eq!(location.unwrap(), chosen);
    }

    #[test]
    fn inbox_is_hidden_when_inside_the_open_workspace() {
        let hidden = inbox_is_hidden(
            Path::new("/home/ana/Folio/Downloads"),
            Path::new("/home/ana/Folio"),
        );

        assert!(hidden);
    }

    #[test]
    fn inbox_is_hidden_when_it_is_the_workspace_root() {
        let hidden = inbox_is_hidden(Path::new("/home/ana/Folio"), Path::new("/home/ana/Folio"));

        assert!(hidden);
    }

    #[test]
    fn inbox_is_shown_when_outside_the_open_workspace() {
        // A sibling whose name starts like the workspace's is still outside.
        let hidden = inbox_is_hidden(
            Path::new("/home/ana/Folio Downloads"),
            Path::new("/home/ana/Folio"),
        );

        assert!(!hidden);
    }
}
