//! Decoding plain text files (spec 07, leitor de TXT).

use encoding_rs::{Encoding, WINDOWS_1252};

/// The text in `bytes`: by its byte order mark when there is one (UTF-8,
/// UTF-16 LE or BE), else UTF-8, falling back to Windows-1252.
pub fn decode_text(bytes: &[u8]) -> String {
    decode(bytes, true)
}

/// Like [`decode_text`], for the start of a longer file: a character cut
/// at the end is dropped instead of making the text look like it is not
/// UTF-8.
pub fn decode_text_start(bytes: &[u8]) -> String {
    decode(bytes, false)
}

fn decode(bytes: &[u8], complete: bool) -> String {
    if let Some((encoding, bom)) = Encoding::for_bom(bytes) {
        let (text, _) = encoding.decode_without_bom_handling(&bytes[bom..]);
        return text.into_owned();
    }
    utf8_text(bytes, complete).unwrap_or_else(|| {
        WINDOWS_1252
            .decode_without_bom_handling(bytes)
            .0
            .into_owned()
    })
}

/// `bytes` as UTF-8, or `None` when they are not.
fn utf8_text(bytes: &[u8], complete: bool) -> Option<String> {
    let valid = match std::str::from_utf8(bytes) {
        Ok(text) => return Some(text.to_string()),
        // `error_len` is `None` only for a sequence cut by the end.
        Err(e) if !complete && e.error_len().is_none() => e.valid_up_to(),
        Err(_) => return None,
    };
    std::str::from_utf8(&bytes[..valid])
        .ok()
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_reads_utf8_without_bom() {
        assert_eq!(decode_text("Relatório".as_bytes()), "Relatório");
    }

    #[test]
    fn decode_strips_utf8_bom() {
        let bytes = [b"\xEF\xBB\xBF".as_slice(), "Notas".as_bytes()].concat();

        assert_eq!(decode_text(&bytes), "Notas");
    }

    fn utf16(bom: [u8; 2], text: &str, to_bytes: fn(u16) -> [u8; 2]) -> Vec<u8> {
        let units = text.encode_utf16().flat_map(to_bytes);
        bom.into_iter().chain(units).collect()
    }

    #[test]
    fn decode_reads_utf16_le_with_bom() {
        let bytes = utf16([0xFF, 0xFE], "Ação", u16::to_le_bytes);

        assert_eq!(decode_text(&bytes), "Ação");
    }

    #[test]
    fn decode_reads_utf16_be_with_bom() {
        let bytes = utf16([0xFE, 0xFF], "Ação", u16::to_be_bytes);

        assert_eq!(decode_text(&bytes), "Ação");
    }

    #[test]
    fn decode_falls_back_to_windows_1252_when_not_utf8() {
        // "Ação – €5" in Windows-1252: ç = E7, ã = E3, en dash = 96, € = 80.
        let bytes = b"A\xE7\xE3o \x96 \x805";

        assert_eq!(decode_text(bytes), "Ação – €5");
    }
}
