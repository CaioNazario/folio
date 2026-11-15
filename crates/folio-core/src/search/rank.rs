//! Ranking of search results (spec 06, "Ordem dos resultados").

/// True when a word of `name` other than the first starts with `prefix`. A
/// word starts after any character that is neither a letter nor a digit.
pub fn has_word_starting_with(name: &str, prefix: &str) -> bool {
    name.match_indices(prefix)
        .any(|(at, _)| at > 0 && starts_word(name, at))
}

/// True when the character before byte `at` of `name` separates words.
fn starts_word(name: &str, at: usize) -> bool {
    name[..at]
        .chars()
        .next_back()
        .is_some_and(|before| !before.is_alphanumeric())
}
