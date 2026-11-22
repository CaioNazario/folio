//! `word/styles.xml`: the styles the body uses, under Word's built-in names
//! so Word and LibreOffice show them translated (spec 08, "Escritores").

/// Calibri 11 pt for all text, unless a style says otherwise.
const DEFAULTS: &str = concat!(
    r#"<w:docDefaults><w:rPrDefault><w:rPr>"#,
    r#"<w:rFonts w:ascii="Calibri" w:hAnsi="Calibri" w:eastAsia="Calibri" w:cs="Calibri"/>"#,
    r#"<w:sz w:val="22"/><w:szCs w:val="22"/><w:lang w:val="pt-BR"/></w:rPr></w:rPrDefault>"#,
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

/// The content of `w:styles`.
pub fn styles() -> String {
    let headings: String = (1..=6).map(heading).collect();
    format!("{DEFAULTS}{NORMAL}{headings}{QUOTE}{CODE}{HYPERLINK}")
}

/// The style "heading N", with the id `HeadingN`.
fn heading(level: u8) -> String {
    let size = HEADING_SIZES[usize::from(level - 1)];
    format!(
        concat!(
            r#"<w:style w:type="paragraph" w:styleId="Heading{0}"><w:name w:val="heading {0}"/>"#,
            r#"<w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/>"#,
            r#"<w:pPr><w:keepNext/><w:spacing w:before="240" w:after="120"/><w:outlineLvl w:val="{1}"/></w:pPr>"#,
            r#"<w:rPr><w:b/><w:bCs/><w:sz w:val="{2}"/><w:szCs w:val="{2}"/></w:rPr></w:style>"#,
        ),
        level,
        level - 1,
        size
    )
}
