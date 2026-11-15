//! The normalized form names are compared in (spec 06, "Normalização").

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use crate::paths::RelativePath;

/// `text` without combining marks and in lowercase, so "relatorio" matches
/// "Relatório".
pub fn normalize_name(text: &str) -> String {
    text.nfd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .collect()
}

/// The normalized file name of `path`: its last component, with invalid
/// UTF-8 shown as U+FFFD (spec 02).
pub fn normalize_file_name(path: &RelativePath) -> String {
    normalize_name(&path.file_name().to_string_lossy())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_removes_accents_and_lowercases() {
        assert_eq!(normalize_name("Relatório Q1.pdf"), "relatorio q1.pdf");
    }

    #[test]
    fn normalize_strips_combining_marks_from_any_letter() {
        assert_eq!(normalize_name("Çà Ñõ Üê"), "ca no ue");
    }

    #[test]
    fn normalize_uses_last_path_component() {
        let path = RelativePath::from_bytes("Faculdade/Prova/Relatório.pdf").unwrap();

        assert_eq!(normalize_file_name(&path), "relatorio.pdf");
    }

    #[test]
    fn normalize_replaces_invalid_utf8_lossily() {
        let path = RelativePath::from_bytes(b"Nota\xff.TXT".to_vec()).unwrap();

        assert_eq!(normalize_file_name(&path), "nota\u{fffd}.txt");
    }
}
