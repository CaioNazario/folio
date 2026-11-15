//! SQL functions Folio registers on every connection, before migrations, so
//! the schema and the queries apply the same rules as the core.

use rusqlite::Connection;
use rusqlite::functions::{Context, FunctionFlags};

use crate::paths::RelativePath;
use crate::search::{has_word_starting_with, normalize_file_name};

/// Registers the functions of spec 06:
/// - `folio_normalize(path)`: the normalized file name of a path;
/// - `folio_word_start(name, prefix)`: whether a later word of `name`
///   starts with `prefix`, for ranking search results.
pub fn register_functions(conn: &Connection) -> rusqlite::Result<()> {
    let flags = FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC;
    conn.create_scalar_function("folio_normalize", 1, flags, normalize_path)?;
    conn.create_scalar_function("folio_word_start", 2, flags, word_start)
}

fn normalize_path(ctx: &mut Context<'_>) -> rusqlite::Result<String> {
    let path: RelativePath = ctx.get(0)?;
    Ok(normalize_file_name(&path))
}

/// Reads the arguments in place: it runs once per matching row.
fn word_start(ctx: &mut Context<'_>) -> rusqlite::Result<bool> {
    let name = ctx.get_raw(0).as_str()?;
    let prefix = ctx.get_raw(1).as_str()?;
    Ok(has_word_starting_with(name, prefix))
}
