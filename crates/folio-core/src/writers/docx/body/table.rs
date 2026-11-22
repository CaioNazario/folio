//! Tables of the body: a grid with borders, one paragraph per cell.

use super::Body;
use crate::model::Inline;

/// Single thin borders around and between all cells.
const BORDERS: &str = concat!(
    r#"<w:tblBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="auto"/>"#,
    r#"<w:left w:val="single" w:sz="4" w:space="0" w:color="auto"/>"#,
    r#"<w:bottom w:val="single" w:sz="4" w:space="0" w:color="auto"/>"#,
    r#"<w:right w:val="single" w:sz="4" w:space="0" w:color="auto"/>"#,
    r#"<w:insideH w:val="single" w:sz="4" w:space="0" w:color="auto"/>"#,
    r#"<w:insideV w:val="single" w:sz="4" w:space="0" w:color="auto"/></w:tblBorders>"#,
);

impl Body {
    pub(super) fn table(&mut self, rows: &[Vec<Vec<Inline>>]) {
        let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
        let grid = "<w:gridCol/>".repeat(columns);
        self.xml.push_str(&format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/>{BORDERS}</w:tblPr><w:tblGrid>{grid}</w:tblGrid>"#
        ));
        for (index, cells) in rows.iter().enumerate() {
            self.row(index == 0, cells, columns);
        }
        self.xml.push_str("</w:tbl>");
    }

    /// A row of `columns` cells, the missing ones empty; the header repeats
    /// at the top of each page the table spans.
    fn row(&mut self, header: bool, cells: &[Vec<Inline>], columns: usize) {
        self.xml.push_str("<w:tr>");
        if header {
            self.xml.push_str("<w:trPr><w:tblHeader/></w:trPr>");
        }
        let empty = Vec::new();
        let padded = cells.iter().chain(std::iter::repeat(&empty)).take(columns);
        for content in padded {
            self.xml.push_str("<w:tc>");
            self.paragraph("", content);
            self.xml.push_str("</w:tc>");
        }
        self.xml.push_str("</w:tr>");
    }
}

#[cfg(test)]
mod tests {
    use crate::writers::docx::testing::*;

    #[test]
    fn write_docx_table_reads_back() {
        let blocks = vec![table(vec![row(&["Nome", "Nota"]), row(&["Ana", "9,5"])])];

        assert_eq!(read_back(blocks.clone()), blocks);
    }

    #[test]
    fn write_docx_marks_first_table_row_as_header() {
        let xml = document_xml(vec![table(vec![
            row(&["Nome"]),
            row(&["Ana"]),
            row(&["Bia"]),
        ])]);

        let header = r#"<w:tr><w:trPr><w:tblHeader/></w:trPr><w:tc><w:p><w:r><w:t xml:space="preserve">Nome<"#;
        assert!(xml.contains(header), "{xml}");
        assert_eq!(xml.matches("<w:tblHeader/>").count(), 1, "{xml}");
    }

    #[test]
    fn write_docx_pads_short_table_rows() {
        let blocks = vec![table(vec![row(&["Nome"]), row(&["Ana", "9,5"])])];

        let padded = table(vec![
            vec![vec![text("Nome")], Vec::new()],
            row(&["Ana", "9,5"]),
        ]);
        assert_eq!(read_back(blocks), [padded]);
    }
}
