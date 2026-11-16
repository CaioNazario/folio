//! Pictures embedded in the package (`w:drawing` pointing to a part in
//! `word/media/` by relationship), as model image blocks.

use std::collections::HashMap;

use super::xml::Element;
use crate::model::Block;
use crate::readers::image_mime;

/// The picture of a `w:drawing`, from the embedded `media` by
/// relationship id. `None` when its part is missing or not a recognized
/// image.
pub fn image_of(drawing: &Element, media: &HashMap<String, Vec<u8>>) -> Option<Block> {
    let blip = drawing.descendants("blip").into_iter().next()?;
    let bytes = media.get(blip.attribute("embed")?)?;
    Some(Block::Image {
        mime: image_mime(bytes)?.to_string(),
        bytes: bytes.clone(),
        alt: alt_of(drawing),
    })
}

/// The description the author gave the picture (`wp:docPr/@descr`).
fn alt_of(drawing: &Element) -> String {
    let doc_pr = drawing.descendants("docPr").into_iter().next();
    let descr = doc_pr.and_then(|doc_pr| doc_pr.attribute("descr"));
    descr.unwrap_or_default().to_string()
}

#[cfg(test)]
mod tests {
    use super::super::read_docx;
    use super::super::test_docx::{RELS, docx_with, rels};
    use crate::model::Block;

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\nresto";

    /// A paragraph with a picture pointing to the relationship `rel_id`.
    fn picture(rel_id: &str, descr: &str) -> String {
        format!(
            concat!(
                r#"<w:p><w:r><w:drawing><wp:inline><wp:docPr id="1" name="Imagem 1" descr="{}"/>"#,
                r#"<a:graphic><a:graphicData><pic:pic><pic:blipFill><a:blip r:embed="{}"/>"#,
                "</pic:blipFill></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>",
            ),
            descr, rel_id
        )
    }

    #[test]
    fn read_docx_maps_embedded_image_with_mime_and_alt() {
        let rels = rels(&[("rId7", "image", "media/image1.png")]);
        let parts = [(RELS, rels.as_bytes()), ("word/media/image1.png", PNG)];

        let model = read_docx(&docx_with(&picture("rId7", "Gráfico de vendas"), &parts)).unwrap();

        let image = Block::Image {
            bytes: PNG.to_vec(),
            mime: "image/png".to_string(),
            alt: "Gráfico de vendas".to_string(),
        };
        assert_eq!(model.blocks, [image]);
    }

    #[test]
    fn read_docx_keeps_image_in_place_among_text_and_page_breaks() {
        let rels = rels(&[("rId7", "image", "media/image1.png")]);
        let parts = [(RELS, rels.as_bytes()), ("word/media/image1.png", PNG)];
        let drawing = picture("rId7", "Ponto");
        let drawing = drawing
            .trim_start_matches("<w:p>")
            .trim_end_matches("</w:p>");
        let body = format!(
            r#"<w:p><w:r><w:t>Antes</w:t></w:r>{drawing}<w:r><w:br w:type="page"/><w:t>Depois</w:t></w:r></w:p>"#
        );

        let model = read_docx(&docx_with(&body, &parts)).unwrap();

        let image = Block::Image {
            bytes: PNG.to_vec(),
            mime: "image/png".to_string(),
            alt: "Ponto".to_string(),
        };
        let para = |text: &str| Block::Paragraph(vec![crate::model::Inline::Text(text.into())]);
        assert_eq!(
            model.blocks,
            [para("Antes"), image, Block::PageBreak, para("Depois")]
        );
    }
}
