//! The runs of a paragraph as a flow, in document order: model inlines,
//! page breaks and pictures.

use super::rels::Relationships;
use super::xml::Element;
use crate::model::Inline;

/// What a paragraph holds, in order.
pub enum Flow<'a> {
    Inline(Inline),
    PageBreak,
    /// A `w:drawing`, which may hold a picture.
    Drawing(&'a Element),
}

/// The flow of the runs and hyperlinks in `p`.
pub fn flow_of<'a>(p: &'a Element, rels: &Relationships) -> Vec<Flow<'a>> {
    let mut flow = Vec::new();
    for element in p.elements() {
        match element.name.as_str() {
            "r" => flow.extend(run_flow(element)),
            "hyperlink" => flow.extend(hyperlink(element, rels).into_iter().map(Flow::Inline)),
            _ => {}
        }
    }
    flow
}

/// The inlines of `p` alone, merged; page breaks and pictures are dropped.
pub fn inlines_of(p: &Element, rels: &Relationships) -> Vec<Inline> {
    let inlines = flow_of(p, rels).into_iter().filter_map(|item| match item {
        Flow::Inline(inline) => Some(inline),
        Flow::PageBreak | Flow::Drawing(_) => None,
    });
    merge(inlines.collect())
}

/// A `w:hyperlink` as a link to its relationship's target, or to its
/// bookmark (`w:anchor`) inside the document.
fn hyperlink(link: &Element, rels: &Relationships) -> Vec<Inline> {
    let content = inlines_of(link, rels);
    match link_target(link, rels) {
        Some(target) => vec![Inline::Link { target, content }],
        None => content,
    }
}

fn link_target(link: &Element, rels: &Relationships) -> Option<String> {
    if let Some(target) = link.attribute("id").and_then(|id| rels.target(id)) {
        return Some(target.to_string());
    }
    link.attribute("anchor").map(|anchor| format!("#{anchor}"))
}

/// The flow of `run`: its inlines, wrapped in the emphasis of the run's
/// properties, around its page breaks and drawings.
fn run_flow(run: &Element) -> Vec<Flow<'_>> {
    let props = run.child("rPr");
    let mut flow = Vec::new();
    let mut pending = Vec::new();
    for element in run.elements() {
        let marker = match element.name.as_str() {
            "drawing" => Flow::Drawing(element),
            "br" if is_page_break(element) => Flow::PageBreak,
            _ => {
                pending.extend(content_inline(element));
                continue;
            }
        };
        push_emphasized(&mut flow, &mut pending, props);
        flow.push(marker);
    }
    push_emphasized(&mut flow, &mut pending, props);
    flow
}

/// Moves the `pending` inlines to `flow`, with the run's emphasis.
fn push_emphasized(flow: &mut Vec<Flow>, pending: &mut Vec<Inline>, props: Option<&Element>) {
    let emphasized = emphasized(std::mem::take(pending), props);
    flow.extend(emphasized.into_iter().map(Flow::Inline));
}

/// Whether `element` is a `w:br` that starts a new page.
pub fn is_page_break(element: &Element) -> bool {
    element.name == "br" && element.attribute("type") == Some("page")
}

/// `content` wrapped in the run's italic and bold, if set.
fn emphasized(mut content: Vec<Inline>, props: Option<&Element>) -> Vec<Inline> {
    if content.is_empty() {
        return content;
    }
    if is_on(props, "i") {
        content = vec![Inline::Italic(content)];
    }
    if is_on(props, "b") {
        content = vec![Inline::Bold(content)];
    }
    content
}

/// The inline for an element of a run's content, `None` for the rest.
fn content_inline(element: &Element) -> Option<Inline> {
    match element.name.as_str() {
        "t" => Some(Inline::Text(element.text())),
        "br" => Some(Inline::LineBreak),
        _ => None,
    }
}

/// Whether the toggle property `name` (`w:b`, `w:i`) is set: present,
/// and not turned off by `w:val`.
fn is_on(props: Option<&Element>, name: &str) -> bool {
    let Some(toggle) = props.and_then(|props| props.child(name)) else {
        return false;
    };
    !matches!(toggle.attribute("val"), Some("0" | "false" | "off"))
}

/// Adjacent texts as one, and adjacent bold or italic as one, merged
/// inside too: Word splits runs at will.
pub fn merge(inlines: Vec<Inline>) -> Vec<Inline> {
    let mut merged: Vec<Inline> = Vec::new();
    for inline in inlines {
        match (merged.last_mut(), inline) {
            (Some(Inline::Text(last)), Inline::Text(text)) => last.push_str(&text),
            (Some(Inline::Bold(last)), Inline::Bold(more))
            | (Some(Inline::Italic(last)), Inline::Italic(more)) => join(last, more),
            (_, inline) => merged.push(inline),
        }
    }
    merged
}

/// `more` appended to `last`, merged again.
fn join(last: &mut Vec<Inline>, more: Vec<Inline>) {
    last.extend(more);
    *last = merge(std::mem::take(last));
}

#[cfg(test)]
mod tests {
    use super::super::read_docx;
    use super::super::test_docx::{RELS, docx, docx_with, rels};
    use crate::model::{Block, Inline};

    /// The inlines of a document whose single paragraph holds `runs`.
    fn inlines(runs: &str) -> Vec<Inline> {
        let model = read_docx(&docx(&format!("<w:p>{runs}</w:p>")), None).unwrap();
        match model.blocks.as_slice() {
            [Block::Paragraph(content)] => content.clone(),
            other => panic!("expected one paragraph, got {other:?}"),
        }
    }

    fn text(text: &str) -> Inline {
        Inline::Text(text.to_string())
    }

    /// A run with the run properties `props` and the text `text`.
    fn run(props: &str, text: &str) -> String {
        format!(r#"<w:r><w:rPr>{props}</w:rPr><w:t xml:space="preserve">{text}</w:t></w:r>"#)
    }

    #[test]
    fn read_docx_maps_bold_and_italic_runs() {
        let runs = [
            run("", "Um "),
            run("<w:b/>", "forte"),
            run("", " e "),
            run("<w:i/>", "leve"),
            run("", "."),
        ]
        .concat();

        assert_eq!(
            inlines(&runs),
            [
                text("Um "),
                Inline::Bold(vec![text("forte")]),
                text(" e "),
                Inline::Italic(vec![text("leve")]),
                text("."),
            ]
        );
    }

    #[test]
    fn read_docx_ignores_bold_turned_off_by_val_false() {
        let runs = [
            run(r#"<w:b w:val="0"/>"#, "sem "),
            run(r#"<w:b w:val="false"/><w:i w:val="off"/>"#, "ênfase"),
        ]
        .concat();

        assert_eq!(inlines(&runs), [text("sem ênfase")]);
    }

    #[test]
    fn read_docx_maps_line_break_inside_paragraph() {
        let runs = "<w:r><w:t>Rua A</w:t><w:br/><w:t>Centro</w:t></w:r>";

        assert_eq!(
            inlines(runs),
            [text("Rua A"), Inline::LineBreak, text("Centro")]
        );
    }

    #[test]
    fn read_docx_maps_external_hyperlink_from_relationships() {
        let rels = rels(&[("rId9", "hyperlink", "https://exemplo.com/a?b=1&amp;c=2")]);
        let body = concat!(
            "<w:p><w:r><w:t xml:space=\"preserve\">Veja </w:t></w:r>",
            "<w:hyperlink r:id=\"rId9\"><w:r><w:t>o site</w:t></w:r></w:hyperlink></w:p>",
        );

        let model = read_docx(&docx_with(body, &[(RELS, rels.as_bytes())]), None).unwrap();

        let link = Inline::Link {
            target: "https://exemplo.com/a?b=1&c=2".to_string(),
            content: vec![text("o site")],
        };
        assert_eq!(model.blocks, [Block::Paragraph(vec![text("Veja "), link])]);
    }

    #[test]
    fn read_docx_maps_internal_anchor_hyperlink() {
        let runs =
            r#"<w:hyperlink w:anchor="_Toc1"><w:r><w:t>Introdução</w:t></w:r></w:hyperlink>"#;

        assert_eq!(
            inlines(runs),
            [Inline::Link {
                target: "#_Toc1".to_string(),
                content: vec![text("Introdução")],
            }]
        );
    }

    #[test]
    fn read_docx_merges_adjacent_runs_with_same_emphasis() {
        let runs = [
            run("<w:b/>", "neg"),
            run("<w:b/>", "rito"),
            run("<w:b/><w:i/>", " forte"),
        ]
        .concat();

        assert_eq!(
            inlines(&runs),
            [Inline::Bold(vec![
                text("negrito"),
                Inline::Italic(vec![text(" forte")]),
            ])]
        );
    }
}
