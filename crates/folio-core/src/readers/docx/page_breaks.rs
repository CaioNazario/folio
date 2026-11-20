//! "Page break before" (`w:pageBreakBefore`): a paragraph that starts on a
//! new page, set on the paragraph or on its style.

use super::styles::Styles;
use super::xml::Element;

/// Whether the paragraph `p` starts on a new page.
pub fn starts_page(p: &Element, styles: &Styles) -> bool {
    let props = p.child("pPr");
    if let Some(set) = props.and_then(page_break_before) {
        return set;
    }
    let style = props.and_then(|props| props.child("pStyle")?.attribute("val"));
    style.is_some_and(|id| styles.starts_page(id))
}

/// The `w:pageBreakBefore` set in the properties `props`: on, off (`w:val`
/// of `0`, `false` or `off`), or `None` when they do not set it.
pub fn page_break_before(props: &Element) -> Option<bool> {
    let toggle = props.child("pageBreakBefore")?;
    Some(!matches!(
        toggle.attribute("val"),
        Some("0" | "false" | "off")
    ))
}

#[cfg(test)]
mod tests {
    use super::super::read_docx;
    use super::super::test_docx::{self, docx, docx_with};
    use crate::model::{Block, Inline};

    fn blocks(body: &str) -> Vec<Block> {
        read_docx(&docx(body), None).unwrap().blocks
    }

    fn para(text: &str) -> Block {
        Block::Paragraph(vec![Inline::Text(text.to_string())])
    }

    /// A paragraph holding `text`, with `props` in its `w:pPr`.
    fn with_props(props: &str, text: &str) -> String {
        format!("<w:p><w:pPr>{props}</w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>")
    }

    #[test]
    fn paragraph_with_page_break_before_starts_on_new_page() {
        let body = with_props("", "Capa") + &with_props("<w:pageBreakBefore/>", "Capítulo");

        assert_eq!(
            blocks(&body),
            [para("Capa"), Block::PageBreak, para("Capítulo")]
        );
    }

    /// The blocks of `body` read with the `w:style` elements in `styles`.
    fn styled_blocks(styles: &str, body: &str) -> Vec<Block> {
        let styles = test_docx::styles(styles);
        let bytes = docx_with(body, &[("word/styles.xml", styles.as_bytes())]);
        read_docx(&bytes, None).unwrap().blocks
    }

    const CHAPTER_STYLE: &str = concat!(
        r#"<w:style w:type="paragraph" w:styleId="Capitulo"><w:name w:val="Capítulo"/>"#,
        r#"<w:pPr><w:pageBreakBefore/></w:pPr></w:style>"#,
    );

    #[test]
    fn paragraph_style_with_page_break_before_starts_on_new_page() {
        let body =
            with_props("", "Capa") + &with_props(r#"<w:pStyle w:val="Capitulo"/>"#, "Capítulo");

        assert_eq!(
            styled_blocks(CHAPTER_STYLE, &body),
            [para("Capa"), Block::PageBreak, para("Capítulo")]
        );
    }

    #[test]
    fn page_break_before_is_inherited_through_based_on_styles() {
        let styles = String::from(CHAPTER_STYLE)
            + r#"<w:style w:type="paragraph" w:styleId="Apendice"><w:name w:val="Apêndice"/>"#
            + r#"<w:basedOn w:val="Capitulo"/></w:style>"#;
        let body =
            with_props("", "Capa") + &with_props(r#"<w:pStyle w:val="Apendice"/>"#, "Apêndice");

        assert_eq!(
            styled_blocks(&styles, &body),
            [para("Capa"), Block::PageBreak, para("Apêndice")]
        );
    }

    #[test]
    fn page_break_before_turned_off_on_paragraph_overrides_style() {
        let props = r#"<w:pStyle w:val="Capitulo"/><w:pageBreakBefore w:val="0"/>"#;
        let body = with_props("", "Capa") + &with_props(props, "Mesma página");

        assert_eq!(
            styled_blocks(CHAPTER_STYLE, &body),
            [para("Capa"), para("Mesma página")]
        );
    }

    #[test]
    fn page_break_before_turned_off_on_style_overrides_based_on_style() {
        let styles = String::from(CHAPTER_STYLE)
            + r#"<w:style w:type="paragraph" w:styleId="Secao"><w:name w:val="Seção"/>"#
            + r#"<w:basedOn w:val="Capitulo"/>"#
            + r#"<w:pPr><w:pageBreakBefore w:val="false"/></w:pPr></w:style>"#;
        let body = with_props("", "Capa") + &with_props(r#"<w:pStyle w:val="Secao"/>"#, "Seção");

        assert_eq!(styled_blocks(&styles, &body), [para("Capa"), para("Seção")]);
    }

    #[test]
    fn page_break_before_on_first_paragraph_adds_no_break() {
        let body = with_props("<w:pageBreakBefore/>", "Capítulo") + &with_props("", "Corpo");

        assert_eq!(blocks(&body), [para("Capítulo"), para("Corpo")]);
    }

    #[test]
    fn page_break_before_after_manual_break_adds_no_second_break() {
        let manual = r#"<w:p><w:r><w:t>Capa</w:t><w:br w:type="page"/></w:r></w:p>"#;
        let body = String::from(manual) + &with_props("<w:pageBreakBefore/>", "Capítulo");

        assert_eq!(
            blocks(&body),
            [para("Capa"), Block::PageBreak, para("Capítulo")]
        );
    }
}
