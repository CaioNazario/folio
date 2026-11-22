//! The pixel size of an image, read from its file header: Word needs it to
//! place the image (spec 08, "Escritores").

/// The width and height of the PNG, JPEG, GIF or WebP image in `bytes`, in
/// pixels; `None` for other formats and for a header cut short.
pub fn image_size(bytes: &[u8]) -> Option<(u32, u32)> {
    let (_, _, read) = FORMATS
        .iter()
        .find(|(magic, at, _)| bytes.get(*at..).is_some_and(|rest| rest.starts_with(magic)))?;
    read(bytes)
}

type Reader = fn(&[u8]) -> Option<(u32, u32)>;

/// Each format's signature, where it sits, and how to read its size.
const FORMATS: [(&[u8], usize, Reader); 4] = [
    (b"\x89PNG\r\n\x1a\n", 0, png),
    (b"\xFF\xD8", 0, jpeg),
    (b"GIF8", 0, gif),
    (b"WEBP", 8, webp),
];

/// The size of the logical screen, right after the signature.
fn gif(bytes: &[u8]) -> Option<(u32, u32)> {
    Some((u32::from(le_u16(bytes, 6)?), u32::from(le_u16(bytes, 8)?)))
}

/// The size in the first chunk: a lossy (`VP8 `) or lossless (`VP8L`)
/// header, or the canvas of an extended file (`VP8X`).
fn webp(bytes: &[u8]) -> Option<(u32, u32)> {
    match bytes.get(12..16)? {
        b"VP8 " => Some((fourteen_bits(bytes, 26)?, fourteen_bits(bytes, 28)?)),
        b"VP8L" => lossless(bytes),
        b"VP8X" => Some((le_u24(bytes, 24)? + 1, le_u24(bytes, 27)? + 1)),
        _ => None,
    }
}

/// A lossless header: after its signature byte, the width and height less
/// one, in 14 bits each.
fn lossless(bytes: &[u8]) -> Option<(u32, u32)> {
    let bits = le_u32(bytes, 21)?;
    Some(((bits & 0x3FFF) + 1, ((bits >> 14) & 0x3FFF) + 1))
}

/// A 14-bit size of a lossy WebP frame; the top two bits are its scale.
fn fourteen_bits(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from(le_u16(bytes, at)? & 0x3FFF))
}

/// The size in the `IHDR` chunk, which comes first.
fn png(bytes: &[u8]) -> Option<(u32, u32)> {
    Some((be_u32(bytes, 16)?, be_u32(bytes, 20)?))
}

/// The size in the first frame header (SOF), after the segments before it.
fn jpeg(bytes: &[u8]) -> Option<(u32, u32)> {
    let mut at = 2;
    loop {
        if *bytes.get(at)? != 0xFF {
            return None;
        }
        let marker = *bytes.get(at + 1)?;
        if is_frame_header(marker) {
            let height = be_u16(bytes, at + 5)?;
            return Some((u32::from(be_u16(bytes, at + 7)?), u32::from(height)));
        }
        at += 2 + usize::from(be_u16(bytes, at + 2)?);
    }
}

/// SOF0 to SOF15, except the markers in that range that are not frames
/// (DHT, JPG and DAC).
fn is_frame_header(marker: u8) -> bool {
    (0xC0..=0xCF).contains(&marker) && !matches!(marker, 0xC4 | 0xC8 | 0xCC)
}

fn le_u32(bytes: &[u8], at: usize) -> Option<u32> {
    let slice = bytes.get(at..at + 4)?;
    Some(u32::from_le_bytes(slice.try_into().ok()?))
}

fn le_u24(bytes: &[u8], at: usize) -> Option<u32> {
    let slice = bytes.get(at..at + 3)?;
    Some(u32::from_le_bytes([slice[0], slice[1], slice[2], 0]))
}

fn le_u16(bytes: &[u8], at: usize) -> Option<u16> {
    let slice = bytes.get(at..at + 2)?;
    Some(u16::from_le_bytes(slice.try_into().ok()?))
}

fn be_u16(bytes: &[u8], at: usize) -> Option<u16> {
    let slice = bytes.get(at..at + 2)?;
    Some(u16::from_be_bytes(slice.try_into().ok()?))
}

fn be_u32(bytes: &[u8], at: usize) -> Option<u32> {
    let slice = bytes.get(at..at + 4)?;
    Some(u32::from_be_bytes(slice.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_size_reads_png_dimensions() {
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        png.extend_from_slice(&640_u32.to_be_bytes());
        png.extend_from_slice(&480_u32.to_be_bytes());
        png.extend_from_slice(&[8, 6, 0, 0, 0]);

        assert_eq!(image_size(&png), Some((640, 480)));
    }

    #[test]
    fn image_size_reads_jpeg_dimensions() {
        // SOI, an APP0 segment of 16 bytes, then SOF0: precision, height, width.
        let mut jpeg = b"\xFF\xD8\xFF\xE0\x00\x10JFIF\0\x01\x01\0\0\x01\0\x01\0\0".to_vec();
        jpeg.extend_from_slice(b"\xFF\xC0\x00\x11\x08");
        jpeg.extend_from_slice(&300_u16.to_be_bytes());
        jpeg.extend_from_slice(&400_u16.to_be_bytes());
        jpeg.extend_from_slice(&[3, 1, 0x22, 0, 2, 0x11, 1, 3, 0x11, 1]);

        assert_eq!(image_size(&jpeg), Some((400, 300)));
    }

    #[test]
    fn image_size_reads_gif_dimensions() {
        let mut gif = b"GIF89a".to_vec();
        gif.extend_from_slice(&320_u16.to_le_bytes());
        gif.extend_from_slice(&200_u16.to_le_bytes());
        gif.extend_from_slice(&[0, 0, 0]);

        assert_eq!(image_size(&gif), Some((320, 200)));
    }

    /// A WebP file whose first chunk is `kind` with `data`.
    fn webp(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut webp = b"RIFF\0\0\0\0WEBP".to_vec();
        webp.extend_from_slice(kind);
        webp.extend_from_slice(&u32::try_from(data.len()).unwrap().to_le_bytes());
        webp.extend_from_slice(data);
        webp
    }

    #[test]
    fn image_size_reads_webp_lossy_dimensions() {
        // Frame tag, start code, then 14-bit width and height.
        let mut frame = b"\x30\x01\x00\x9D\x01\x2A".to_vec();
        frame.extend_from_slice(&800_u16.to_le_bytes());
        frame.extend_from_slice(&600_u16.to_le_bytes());

        assert_eq!(image_size(&webp(b"VP8 ", &frame)), Some((800, 600)));
    }

    #[test]
    fn image_size_reads_webp_lossless_dimensions() {
        // Signature byte, then width - 1 and height - 1 in 14 bits each.
        let bits: u32 = (1024 - 1) | ((768 - 1) << 14);
        let mut data = vec![0x2F];
        data.extend_from_slice(&bits.to_le_bytes());

        assert_eq!(image_size(&webp(b"VP8L", &data)), Some((1024, 768)));
    }

    #[test]
    fn image_size_reads_webp_extended_dimensions() {
        // Flags, then canvas width - 1 and height - 1 in 24 bits each.
        let mut data = vec![0x10, 0, 0, 0];
        data.extend_from_slice(&(5000_u32 - 1).to_le_bytes()[..3]);
        data.extend_from_slice(&(20_u32 - 1).to_le_bytes()[..3]);

        assert_eq!(image_size(&webp(b"VP8X", &data)), Some((5000, 20)));
    }

    #[test]
    fn image_size_of_truncated_image_is_none() {
        let cut: [&[u8]; 5] = [
            b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR\0\0",
            b"\xFF\xD8\xFF\xE0\x00\x10JFIF",
            b"GIF89a\x01",
            b"RIFF\0\0\0\0WEBPVP8L\0\0\0\0\x2F\0",
            b"<svg width='10' height='10'/>",
        ];

        for bytes in cut {
            assert_eq!(image_size(bytes), None, "{bytes:?}");
        }
    }
}
