//! `word/numbering.xml`: the bullets and numbers of the lists (spec 08,
//! "Escritores").

/// The bullet of each level, repeating every three levels.
const BULLETS: [&str; 3] = ["•", "◦", "▪"];
/// The definitions every list points to: bullets, and decimal numbers.
const BULLETED: u8 = 0;
const NUMBERED: u8 = 1;
/// How far each list level is indented, in twips (half an inch).
pub const LEVEL_INDENT: u32 = 720;

/// The content of `w:numbering`: the two definitions, then one numbering
/// per list, in `lists` order, from the definition of its kind.
pub fn numbering(lists: &[bool]) -> String {
    let instances: String = lists
        .iter()
        .enumerate()
        .map(|(index, ordered)| instance(index + 1, *ordered))
        .collect();
    format!(
        "{}{}{instances}",
        definition(BULLETED),
        definition(NUMBERED)
    )
}

/// The nine levels of a definition.
fn definition(id: u8) -> String {
    let levels: String = (0..9).map(|level| self::level(id, level)).collect();
    format!(r#"<w:abstractNum w:abstractNumId="{id}">{levels}</w:abstractNum>"#)
}

fn level(definition: u8, level: u8) -> String {
    let (format, text) = if definition == NUMBERED {
        ("decimal", format!("%{}.", level + 1))
    } else {
        (
            "bullet",
            BULLETS[usize::from(level) % BULLETS.len()].to_string(),
        )
    };
    let left = LEVEL_INDENT * (u32::from(level) + 1);
    format!(
        concat!(
            r#"<w:lvl w:ilvl="{0}"><w:start w:val="1"/><w:numFmt w:val="{1}"/>"#,
            r#"<w:lvlText w:val="{2}"/><w:lvlJc w:val="left"/>"#,
            r#"<w:pPr><w:ind w:left="{3}" w:hanging="360"/></w:pPr></w:lvl>"#,
        ),
        level, format, text, left
    )
}

/// A list's numbering. It restarts every level: Word would otherwise go on
/// counting from the last list of the same definition.
fn instance(id: usize, ordered: bool) -> String {
    let definition = if ordered { NUMBERED } else { BULLETED };
    let restarts: String = (0..9)
        .map(|level| {
            format!(
                r#"<w:lvlOverride w:ilvl="{level}"><w:startOverride w:val="1"/></w:lvlOverride>"#
            )
        })
        .collect();
    format!(r#"<w:num w:numId="{id}"><w:abstractNumId w:val="{definition}"/>{restarts}</w:num>"#)
}

#[cfg(test)]
mod tests {
    use crate::model::Block;
    use crate::writers::docx::testing::*;

    #[test]
    fn write_docx_unordered_list_reads_back() {
        let blocks = vec![list(false, &["Leite", "Pão"])];

        assert_eq!(read_back(blocks.clone()), blocks);
    }

    #[test]
    fn write_docx_ordered_list_reads_back() {
        let blocks = vec![list(true, &["Abrir", "Ler", "Fechar"])];

        assert_eq!(read_back(blocks.clone()), blocks);
    }

    #[test]
    fn write_docx_nested_list_reads_back() {
        let fruits = list(false, &["Maçã", "Pera"]);
        let items = vec![vec![paragraph("Frutas"), fruits], vec![paragraph("Pão")]];
        let blocks = vec![Block::List {
            ordered: true,
            items,
        }];

        assert_eq!(read_back(blocks.clone()), blocks);
    }

    #[test]
    fn write_docx_restarts_numbering_for_each_ordered_list() {
        let blocks = vec![
            list(true, &["Um", "Dois"]),
            paragraph("Entre."),
            list(true, &["Um de novo"]),
        ];

        let xml = part(&docx_of(blocks), "word/numbering.xml");

        // Word continues a numbering across lists that share a definition,
        // unless the list restarts it.
        let restart = r#"<w:lvlOverride w:ilvl="0"><w:startOverride w:val="1"/></w:lvlOverride>"#;
        let instances: Vec<&str> = xml.split("<w:num ").skip(1).collect();
        assert_eq!(instances.len(), 2, "{xml}");
        assert!(
            instances.iter().all(|instance| instance.contains(restart)),
            "{xml}"
        );
    }

    #[test]
    fn write_docx_indents_later_blocks_of_item_without_number() {
        let items = vec![vec![paragraph("Item"), paragraph("Segundo parágrafo")]];

        let xml = document_xml(vec![Block::List {
            ordered: false,
            items,
        }]);

        let indented = concat!(
            r#"<w:p><w:pPr><w:ind w:left="720"/></w:pPr>"#,
            r#"<w:r><w:t xml:space="preserve">Segundo parágrafo</w:t></w:r></w:p>"#,
        );
        assert!(xml.contains(indented), "{xml}");
    }
}
