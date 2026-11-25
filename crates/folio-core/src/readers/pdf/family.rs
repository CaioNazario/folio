//! The family of a font, from the name a PDF gives it: `ABCDEF+ArialMT`
//! and `TimesNewRomanPS-BoldMT` are Arial and Times New Roman.

/// The family `base_font` names, or `None` when it cannot be told.
pub fn family_of(base_font: &str) -> Option<String> {
    let name = without_postscript(without_style(without_subset(base_font)));
    is_word_like(name).then(|| split_camel_case(name))
}

/// Whether `name` looks like words: letters only, starting with a capital
/// and with some lowercase, which an abbreviation like `CMR10` lacks.
fn is_word_like(name: &str) -> bool {
    name.starts_with(|c: char| c.is_uppercase())
        && name.chars().all(char::is_alphabetic)
        && name.chars().any(char::is_lowercase)
}

/// The suffixes PostScript names add to the family, longest first.
const POSTSCRIPT_SUFFIXES: [&str; 3] = ["PSMT", "MT", "PS"];

/// `name` without a PostScript suffix, as in `ArialMT`.
fn without_postscript(name: &str) -> &str {
    POSTSCRIPT_SUFFIXES
        .iter()
        .find_map(|suffix| name.strip_suffix(suffix))
        .unwrap_or(name)
}

/// `name` without the style after a hyphen or a comma, as in
/// `Arial-BoldMT` or `Calibri,Bold`.
fn without_style(name: &str) -> &str {
    name.split([',', '-']).next().unwrap_or(name)
}

/// `name` without the tag of a font subset: six capitals and a plus.
fn without_subset(name: &str) -> &str {
    match name.split_once('+') {
        Some((tag, rest)) if tag.len() == 6 && tag.chars().all(|c| c.is_ascii_uppercase()) => rest,
        _ => name,
    }
}

/// `name` with a space before each capital that follows a lowercase letter.
fn split_camel_case(name: &str) -> String {
    let mut words = String::new();
    let mut previous = ' ';
    for c in name.chars() {
        if c.is_uppercase() && previous.is_lowercase() {
            words.push(' ');
        }
        words.push(c);
        previous = c;
    }
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    fn family(name: &str) -> Option<String> {
        family_of(name)
    }

    #[test]
    fn normalizes_family_splits_camel_case() {
        assert_eq!(
            family("LiberationSerif").as_deref(),
            Some("Liberation Serif")
        );
    }

    #[test]
    fn normalizes_family_strips_subset_prefix() {
        assert_eq!(
            family("BAAAAA+LiberationSerif").as_deref(),
            Some("Liberation Serif")
        );
    }

    #[test]
    fn normalizes_family_strips_comma_style_suffix() {
        assert_eq!(family("Calibri,Bold").as_deref(), Some("Calibri"));
        assert_eq!(
            family("LiberationSans-BoldItalic").as_deref(),
            Some("Liberation Sans")
        );
    }

    #[test]
    fn normalizes_family_strips_postscript_suffix() {
        assert_eq!(
            family("TimesNewRomanPSMT").as_deref(),
            Some("Times New Roman")
        );
        assert_eq!(family("ArialMT").as_deref(), Some("Arial"));
        assert_eq!(
            family("TimesNewRomanPS-BoldMT").as_deref(),
            Some("Times New Roman")
        );
    }

    #[test]
    fn normalizes_family_returns_none_for_unknown_name() {
        assert_eq!(family("ABCDEF+CMR10"), None);
        assert_eq!(family(""), None);
    }
}
