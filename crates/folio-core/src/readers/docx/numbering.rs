//! The list kinds of `numbering.xml`: each `w:num` points to an abstract
//! numbering whose levels say whether they are bulleted or numbered.

use std::collections::HashMap;

use super::xml::Element;

/// Whether each (list id, level) is ordered.
#[derive(Debug, Default)]
pub struct Numbering {
    ordered: HashMap<(String, u8), bool>,
}

impl Numbering {
    /// The numbering of the `w:numbering` element; empty without the part.
    pub fn of(numbering: Option<&Element>) -> Self {
        let Some(numbering) = numbering else {
            return Self::default();
        };
        let abstracts = abstract_levels(numbering);
        let mut ordered = HashMap::new();
        for num in numbering.elements().filter(|element| element.name == "num") {
            let (Some(id), Some(levels)) = (num.attribute("numId"), abstract_of(num, &abstracts))
            else {
                continue;
            };
            ordered.extend(
                levels
                    .iter()
                    .map(|(level, kind)| ((id.to_string(), *level), *kind)),
            );
        }
        Self { ordered }
    }

    /// Whether level `level` of the list `num_id` is numbered; unknown
    /// lists are bulleted.
    pub fn is_ordered(&self, num_id: &str, level: u8) -> bool {
        let key = (num_id.to_string(), level);
        self.ordered.get(&key).copied().unwrap_or(false)
    }
}

type Levels = Vec<(u8, bool)>;

/// The levels of each `w:abstractNum`, by its id.
fn abstract_levels(numbering: &Element) -> HashMap<&str, Levels> {
    let abstracts = numbering
        .elements()
        .filter(|element| element.name == "abstractNum");
    abstracts
        .filter_map(|abs| Some((abs.attribute("abstractNumId")?, levels_of(abs))))
        .collect()
}

fn abstract_of<'a>(num: &Element, abstracts: &'a HashMap<&str, Levels>) -> Option<&'a Levels> {
    let id = num.child("abstractNumId")?.attribute("val")?;
    abstracts.get(id)
}

/// Each `w:lvl` with whether its format numbers the items.
fn levels_of(abs: &Element) -> Levels {
    let levels = abs.elements().filter(|element| element.name == "lvl");
    levels
        .filter_map(|lvl| {
            let level = lvl.attribute("ilvl")?.parse().ok()?;
            let format = lvl.child("numFmt").and_then(|fmt| fmt.attribute("val"));
            Some((level, !matches!(format, Some("bullet" | "none") | None)))
        })
        .collect()
}
