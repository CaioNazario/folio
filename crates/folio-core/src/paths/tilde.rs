//! Paths as the user reads them, with the home shown as `~`.

use std::path::Path;

/// `path` with the `home` prefix replaced by `~` ("~/Documentos").
pub fn tilde_path(path: &Path, home: &Path) -> String {
    let Ok(rest) = path.strip_prefix(home) else {
        return path.display().to_string();
    };
    if rest.as_os_str().is_empty() {
        return "~".to_string();
    }
    format!("~/{}", rest.display())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tilde_path_replaces_home_prefix() {
        let home = Path::new("/home/ana");

        assert_eq!(
            (
                tilde_path(Path::new("/home/ana/Documentos/Folio"), home),
                tilde_path(home, home),
            ),
            ("~/Documentos/Folio".to_string(), "~".to_string())
        );
    }

    #[test]
    fn tilde_path_keeps_paths_outside_home() {
        let home = Path::new("/home/ana");

        assert_eq!(
            (
                tilde_path(Path::new("/media/pendrive/Folio"), home),
                tilde_path(Path::new("/home/anabela/Folio"), home),
            ),
            (
                "/media/pendrive/Folio".to_string(),
                "/home/anabela/Folio".to_string()
            )
        );
    }
}
