//! Absolute reference to a directory (workspace or Downloads), stored only in
//! the global database (spec 02).

use std::path::PathBuf;

use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

use super::os_bytes;

/// Where a directory lives. Desktop uses a path; Android will add a
/// persisted SAF URI variant.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Location {
    Path(PathBuf),
}

/// Stored as a `BLOB` with the path's exact bytes, like `RelativePath`.
impl ToSql for Location {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let Self::Path(path) = self;
        let bytes = os_bytes::as_bytes(path.as_os_str());
        Ok(ToSqlOutput::Borrowed(ValueRef::Blob(bytes)))
    }
}

impl FromSql for Location {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let bytes = value.as_blob()?;
        Ok(Self::Path(PathBuf::from(os_bytes::from_bytes(bytes))))
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    use super::*;

    #[test]
    fn location_round_trips_path_bytes_through_sqlite() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute("CREATE TABLE t (location BLOB NOT NULL)", [])
            .unwrap();
        let path = PathBuf::from(OsStr::from_bytes(b"/home/caio/Pendrive/relat\xf3rios"));
        let location = Location::Path(path);

        conn.execute("INSERT INTO t VALUES (?1)", [&location])
            .unwrap();
        let (read, kind): (Location, String) = conn
            .query_row("SELECT location, typeof(location) FROM t", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .unwrap();

        assert_eq!((read, kind.as_str()), (location, "blob"));
    }
}
