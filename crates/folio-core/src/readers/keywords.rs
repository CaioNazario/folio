//! Keywords as DOCX and PDF store them: one text, separated by commas or
//! semicolons.

/// The keywords in `text`, trimmed, without empty ones.
pub fn split_keywords(text: Option<String>) -> Vec<String> {
    let text = text.unwrap_or_default();
    let words = text.split([',', ';']).map(str::trim);
    words
        .filter(|word| !word.is_empty())
        .map(str::to_string)
        .collect()
}
