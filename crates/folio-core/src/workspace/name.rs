//! Validation of names the user types for projects and folders.

/// Why a typed name cannot be used. The UI picks the message per variant.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidName {
    #[error("name is empty")]
    Empty,
    #[error("name contains `/` or NUL")]
    ContainsSlash,
    #[error("name is `.` or `..`")]
    Reserved,
    #[error("name starts with `.` and would be hidden")]
    Hidden,
    #[error("name is longer than 255 bytes")]
    TooLong,
}

/// Returns the name to use (trimmed), or why it is not valid.
pub fn validate_name(typed: &str) -> Result<String, InvalidName> {
    let name = typed.trim();
    match rejection(name) {
        Some(reason) => Err(reason),
        None => Ok(name.to_string()),
    }
}

/// Linux limit for one path component, in bytes.
const MAX_NAME_BYTES: usize = 255;

fn rejection(name: &str) -> Option<InvalidName> {
    if name.is_empty() {
        return Some(InvalidName::Empty);
    }
    if name.contains(['/', '\0']) {
        return Some(InvalidName::ContainsSlash);
    }
    if name == "." || name == ".." {
        return Some(InvalidName::Reserved);
    }
    if name.starts_with('.') {
        return Some(InvalidName::Hidden);
    }
    if name.len() > MAX_NAME_BYTES {
        return Some(InvalidName::TooLong);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_trimmed() {
        assert_eq!(validate_name("  Faculdade \t"), Ok("Faculdade".to_string()));
    }

    #[test]
    fn name_rejects_empty_or_blank() {
        assert_eq!(validate_name(""), Err(InvalidName::Empty));
        assert_eq!(validate_name("   "), Err(InvalidName::Empty));
    }

    #[test]
    fn name_rejects_slash_and_nul() {
        assert_eq!(validate_name("TCC/2024"), Err(InvalidName::ContainsSlash));
        assert_eq!(validate_name("TCC\0"), Err(InvalidName::ContainsSlash));
    }

    #[test]
    fn name_rejects_dot_and_dot_dot() {
        assert_eq!(validate_name("."), Err(InvalidName::Reserved));
        assert_eq!(validate_name(" .. "), Err(InvalidName::Reserved));
    }

    #[test]
    fn name_rejects_leading_dot() {
        assert_eq!(validate_name(".rascunhos"), Err(InvalidName::Hidden));
    }

    #[test]
    fn name_rejects_more_than_255_bytes() {
        let name = "ç".repeat(128);

        assert_eq!(validate_name(&name), Err(InvalidName::TooLong));
    }

    #[test]
    fn name_accepts_exactly_255_bytes() {
        let name = format!("{}a", "ç".repeat(127));

        assert_eq!(validate_name(&name), Ok(name));
    }
}
