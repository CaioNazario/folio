//! Test-only helpers: minimal PDFs written by hand with the standard
//! fonts, so each test states the exact font and position of its text.

use super::test_crypt::Security;

/// The standard fonts every page declares, as `/F1`, `/F2` and `/F3`.
#[derive(Debug, Clone, Copy)]
pub enum Font {
    Regular,
    Bold,
    Italic,
}

impl Font {
    fn resource(self) -> &'static str {
        match self {
            Font::Regular => "F1",
            Font::Bold => "F2",
            Font::Italic => "F3",
        }
    }
}

/// `text` shown at (`x`, `y`), from the bottom left of a 612 × 792 page.
#[derive(Debug, Clone)]
pub struct Text {
    pub font: Font,
    pub size: f64,
    pub x: f64,
    pub y: f64,
    pub text: String,
}

/// Regular 12 pt text at (`x`, `y`).
pub fn text(text: &str, x: f64, y: f64) -> Text {
    Text {
        font: Font::Regular,
        size: 12.0,
        x,
        y,
        text: text.to_string(),
    }
}

impl Text {
    pub fn bold(self) -> Self {
        Self {
            font: Font::Bold,
            ..self
        }
    }

    pub fn italic(self) -> Self {
        Self {
            font: Font::Italic,
            ..self
        }
    }

    pub fn size(self, size: f64) -> Self {
        Self { size, ..self }
    }
}

/// A PDF with one page per item of `pages`.
pub fn pdf(pages: &[Vec<Text>]) -> Vec<u8> {
    pdf_with_info(pages, "")
}

/// A PDF whose Info dictionary holds the `info` entries.
pub fn pdf_with_info(pages: &[Vec<Text>], info: &str) -> Vec<u8> {
    let contents: Vec<Vec<u8>> = pages.iter().map(|texts| content(texts)).collect();
    let mut writer = Writer::default();
    writer.document(&contents, "", &format!("<< {info} >>"));
    writer.finish()
}

/// A PDF whose fonts are embedded and described as pdflatex describes
/// Computer Modern: names without "bold" and no weight, only the stem
/// width. `/F1` is CMR10, `/F2` CMBX10 and `/F3` CMTI10.
pub fn pdf_with_latex_fonts(pages: &[Vec<Text>]) -> Vec<u8> {
    use typst_assets::pdf::{SERIF, SERIF_BOLD, SERIF_ITALIC};
    let mut writer = Writer::default();
    let fonts = [
        (SERIF, "CMR10", 69, 0.0),
        (SERIF_BOLD, "CMBX10", 114, 0.0),
        (SERIF_ITALIC, "CMTI10", 68, -14.04),
    ];
    let dicts = fonts.map(|(program, name, stem, angle)| {
        let file = writer.stream("/Subtype /Type1C", program);
        let descriptor = writer.add(format!(
            "<< /Type /FontDescriptor /FontName /ABCDEF+{name} /Flags 34 /FontBBox [0 -250 1000 750] \
             /ItalicAngle {angle} /Ascent 694 /Descent -194 /CapHeight 683 /StemV {stem} /FontFile3 {file} 0 R >>"
        ));
        format!("<< /Type /Font /Subtype /Type1 /BaseFont /ABCDEF+{name} /Encoding /WinAnsiEncoding /FontDescriptor {descriptor} 0 R >>")
    });
    writer.font_dicts = dicts.to_vec();
    let contents: Vec<Vec<u8>> = pages.iter().map(|texts| content(texts)).collect();
    writer.document(&contents, "", "<< >>");
    writer.finish()
}

/// A PDF that opens with `user` or `owner`; an empty `user` opens it
/// without a password.
pub fn pdf_with_password(pages: &[Vec<Text>], user: &str, owner: &str) -> Vec<u8> {
    let contents: Vec<Vec<u8>> = pages.iter().map(|texts| content(texts)).collect();
    let mut writer = Writer {
        security: Some(Security::new(user, owner)),
        ..Writer::default()
    };
    writer.document(&contents, "", "<< >>");
    writer.finish()
}

/// A PDF whose page draws a form XObject that shows `text` in
/// Times-Bold, a font only the form declares.
pub fn pdf_with_form(text: &str) -> Vec<u8> {
    let mut writer = Writer::default();
    let font = writer
        .add("<< /Type /Font /Subtype /Type1 /BaseFont /Times-Bold /Encoding /WinAnsiEncoding >>");
    form_document(writer, font, text)
}

/// A PDF that shows `text` (letters `a` and `b` only) in a Type3 font
/// whose glyphs have private names and no Unicode map.
pub fn pdf_without_unicode(text: &str) -> Vec<u8> {
    let mut writer = Writer::default();
    let glyph = writer.stream("", b"600 0 0 0 500 700 d1 50 0 400 700 re f");
    let font = writer.add(format!(
        "<< /Type /Font /Subtype /Type3 /FontBBox [0 0 500 700] /FontMatrix [0.001 0 0 0.001 0 0] \
         /CharProcs << /g0 {glyph} 0 R /g1 {glyph} 0 R >> \
         /Encoding << /Type /Encoding /Differences [97 /g0 /g1] >> /FirstChar 97 /LastChar 98 /Widths [600 600] >>"
    ));
    form_document(writer, font, text)
}

/// A PDF whose single page is only an image, as a scanner makes it.
pub fn pdf_with_image() -> Vec<u8> {
    let mut writer = Writer::default();
    let dict = "/Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceGray /BitsPerComponent 8";
    let image = writer.stream(dict, &[0, 255, 255, 0]);
    let resources = format!("/XObject << /Im1 {image} 0 R >>");
    let content = b"q 612 0 0 792 0 0 cm /Im1 Do Q".to_vec();
    writer.document(&[content], &resources, "<< >>");
    writer.finish()
}

/// A PDF whose page shows "Antes" at the top, then a 100 pt square image
/// in the middle (y 400 to 500 from the bottom), then "Depois" at the
/// bottom.
pub fn pdf_with_text_and_image() -> Vec<u8> {
    pdf_with_image_amid(
        &[text("Antes", 72.0, 700.0)],
        &[text("Depois", 72.0, 200.0)],
    )
}

/// A PDF whose page shows the `before` texts, then draws a 100 pt square
/// image (y 400 to 500 from the bottom), then shows the `after` texts.
pub fn pdf_with_image_amid(before: &[Text], after: &[Text]) -> Vec<u8> {
    let mut writer = Writer::default();
    let dict = "/Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceGray /BitsPerComponent 8";
    let image = writer.stream(dict, &[0, 255, 255, 0]);
    let resources = format!("/XObject << /Im1 {image} 0 R >>");
    let mut page = content(before);
    page.extend(b"q 100 0 0 100 100 400 cm /Im1 Do Q\n");
    page.extend(content(after));
    writer.document(&[page], &resources, "<< >>");
    writer.finish()
}

/// A PDF whose page is a 2 × 2 gray image drawn `width` × `height` pt.
pub fn pdf_with_sized_image(width: f64, height: f64) -> Vec<u8> {
    let mut writer = Writer::default();
    let dict = "/Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceGray /BitsPerComponent 8";
    let image = writer.stream(dict, &[0, 255, 255, 0]);
    let resources = format!("/XObject << /Im1 {image} 0 R >>");
    let content = format!("q {width} 0 0 {height} 100 400 cm /Im1 Do Q").into_bytes();
    writer.document(&[content], &resources, "<< >>");
    writer.finish()
}

/// A PDF whose page draws a 6000 × 5000 gray image (30 megapixels), all
/// zeros, run-length encoded so that it stays small.
pub fn pdf_with_huge_image() -> Vec<u8> {
    let mut writer = Writer::default();
    let dict = "/Type /XObject /Subtype /Image /Width 6000 /Height 5000 /ColorSpace /DeviceGray \
                /BitsPerComponent 8 /Filter /RunLengthDecode";
    // 129 repeats the next byte 128 times; 128 ends the data.
    let runs = (6000 * 5000) / 128 + 1;
    let data: Vec<u8> = [129, 0].repeat(runs).into_iter().chain([128]).collect();
    let image = writer.stream(dict, &data);
    let resources = format!("/XObject << /Im1 {image} 0 R >>");
    let content = b"q 100 0 0 100 100 400 cm /Im1 Do Q".to_vec();
    writer.document(&[content], &resources, "<< >>");
    writer.finish()
}

/// A PDF whose page paints an 8 × 8 stencil mask (`/ImageMask`).
pub fn pdf_with_stencil_mask() -> Vec<u8> {
    let mut writer = Writer::default();
    let dict =
        "/Type /XObject /Subtype /Image /Width 8 /Height 8 /ImageMask true /BitsPerComponent 1";
    let image = writer.stream(dict, &[0xAA; 8]);
    let resources = format!("/XObject << /Im1 {image} 0 R >>");
    let content = b"q 100 0 0 100 100 400 cm /Im1 Do Q".to_vec();
    writer.document(&[content], &resources, "<< >>");
    writer.finish()
}

/// A PDF whose page is a 2 × 2 RGB image with a soft mask (alpha).
pub fn pdf_with_soft_masked_image() -> Vec<u8> {
    let mut writer = Writer::default();
    let mask = writer.stream(
        "/Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceGray /BitsPerComponent 8",
        &[0, 85, 170, 255],
    );
    let dict = format!(
        "/Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceRGB /BitsPerComponent 8 /SMask {mask} 0 R"
    );
    let image = writer.stream(&dict, &[200; 12]);
    let resources = format!("/XObject << /Im1 {image} 0 R >>");
    let content = b"q 100 0 0 100 100 400 cm /Im1 Do Q".to_vec();
    writer.document(&[content], &resources, "<< >>");
    writer.finish()
}

/// Finishes `writer` with one page that draws a form XObject, which shows
/// `text` in `font`, declared only by the form as `/FX`.
fn form_document(mut writer: Writer, font: usize, text: &str) -> Vec<u8> {
    let mut shown = b"BT /FX 12 Tf 72 700 Td (".to_vec();
    shown.extend(literal(text));
    shown.extend(b") Tj ET");
    let dict = format!(
        "/Type /XObject /Subtype /Form /BBox [0 0 612 792] /Resources << /Font << /FX {font} 0 R >> >>"
    );
    let form = writer.stream(&dict, &shown);
    let resources = format!("/XObject << /X1 {form} 0 R >>");
    writer.document(&[b"/X1 Do".to_vec()], &resources, "<< >>");
    writer.finish()
}

/// The content stream that shows `texts`.
pub fn content(texts: &[Text]) -> Vec<u8> {
    let mut stream = Vec::new();
    for t in texts {
        stream.extend(
            format!(
                "BT /{} {} Tf {} {} Td (",
                t.font.resource(),
                t.size,
                t.x,
                t.y
            )
            .bytes(),
        );
        stream.extend(literal(&t.text));
        stream.extend(b") Tj ET\n");
    }
    stream
}

/// `text` in WinAnsi, escaped for a literal string.
fn literal(text: &str) -> Vec<u8> {
    let (bytes, _, _) = encoding_rs::WINDOWS_1252.encode(text);
    let mut escaped = Vec::new();
    for byte in bytes.iter() {
        if matches!(byte, b'(' | b')' | b'\\') {
            escaped.push(b'\\');
        }
        escaped.push(*byte);
    }
    escaped
}

/// A PDF file being written, object by object.
#[derive(Default)]
pub struct Writer {
    objects: Vec<Vec<u8>>,
    trailer: String,
    /// Encrypts the streams when set.
    security: Option<Security>,
    /// The dictionaries of `/F1`, `/F2` and `/F3`; the standard fonts
    /// when empty.
    font_dicts: Vec<String>,
}

impl Writer {
    /// Adds `object` and returns its number.
    pub fn add(&mut self, object: impl Into<Vec<u8>>) -> usize {
        self.objects.push(object.into());
        self.objects.len()
    }

    /// Adds a stream with `dict` entries around `data`.
    pub fn stream(&mut self, dict: &str, data: &[u8]) -> usize {
        let mut data = data.to_vec();
        if let Some(security) = &self.security {
            let key = security.object_key(self.objects.len() + 1);
            super::test_crypt::rc4(&key, &mut data);
        }
        let mut object = format!("<< {dict} /Length {} >>\nstream\n", data.len()).into_bytes();
        object.extend(data);
        object.extend(b"\nendstream");
        self.add(object)
    }

    /// Adds the catalog, the pages with `contents` and the standard fonts,
    /// and the `info` dictionary. `resources` goes into every page's
    /// resources, next to the fonts.
    pub fn document(&mut self, contents: &[Vec<u8>], resources: &str, info: &str) {
        let root = self.add("<< /Type /Catalog /Pages 2 0 R >>");
        let pages = self.add("");
        let fonts = self.fonts();
        let kids: Vec<String> = contents
            .iter()
            .map(|content| self.page(pages, content, &format!("/Font {fonts} 0 R {resources}")))
            .collect();
        self.objects[pages - 1] = format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            kids.join(" "),
            kids.len()
        )
        .into_bytes();
        let info = self.add(info);
        self.trailer = format!("/Root {root} 0 R /Info {info} 0 R");
    }

    fn fonts(&mut self) -> usize {
        if self.font_dicts.is_empty() {
            self.font_dicts = ["Helvetica", "Helvetica-Bold", "Helvetica-Oblique"]
                .map(|name| {
                    format!("<< /Type /Font /Subtype /Type1 /BaseFont /{name} /Encoding /WinAnsiEncoding >>")
                })
                .to_vec();
        }
        let dicts = std::mem::take(&mut self.font_dicts);
        let ids: Vec<String> = dicts
            .into_iter()
            .enumerate()
            .map(|(i, font)| format!("/F{} {} 0 R", i + 1, self.add(font)))
            .collect();
        self.add(format!("<< {} >>", ids.join(" ")))
    }

    fn page(&mut self, parent: usize, content: &[u8], resources: &str) -> String {
        let content = self.stream("", content);
        let page = format!(
            "<< /Type /Page /Parent {parent} 0 R /MediaBox [0 0 612 792] /Contents {content} 0 R /Resources << {resources} >> >>"
        );
        format!("{} 0 R", self.add(page))
    }

    /// The file: header, objects, cross-reference table and trailer.
    pub fn finish(mut self) -> Vec<u8> {
        if let Some(security) = self.security.take() {
            let dict = self.add(security.dict());
            self.trailer = format!("{} {}", self.trailer, Security::trailer(dict));
        }
        let mut file = b"%PDF-1.4\n".to_vec();
        let mut offsets = Vec::new();
        for (i, object) in self.objects.iter().enumerate() {
            offsets.push(file.len());
            file.extend(format!("{} 0 obj\n", i + 1).bytes());
            file.extend(object);
            file.extend(b"\nendobj\n");
        }
        let xref = file.len();
        file.extend(xref_table(&offsets).bytes());
        let size = offsets.len() + 1;
        let trailer = format!("<< /Size {size} {} >>", self.trailer);
        file.extend(format!("trailer\n{trailer}\nstartxref\n{xref}\n%%EOF\n").bytes());
        file
    }
}

/// The cross-reference table of objects at `offsets`.
fn xref_table(offsets: &[usize]) -> String {
    let entries: String = offsets
        .iter()
        .map(|offset| format!("{offset:010} 00000 n \n"))
        .collect();
    format!(
        "xref\n0 {}\n0000000000 65535 f \n{entries}",
        offsets.len() + 1
    )
}
