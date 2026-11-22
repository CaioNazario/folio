//! Images of the body: pictures embedded in the package, each in a
//! paragraph of its own.

use quick_xml::escape::escape;

use super::Body;
use crate::model::Inline;
use crate::writers::docx::TEXT_WIDTH;
use crate::writers::docx::image_size::image_size;
use crate::writers::docx::package::Relationship;
use crate::writers::image_extension;

/// An image file of `word/media`.
pub struct Media {
    pub name: String,
    pub bytes: Vec<u8>,
    pub mime: String,
}

/// EMUs per pixel, at 96 pixels per inch.
const EMU_PER_PIXEL: u64 = 9525;
/// EMUs per twip.
const EMU_PER_TWIP: u64 = 635;

impl Body {
    /// An image Word can show, by its size in pixels; anything else (SVG,
    /// which Word wants with a PNG copy, or a size Folio cannot read) is its
    /// alternative text in brackets.
    pub(super) fn image(&mut self, bytes: &[u8], mime: &str, alt: &str) {
        let Some(pixels) = image_size(bytes) else {
            self.alt_text(alt);
            return;
        };
        let number = self.media.len() + 1;
        let name = format!("imagem-{number}.{}", image_extension(mime));
        let id = format!("rIdImage{number}");
        self.relationships
            .push(Relationship::new(&id, "image", &format!("media/{name}")));
        self.media.push(Media {
            name,
            bytes: bytes.to_vec(),
            mime: mime.to_string(),
        });
        let size = extent(pixels);
        let drawing = drawing(number, &id, &escape(alt), size);
        self.xml
            .push_str(&format!("<w:p><w:r>{drawing}</w:r></w:p>"));
    }

    /// The alternative text in brackets, as in TXT; nothing without one.
    fn alt_text(&mut self, alt: &str) {
        if alt.is_empty() {
            return;
        }
        let text = Inline::Text(format!("[{alt}]"));
        self.paragraph(&self.plain(), &[text]);
    }
}

/// The size on the page, in EMUs, of an image of `width` × `height`
/// pixels: its own at 96 dpi, shrunk to the text width if wider.
fn extent((width, height): (u32, u32)) -> (u64, u64) {
    let natural = (
        u64::from(width) * EMU_PER_PIXEL,
        u64::from(height) * EMU_PER_PIXEL,
    );
    let text_width = u64::from(TEXT_WIDTH) * EMU_PER_TWIP;
    if natural.0 <= text_width {
        return natural;
    }
    (text_width, natural.1 * text_width / natural.0)
}

/// An inline picture of `size` (in EMUs) showing the image related by `id`.
fn drawing(number: usize, id: &str, alt: &str, (width, height): (u64, u64)) -> String {
    format!(
        concat!(
            r#"<w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0">"#,
            r#"<wp:extent cx="{2}" cy="{3}"/><wp:docPr id="{0}" name="Imagem {0}" descr="{4}"/>"#,
            r#"<wp:cNvGraphicFramePr><a:graphicFrameLocks noChangeAspect="1"/></wp:cNvGraphicFramePr>"#,
            r#"<a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture">"#,
            r#"<pic:pic><pic:nvPicPr><pic:cNvPr id="{0}" name="Imagem {0}" descr="{4}"/><pic:cNvPicPr/></pic:nvPicPr>"#,
            r#"<pic:blipFill><a:blip r:embed="{1}"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill>"#,
            r#"<pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{2}" cy="{3}"/></a:xfrm>"#,
            r#"<a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic>"#,
            r#"</a:graphicData></a:graphic></wp:inline></w:drawing>"#,
        ),
        number, id, width, height, alt
    )
}

#[cfg(test)]
mod tests {
    use crate::writers::docx::testing::*;

    #[test]
    fn write_docx_image_reads_back_with_alt() {
        let blocks = vec![
            paragraph("Antes."),
            image("image/png", png(640, 480), "Diagrama"),
            paragraph("Depois."),
        ];

        assert_eq!(read_back(blocks.clone()), blocks);
    }

    #[test]
    fn write_docx_sizes_image_from_its_pixels() {
        let xml = document_xml(vec![image("image/png", png(200, 100), "Logo")]);

        // 9525 EMUs per pixel at 96 dpi.
        assert!(
            xml.contains(r#"<wp:extent cx="1905000" cy="952500"/>"#),
            "{xml}"
        );
    }

    #[test]
    fn write_docx_shrinks_wide_image_to_text_width() {
        let xml = document_xml(vec![image("image/png", png(1210, 605), "Panorama")]);

        // The text is 11906 - 2 × 1417 = 9072 twips wide, 5760720 EMUs.
        assert!(
            xml.contains(r#"<wp:extent cx="5760720" cy="2880360"/>"#),
            "{xml}"
        );
    }

    #[test]
    fn write_docx_writes_svg_image_as_alt_text() {
        let svg = image(
            "image/svg+xml",
            br#"<svg width="10" height="10"/>"#.to_vec(),
            "Ícone",
        );

        assert_eq!(read_back(vec![svg]), [paragraph("[Ícone]")]);
    }

    #[test]
    fn write_docx_writes_image_of_unknown_size_as_alt_text() {
        let cut_png = image("image/png", b"\x89PNG\r\n\x1a\n\0\0".to_vec(), "Foto");

        assert_eq!(read_back(vec![cut_png]), [paragraph("[Foto]")]);
    }

    #[test]
    fn write_docx_drops_unshown_image_without_alt() {
        let svg = image("image/svg+xml", b"<svg/>".to_vec(), "");
        let blocks = vec![paragraph("Antes."), svg, paragraph("Depois.")];

        assert!(!document_xml(blocks.clone()).contains("[]"));
        assert_eq!(
            read_back(blocks),
            [paragraph("Antes."), paragraph("Depois.")]
        );
    }
}
