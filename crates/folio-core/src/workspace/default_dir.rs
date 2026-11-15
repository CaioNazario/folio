//! The directory pre-selected on first run (spec 03).

use std::path::{Path, PathBuf};

/// `Folio/` inside the system documents dir (`XDG_DOCUMENTS_DIR`, which
/// follows the system language), or `~/Documents/Folio` without it.
pub fn default_workspace_dir(documents_dir: Option<&Path>, home: &Path) -> PathBuf {
    let documents = documents_dir.map_or_else(|| home.join("Documents"), Path::to_path_buf);
    documents.join("Folio")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_workspace_dir_is_folio_inside_documents_dir() {
        let dir = default_workspace_dir(
            Some(Path::new("/home/caio/Documentos")),
            Path::new("/home/caio"),
        );

        assert_eq!(dir, Path::new("/home/caio/Documentos/Folio"));
    }

    #[test]
    fn default_workspace_dir_falls_back_to_home_documents() {
        let dir = default_workspace_dir(None, Path::new("/home/caio"));

        assert_eq!(dir, Path::new("/home/caio/Documents/Folio"));
    }
}
