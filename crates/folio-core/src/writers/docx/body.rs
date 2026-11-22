//! The body of `word/document.xml`: the model's blocks as WordprocessingML.

mod image;
mod table;

pub use image::Media;

use quick_xml::escape::escape;

use super::numbering::LEVEL_INDENT;
use super::package::Relationship;
use crate::model::{Block, Inline};

/// Writes blocks into the body's XML, collecting what the other parts need.
#[derive(Default)]
pub struct Body {
    xml: String,
    relationships: Vec<Relationship>,
    /// Whether each list is ordered; list N uses `w:numId` N.
    lists: Vec<bool>,
    /// The image files, in document order.
    media: Vec<Media>,
    /// How deep in lists the current block is.
    depth: u8,
    /// Whether the current block is inside a quote.
    quoted: bool,
}

/// The body's XML and what `document.xml` needs around it.
pub struct Written {
    pub xml: String,
    pub relationships: Vec<Relationship>,
    pub lists: Vec<bool>,
    pub media: Vec<Media>,
}

impl Body {
    pub fn blocks(&mut self, blocks: &[Block]) {
        for block in blocks {
            self.block(block);
        }
    }

    pub fn finish(self) -> Written {
        Written {
            xml: self.xml,
            relationships: self.relationships,
            lists: self.lists,
            media: self.media,
        }
    }

    fn block(&mut self, block: &Block) {
        match block {
            Block::Heading { content, .. } if self.quoted => self.paragraph(&self.plain(), content),
            Block::Heading { level, content } => {
                self.paragraph(&style(&format!("Heading{level}")), content)
            }
            Block::Paragraph(content) => self.paragraph(&self.plain(), content),
            Block::List { ordered, items } => self.list(*ordered, items),
            Block::Quote(inner) => self.quote(inner),
            Block::Code { text, .. } => self.paragraph(&style("Code"), &code_lines(text)),
            Block::Table { rows } => self.table(rows),
            Block::Separator => self.paragraph(SEPARATOR, &[]),
            Block::PageBreak => self
                .xml
                .push_str(r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#),
            Block::Image { bytes, mime, alt } => self.image(bytes, mime, alt),
        }
    }

    /// The properties of a paragraph that is not a heading or an item: the
    /// quote's style inside a quote, and the item's indentation in a list.
    fn plain(&self) -> String {
        let quote = if self.quoted {
            style("Quote")
        } else {
            String::new()
        };
        format!("{quote}{}", indent(self.depth))
    }

    fn quote(&mut self, blocks: &[Block]) {
        let outer = std::mem::replace(&mut self.quoted, true);
        self.blocks(blocks);
        self.quoted = outer;
    }

    /// A paragraph with the properties `properties` (the content of `w:pPr`).
    fn paragraph(&mut self, properties: &str, content: &[Inline]) {
        self.xml.push_str("<w:p>");
        if !properties.is_empty() {
            self.xml.push_str(&format!("<w:pPr>{properties}</w:pPr>"));
        }
        self.inlines(content, Format::default());
        self.xml.push_str("</w:p>");
    }

    /// A list of numbered paragraphs, with its own numbering.
    fn list(&mut self, ordered: bool, items: &[Vec<Block>]) {
        self.lists.push(ordered);
        let numbering = self.lists.len();
        for blocks in items {
            self.item(numbering, blocks);
        }
    }

    /// The item's first paragraph carries the number; the blocks after it
    /// are one level deeper.
    fn item(&mut self, numbering: usize, blocks: &[Block]) {
        let Some((first, rest)) = blocks.split_first() else {
            return;
        };
        match first {
            Block::Paragraph(content) => self.paragraph(&numbered(numbering, self.depth), content),
            other => self.block(other),
        }
        self.depth += 1;
        self.blocks(rest);
        self.depth -= 1;
    }

    fn inlines(&mut self, content: &[Inline], format: Format) {
        for inline in content {
            self.inline(inline, format);
        }
    }

    fn inline(&mut self, inline: &Inline, format: Format) {
        match inline {
            Inline::Text(text) => self.run(format, text),
            Inline::Bold(content) => self.inlines(content, format.bold()),
            Inline::Italic(content) => self.inlines(content, format.italic()),
            Inline::Code(text) => self.run(format.code(), text),
            Inline::LineBreak => self.xml.push_str("<w:r><w:br/></w:r>"),
            Inline::Link { target, content } => self.link(target, content, format),
        }
    }

    /// A hyperlink to `target`: a bookmark of the document for `#name`,
    /// anything else through a relationship.
    fn link(&mut self, target: &str, content: &[Inline], format: Format) {
        let destination = match target.strip_prefix('#') {
            Some(anchor) => format!(r#"w:anchor="{}""#, escape(anchor)),
            None => format!(r#"r:id="{}""#, self.hyperlink(target)),
        };
        self.xml.push_str(&format!("<w:hyperlink {destination}>"));
        self.inlines(content, format);
        self.xml.push_str("</w:hyperlink>");
    }

    /// The id of a new relationship to `target`.
    fn hyperlink(&mut self, target: &str) -> String {
        let id = format!("rIdLink{}", self.relationships.len() + 1);
        self.relationships
            .push(Relationship::external(&id, "hyperlink", target));
        id
    }

    /// A run of `text`, with its spaces kept.
    fn run(&mut self, format: Format, text: &str) {
        let properties = format.properties();
        let text = escape(text);
        self.xml.push_str(&format!(
            r#"<w:r>{properties}<w:t xml:space="preserve">{text}</w:t></w:r>"#
        ));
    }
}

/// An empty paragraph whose bottom border draws the line.
const SEPARATOR: &str =
    r#"<w:pBdr><w:bottom w:val="single" w:sz="6" w:space="1" w:color="auto"/></w:pBdr>"#;
/// The monospace font of code.
const CODE_FONT: &str =
    r#"<w:rFonts w:ascii="Courier New" w:hAnsi="Courier New" w:cs="Courier New"/>"#;

/// The formatting a run inherits from the inlines around it.
#[derive(Debug, Clone, Copy, Default)]
struct Format {
    bold: bool,
    italic: bool,
    code: bool,
}

impl Format {
    fn bold(self) -> Self {
        Self { bold: true, ..self }
    }

    fn italic(self) -> Self {
        Self {
            italic: true,
            ..self
        }
    }

    fn code(self) -> Self {
        Self { code: true, ..self }
    }

    /// The run's `w:rPr`, or nothing for plain text. The order of the
    /// elements is the schema's.
    fn properties(self) -> String {
        let parts = [
            (self.code, CODE_FONT),
            (self.bold, "<w:b/>"),
            (self.italic, "<w:i/>"),
        ];
        let set: String = parts
            .iter()
            .filter(|(on, _)| *on)
            .map(|(_, xml)| *xml)
            .collect();
        if set.is_empty() {
            return set;
        }
        format!("<w:rPr>{set}</w:rPr>")
    }
}

fn style(id: &str) -> String {
    format!(r#"<w:pStyle w:val="{id}"/>"#)
}

/// The paragraph properties of an item of list `numbering` at `level`.
fn numbered(numbering: usize, level: u8) -> String {
    format!(r#"<w:numPr><w:ilvl w:val="{level}"/><w:numId w:val="{numbering}"/></w:numPr>"#)
}

/// Inside an item, the indentation that lines a block up with the item's
/// text; nothing outside lists.
fn indent(depth: u8) -> String {
    if depth == 0 {
        return String::new();
    }
    format!(r#"<w:ind w:left="{}"/>"#, LEVEL_INDENT * u32::from(depth))
}

/// The lines of a code block, joined by line breaks.
fn code_lines(text: &str) -> Vec<Inline> {
    let mut content = Vec::new();
    for line in text.trim_end_matches('\n').split('\n') {
        if !content.is_empty() {
            content.push(Inline::LineBreak);
        }
        content.push(Inline::Text(line.to_string()));
    }
    content
}

#[cfg(test)]
mod tests {
    use crate::model::{Block, Inline};
    use crate::writers::docx::testing::*;

    #[test]
    fn write_docx_quote_reads_back() {
        let blocks = vec![
            paragraph("Antes."),
            Block::Quote(vec![paragraph("Primeiro."), paragraph("Segundo.")]),
            paragraph("Depois."),
        ];

        assert_eq!(read_back(blocks.clone()), blocks);
    }

    #[test]
    fn write_docx_writes_code_block_in_code_style_keeping_lines() {
        let code = Block::Code {
            language: Some("rust".to_string()),
            text: "fn main() {\n    run();\n}\n".to_string(),
        };
        let docx = docx_of(vec![code]);

        assert!(part(&docx, "word/document.xml").contains(r#"<w:pStyle w:val="Code"/>"#));
        assert!(part(&docx, "word/styles.xml").contains(r#"w:styleId="Code""#));
        let lines = vec![
            text("fn main() {"),
            Inline::LineBreak,
            text("    run();"),
            Inline::LineBreak,
            text("}"),
        ];
        assert_eq!(blocks_of(&docx), [Block::Paragraph(lines)]);
    }

    #[test]
    fn write_docx_separator_is_paragraph_with_bottom_border() {
        let xml = document_xml(vec![Block::Separator]);

        let separator = concat!(
            r#"<w:p><w:pPr><w:pBdr><w:bottom w:val="single" w:sz="6" w:space="1" w:color="auto"/>"#,
            r#"</w:pBdr></w:pPr></w:p>"#,
        );
        assert!(xml.contains(separator), "{xml}");
    }

    #[test]
    fn write_docx_page_break_reads_back() {
        let blocks = vec![
            paragraph("Página 1."),
            Block::PageBreak,
            paragraph("Página 2."),
        ];

        assert_eq!(read_back(blocks.clone()), blocks);
    }
}
