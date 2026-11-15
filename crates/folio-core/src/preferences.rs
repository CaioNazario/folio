//! The app's preferences kept in the global database (spec 02):
//! interface language and theme.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ts_rs::TS;

use crate::Result;
use crate::db::GlobalDb;

/// The interface language; `System` follows the operating system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Language {
    System,
    #[serde(rename = "pt-BR")]
    PtBr,
    En,
}

/// The color theme; `System` follows the operating system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Theme {
    System,
    Dark,
    Light,
}

impl Default for Preferences {
    /// Follow the system language; dark theme, as in the design.
    fn default() -> Self {
        Self {
            language: Language::System,
            theme: Theme::Dark,
        }
    }
}

/// Every preference the UI reads at startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct Preferences {
    pub language: Language,
    pub theme: Theme,
}

impl GlobalDb {
    /// The stored preferences, with the default for each one not stored.
    pub fn preferences(&self) -> Result<Preferences> {
        let defaults = Preferences::default();
        Ok(Preferences {
            language: self.stored(LANGUAGE)?.unwrap_or(defaults.language),
            theme: self.stored(THEME)?.unwrap_or(defaults.theme),
        })
    }

    /// Stores the interface language.
    pub fn set_language(&self, language: Language) -> Result<()> {
        self.set_preference(LANGUAGE, &json!(language))
    }

    /// Stores the color theme.
    pub fn set_theme(&self, theme: Theme) -> Result<()> {
        self.set_preference(THEME, &json!(theme))
    }

    /// The preference `key`, when it is stored and readable.
    fn stored<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let value = self.preference(key)?;
        Ok(value.and_then(|value| serde_json::from_value(value).ok()))
    }
}

const LANGUAGE: &str = "language";
const THEME: &str = "theme";

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::to_value;

    use super::*;
    use crate::testing::{ManualClock, SequentialIds};

    fn open(dir: &tempfile::TempDir) -> GlobalDb {
        let clock = Arc::new(ManualClock::at(1_000));
        let ids = Arc::new(SequentialIds::default());
        GlobalDb::open(&dir.path().join("folio.db"), clock, ids).unwrap()
    }

    #[test]
    fn preferences_default_to_system_language_and_dark_theme() {
        let dir = tempfile::tempdir().unwrap();

        let preferences = open(&dir).preferences().unwrap();

        assert_eq!(
            preferences,
            Preferences {
                language: Language::System,
                theme: Theme::Dark,
            }
        );
    }

    #[test]
    fn language_and_theme_serialize_as_spec_strings() {
        let languages =
            [Language::System, Language::PtBr, Language::En].map(|l| to_value(l).unwrap());
        let themes = [Theme::System, Theme::Dark, Theme::Light].map(|t| to_value(t).unwrap());

        assert_eq!(languages, [json!("system"), json!("pt-BR"), json!("en")]);
        assert_eq!(themes, [json!("system"), json!("dark"), json!("light")]);
    }

    #[test]
    fn set_language_persists_across_reopening() {
        let dir = tempfile::tempdir().unwrap();
        open(&dir).set_language(Language::En).unwrap();

        let preferences = open(&dir).preferences().unwrap();

        assert_eq!(preferences.language, Language::En);
    }

    #[test]
    fn set_theme_persists_across_reopening() {
        let dir = tempfile::tempdir().unwrap();
        open(&dir).set_theme(Theme::Light).unwrap();

        let preferences = open(&dir).preferences().unwrap();

        assert_eq!(preferences.theme, Theme::Light);
    }

    #[test]
    fn unreadable_stored_preference_falls_back_to_default() {
        let dir = tempfile::tempdir().unwrap();
        let db = open(&dir);
        db.set_preference("language", &serde_json::json!("klingon"))
            .unwrap();
        db.set_preference("theme", &serde_json::json!(42)).unwrap();

        assert_eq!(db.preferences().unwrap(), Preferences::default());
    }
}
