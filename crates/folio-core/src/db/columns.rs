//! Conversions for column types shared by the databases.

use rusqlite::Row;
use rusqlite::types::Type;
use uuid::Uuid;

/// Ids are stored as hyphenated text (spec 02).
pub fn uuid_column(row: &Row<'_>, index: usize) -> rusqlite::Result<Uuid> {
    let text: String = row.get(index)?;
    Uuid::parse_str(&text)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(index, Type::Text, Box::new(e)))
}
