//! Addresses written inside a document, resolved against its directory
//! (spec 07, "Links" and "Imagens no Markdown").

use percent_encoding::percent_decode_str;

use super::RelativePath;

/// The workspace path `href` points to, relative to the directory `base`
/// (the root for `None`). `None` when it is absolute, leaves the workspace
/// or is not a valid workspace path. `%` escapes are decoded.
pub fn resolve_href(base: Option<&RelativePath>, href: &str) -> Option<RelativePath> {
    let decoded: Vec<u8> = percent_decode_str(href).collect();
    if decoded.starts_with(b"/") {
        return None;
    }
    let mut parts: Vec<&[u8]> = base.map_or_else(Vec::new, |dir| {
        dir.as_bytes().split(|b| *b == b'/').collect()
    });
    for part in decoded.split(|byte| *byte == b'/') {
        match part {
            b"" | b"." => {}
            b".." => _ = parts.pop()?,
            name => parts.push(name),
        }
    }
    RelativePath::from_bytes(parts.join(&b'/')).ok()
}

/// The scheme of `href` in lowercase (`https`, `mailto`...), or `None` for
/// a relative address.
pub fn href_scheme(href: &str) -> Option<String> {
    let (scheme, _) = href.split_once(':')?;
    let mut chars = scheme.chars();
    let starts_with_letter = chars.next().is_some_and(|c| c.is_ascii_alphabetic());
    let rest_valid = chars.all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c));
    (starts_with_letter && rest_valid).then(|| scheme.to_ascii_lowercase())
}
