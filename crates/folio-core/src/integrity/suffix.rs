//! Collision suffix: `Report.docx` becomes `Report (1).docx`.

use std::ffi::{OsStr, OsString};
use std::path::Path;

/// Returns `name` with ` (n)` inserted before its last extension.
pub fn with_suffix(name: &OsStr, n: u32) -> OsString {
    let path = Path::new(name);
    let stem = path.file_stem().unwrap_or(name);
    let mut result = stem.to_os_string();
    result.push(format!(" ({n})"));
    if let Some(extension) = path.extension() {
        result.push(".");
        result.push(extension);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suffixed(name: &str, n: u32) -> OsString {
        with_suffix(OsStr::new(name), n)
    }

    #[test]
    fn suffix_goes_before_extension() {
        assert_eq!(suffixed("Trabalho.docx", 1), "Trabalho (1).docx");
    }

    #[test]
    fn suffix_goes_at_end_when_there_is_no_extension() {
        assert_eq!(suffixed("Notas", 2), "Notas (2)");
    }

    #[test]
    fn suffix_uses_only_last_extension() {
        assert_eq!(suffixed("dados.tar.gz", 1), "dados.tar (1).gz");
    }

    #[test]
    fn suffix_treats_hidden_file_as_having_no_extension() {
        assert_eq!(suffixed(".env", 1), ".env (1)");
    }

    #[cfg(unix)]
    #[test]
    fn suffix_preserves_non_utf8_name_bytes() {
        use std::os::unix::ffi::OsStrExt;
        let name = OsStr::from_bytes(b"relat\xf3rio.txt");
        let expected = OsStr::from_bytes(b"relat\xf3rio (1).txt");
        assert_eq!(with_suffix(name, 1), expected);
    }
}
