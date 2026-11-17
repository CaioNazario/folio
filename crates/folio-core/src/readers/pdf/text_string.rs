//! PDF text strings (PDF 32000, 7.9.2.2) decoded to Rust strings.

/// The byte order mark of a UTF-16BE text string.
const UTF16_BOM: &[u8] = &[0xFE, 0xFF];

/// PDFDocEncoding from 0x18 to 0x1F: accents on their own.
const ACCENTS: [char; 8] = ['˘', 'ˇ', 'ˆ', '˙', '˝', '˛', '˚', '˜'];
/// PDFDocEncoding from 0x80 to 0xA0; 0x9F is undefined.
const PUNCTUATION: [char; 33] = [
    '•', '†', '‡', '…', '—', '–', 'ƒ', '⁄', '‹', '›', '−', '‰', '„', '“', '”', '‘', '’', '‚', '™',
    'ﬁ', 'ﬂ', 'Ł', 'Œ', 'Š', 'Ÿ', 'Ž', 'ı', 'ł', 'œ', 'š', 'ž', '\u{FFFD}', '€',
];

/// The text of a PDF text string: UTF-16BE after its byte order mark,
/// PDFDocEncoding otherwise.
pub fn decode(bytes: &[u8]) -> String {
    match bytes.strip_prefix(UTF16_BOM) {
        Some(utf16) => encoding_rs::UTF_16BE
            .decode_without_bom_handling(utf16)
            .0
            .into_owned(),
        None => bytes.iter().map(|&byte| pdf_doc_char(byte)).collect(),
    }
}

/// The character of `byte` in PDFDocEncoding, which is Latin-1 but for
/// two ranges.
fn pdf_doc_char(byte: u8) -> char {
    match byte {
        0x18..=0x1F => ACCENTS[usize::from(byte - 0x18)],
        0x80..=0xA0 => PUNCTUATION[usize::from(byte - 0x80)],
        _ => char::from(byte),
    }
}
