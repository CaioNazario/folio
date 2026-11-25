//! `word/styles.xml`: the styles the body uses, under Word's built-in names
//! so Word and LibreOffice show them translated (spec 08, "Escritores").

use crate::model::{Face, Typography};

/// The font of all text when the model names none.
const DEFAULT_FAMILY: &str = "Calibri";
/// The size of all text when the model gives none: 11 pt, in half-points.
const DEFAULT_SIZE: u16 = 22;
const PARAGRAPH_DEFAULTS: &str = concat!(
    r#"<w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="259" w:lineRule="auto"/></w:pPr></w:pPrDefault>"#,
    r#"</w:docDefaults>"#,
);
const NORMAL: &str = r#"<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style>"#;
/// A quote: indented on both sides, in grey italics.
const QUOTE: &str = concat!(
    r#"<w:style w:type="paragraph" w:styleId="Quote"><w:name w:val="Quote"/>"#,
    r#"<w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/>"#,
    r#"<w:pPr><w:spacing w:before="160"/><w:ind w:left="864" w:right="864"/></w:pPr>"#,
    r#"<w:rPr><w:i/><w:iCs/><w:color w:val="595959"/></w:rPr></w:style>"#,
);
/// Code: Courier New 10 pt on a light grey background.
const CODE: &str = concat!(
    r#"<w:style w:type="paragraph" w:styleId="Code"><w:name w:val="Code"/>"#,
    r#"<w:basedOn w:val="Normal"/><w:qFormat/>"#,
    r#"<w:pPr><w:shd w:val="clear" w:color="auto" w:fill="F2F2F2"/><w:spacing w:line="240" w:lineRule="auto"/></w:pPr>"#,
    r#"<w:rPr><w:rFonts w:ascii="Courier New" w:hAnsi="Courier New" w:cs="Courier New"/>"#,
    r#"<w:sz w:val="20"/><w:szCs w:val="20"/></w:rPr></w:style>"#,
);
/// The text of a link: blue and underlined, as in Word.
const HYPERLINK: &str = concat!(
    r#"<w:style w:type="character" w:styleId="Hyperlink"><w:name w:val="Hyperlink"/>"#,
    r#"<w:rPr><w:color w:val="0563C1"/><w:u w:val="single"/></w:rPr></w:style>"#,
);
/// The size of each heading level, in half-points.
const HEADING_SIZES: [u8; 6] = [32, 28, 26, 24, 22, 22];

/// The content of `w:styles`, in the faces of `typography`.
pub fn styles(typography: &Typography) -> String {
    let defaults = defaults(&typography.body);
    let headings: String = (1..=6)
        .map(|level| {
            heading(
                level,
                typography.headings.get(usize::from(level - 1)),
                &typography.body,
            )
        })
        .collect();
    format!("{defaults}{NORMAL}{headings}{QUOTE}{CODE}{HYPERLINK}")
}

/// The font and size of all text, unless a style says otherwise: those
/// of the body, or Calibri 11 pt.
fn defaults(body: &Face) -> String {
    let family = body.family.as_deref().unwrap_or(DEFAULT_FAMILY);
    let size = body.size.unwrap_or(DEFAULT_SIZE);
    format!(
        concat!(
            r#"<w:docDefaults><w:rPrDefault><w:rPr>{0}"#,
            r#"<w:sz w:val="{1}"/><w:szCs w:val="{1}"/><w:lang w:val="pt-BR"/></w:rPr></w:rPrDefault>"#,
            "{2}",
        ),
        fonts(family),
        size,
        PARAGRAPH_DEFAULTS
    )
}

/// `w:rFonts` naming `family` for every script.
fn fonts(family: &str) -> String {
    format!(
        r#"<w:rFonts w:ascii="{0}" w:hAnsi="{0}" w:eastAsia="{0}" w:cs="{0}"/>"#,
        escape(family)
    )
}

fn escape(text: &str) -> String {
    quick_xml::escape::escape(text).into_owned()
}

/// The style "heading N", with the id `HeadingN`, in `face` when the
/// model gives one; the font is named only when it is not the `body`'s.
fn heading(level: u8, face: Option<&Face>, body: &Face) -> String {
    let size = face
        .and_then(|face| face.size)
        .unwrap_or(u16::from(HEADING_SIZES[usize::from(level - 1)]));
    let family = face
        .and_then(|face| face.family.as_deref())
        .filter(|family| Some(*family) != body.family.as_deref());
    let bold = face.is_none_or(|face| face.bold);
    format!(
        concat!(
            r#"<w:style w:type="paragraph" w:styleId="Heading{0}"><w:name w:val="heading {0}"/>"#,
            r#"<w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/>"#,
            r#"<w:pPr><w:keepNext/><w:spacing w:before="240" w:after="120"/><w:outlineLvl w:val="{1}"/></w:pPr>"#,
            r#"<w:rPr>{3}{4}<w:sz w:val="{2}"/><w:szCs w:val="{2}"/></w:rPr></w:style>"#,
        ),
        level,
        level - 1,
        size,
        family.map(fonts).unwrap_or_default(),
        if bold { "<w:b/><w:bCs/>" } else { "" }
    )
}

#[cfg(test)]
mod tests {
    use crate::model::{DocumentModel, Face, Typography};
    use crate::writers::docx::testing::part;
    use crate::writers::docx::write_docx;

    fn styles_of(typography: Typography) -> String {
        let model = DocumentModel {
            typography,
            ..DocumentModel::default()
        };
        part(&write_docx(&model).unwrap(), "word/styles.xml")
    }

    fn face(family: &str, size: u16, bold: bool) -> Face {
        Face {
            family: Some(family.to_string()),
            size: Some(size),
            bold,
            italic: false,
        }
    }

    #[test]
    fn write_docx_uses_model_body_font_and_size() {
        let styles = styles_of(Typography {
            body: face("Liberation Serif", 36, false),
            headings: Vec::new(),
        });

        let defaults = &styles[..styles.find("</w:docDefaults>").unwrap()];
        assert!(
            defaults.contains(concat!(
                r#"<w:rFonts w:ascii="Liberation Serif" w:hAnsi="Liberation Serif" "#,
                r#"w:eastAsia="Liberation Serif" w:cs="Liberation Serif"/>"#,
                r#"<w:sz w:val="36"/><w:szCs w:val="36"/>"#,
            )),
            "{defaults}"
        );
    }

    /// The run properties of the style `HeadingN`.
    fn heading_props(styles: &str, level: u8) -> &str {
        let start = styles
            .find(&format!(r#"w:styleId="Heading{level}""#))
            .unwrap();
        let style = &styles[start..];
        &style[..style.find("</w:style>").unwrap()]
    }

    #[test]
    fn write_docx_uses_model_heading_sizes() {
        let styles = styles_of(Typography {
            body: face("Liberation Serif", 36, false),
            headings: vec![
                face("Liberation Serif", 72, true),
                face("Liberation Serif", 54, true),
            ],
        });

        assert!(heading_props(&styles, 1).contains(r#"<w:sz w:val="72"/><w:szCs w:val="72"/>"#));
        assert!(heading_props(&styles, 2).contains(r#"<w:sz w:val="54"/><w:szCs w:val="54"/>"#));
    }

    #[test]
    fn write_docx_sets_heading_font_when_it_differs_from_body() {
        let styles = styles_of(Typography {
            body: face("Liberation Serif", 36, false),
            headings: vec![
                face("Liberation Sans", 72, true),
                face("Liberation Serif", 54, true),
            ],
        });

        assert!(heading_props(&styles, 1).contains(r#"<w:rFonts w:ascii="Liberation Sans""#));
        assert!(!heading_props(&styles, 2).contains("<w:rFonts"));
    }

    #[test]
    fn write_docx_heading_not_bold_when_model_says_so() {
        let styles = styles_of(Typography {
            body: face("Liberation Serif", 36, false),
            headings: vec![face("Liberation Serif", 72, false)],
        });

        assert!(!heading_props(&styles, 1).contains("<w:b/>"));
    }

    #[test]
    fn write_docx_without_typography_keeps_defaults() {
        let styles = styles_of(Typography::default());

        assert!(styles.contains(concat!(
            r#"<w:docDefaults><w:rPrDefault><w:rPr>"#,
            r#"<w:rFonts w:ascii="Calibri" w:hAnsi="Calibri" w:eastAsia="Calibri" w:cs="Calibri"/>"#,
            r#"<w:sz w:val="22"/><w:szCs w:val="22"/><w:lang w:val="pt-BR"/></w:rPr></w:rPrDefault>"#,
        )));
        assert!(
            heading_props(&styles, 1).ends_with(
                r#"<w:rPr><w:b/><w:bCs/><w:sz w:val="32"/><w:szCs w:val="32"/></w:rPr>"#
            )
        );
    }
}
