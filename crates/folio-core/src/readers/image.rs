//! Images a reader or the display embeds, recognized by their signature.

/// The MIME type of an image Folio embeds, by its signature: PNG, JPEG,
/// GIF, WebP or SVG.
pub fn image_mime(bytes: &[u8]) -> Option<&'static str> {
    let signatures: [(&[u8], usize, &str); 5] = [
        (b"\x89PNG\r\n\x1a\n", 0, "image/png"),
        (b"\xFF\xD8\xFF", 0, "image/jpeg"),
        (b"GIF8", 0, "image/gif"),
        (b"WEBP", 8, "image/webp"),
        (b"<svg", 0, "image/svg+xml"),
    ];
    signatures
        .iter()
        .find(|(magic, at, _)| bytes.get(*at..).is_some_and(|rest| rest.starts_with(magic)))
        .map(|(_, _, mime)| *mime)
}
