//! `w:tbl` as a simple model table: rows of cells, each cell's paragraphs
//! as inlines.

use super::inlines::inlines_of;
use super::rels::Relationships;
use super::xml::Element;
use crate::model::{Block, Inline};

/// The table of a `w:tbl` element.
pub fn table_of(tbl: &Element, rels: &Relationships) -> Block {
    let rows = children(tbl, "tr").map(|tr| children(tr, "tc").map(|tc| cell(tc, rels)).collect());
    Block::Table {
        rows: rows.collect(),
    }
}

fn children<'a>(parent: &'a Element, name: &'a str) -> impl Iterator<Item = &'a Element> {
    parent
        .elements()
        .filter(move |element| element.name == name)
}

/// The paragraphs of a cell, joined by line breaks: a model cell holds
/// inlines only.
fn cell(tc: &Element, rels: &Relationships) -> Vec<Inline> {
    let mut content = Vec::new();
    for p in children(tc, "p") {
        let inlines = inlines_of(p, rels);
        if inlines.is_empty() {
            continue;
        }
        if !content.is_empty() {
            content.push(Inline::LineBreak);
        }
        content.extend(inlines);
    }
    content
}

#[cfg(test)]
mod tests {
    use super::super::read_docx;
    use super::super::test_docx::docx;
    use crate::model::{Block, Inline};

    fn text(text: &str) -> Inline {
        Inline::Text(text.to_string())
    }

    /// A cell holding one paragraph per text in `paragraphs`.
    fn cell(paragraphs: &[&str]) -> String {
        let each = paragraphs
            .iter()
            .map(|p| format!("<w:p><w:r><w:t>{p}</w:t></w:r></w:p>"));
        format!("<w:tc><w:tcPr/>{}</w:tc>", each.collect::<String>())
    }

    #[test]
    fn read_docx_maps_table_rows_and_cells() {
        let body = format!(
            "<w:tbl><w:tblPr/><w:tblGrid/><w:tr>{}{}</w:tr><w:tr>{}{}</w:tr></w:tbl>",
            cell(&["Item"]),
            cell(&["Preço"]),
            cell(&["Café"]),
            cell(&["R$ 5", "à vista"]),
        );

        let model = read_docx(&docx(&body)).unwrap();

        let rows = vec![
            vec![vec![text("Item")], vec![text("Preço")]],
            vec![
                vec![text("Café")],
                vec![text("R$ 5"), Inline::LineBreak, text("à vista")],
            ],
        ];
        assert_eq!(model.blocks, [Block::Table { rows }]);
    }
}
