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

/// Matches a directory `?1` and everything inside it, given the bounds `?2`
/// and `?3` from [`RelativePath::descendant_bounds`].
///
/// [`RelativePath::descendant_bounds`]: crate::paths::RelativePath::descendant_bounds
pub const WITHIN: &str = "(path = ?1 OR (path > ?2 AND path < ?3))";
