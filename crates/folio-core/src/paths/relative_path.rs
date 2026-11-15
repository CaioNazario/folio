//! A path relative to the workspace root, valid by construction.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

use super::os_bytes;

/// Path relative to the workspace root, stored as the exact bytes on disk
/// with `/` as separator. Never absolute, empty, with `.` or `..`, or inside
/// `.folio/`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RelativePath(Vec<u8>);

/// Why a byte string is not a valid [`RelativePath`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidRelativePath {
    #[error("path is empty")]
    Empty,
    #[error("path is absolute")]
    Absolute,
    #[error("path has a `.` or `..` component")]
    DotComponent,
    #[error("path has an empty component (`//` or trailing `/`)")]
    EmptyComponent,
    #[error("path is inside `.folio/`")]
    InsideFolioDir,
    #[error("path is outside the workspace root")]
    OutsideRoot,
}

impl RelativePath {
    /// Validates `bytes` as a relative path.
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Result<Self, InvalidRelativePath> {
        let bytes = bytes.into();
        validate(&bytes)?;
        Ok(Self(bytes))
    }

    /// The path of `path` relative to the workspace at `root`.
    pub fn from_absolute(root: &Path, path: &Path) -> Result<Self, InvalidRelativePath> {
        let relative = path
            .strip_prefix(root)
            .map_err(|_| InvalidRelativePath::OutsideRoot)?;
        Self::from_bytes(os_bytes::as_bytes(relative.as_os_str()))
    }

    /// The path of the entry `name` inside this directory.
    pub fn join(&self, name: &OsStr) -> Result<Self, InvalidRelativePath> {
        let mut bytes = self.0.clone();
        bytes.push(b'/');
        bytes.extend_from_slice(os_bytes::as_bytes(name));
        Self::from_bytes(bytes)
    }

    /// The last component: the entry's own name.
    pub fn file_name(&self) -> &OsStr {
        let start = self
            .0
            .iter()
            .rposition(|&b| b == b'/')
            .map_or(0, |slash| slash + 1);
        os_bytes::from_bytes(&self.0[start..])
    }

    /// The directory containing this entry; `None` at the workspace root.
    pub fn parent(&self) -> Option<Self> {
        let slash = self.0.iter().rposition(|&b| b == b'/')?;
        Some(Self(self.0[..slash].to_vec()))
    }

    /// Byte bounds of the paths inside this directory: `dir/` .. `dir0`. `0`
    /// is the byte right after `/`, so the open range holds exactly them.
    pub fn descendant_bounds(&self) -> (Vec<u8>, Vec<u8>) {
        let bound = |last: u8| [self.0.as_slice(), &[last]].concat();
        (bound(b'/'), bound(b'0'))
    }

    /// True for `dir` itself or any path inside it.
    pub fn is_within(&self, dir: &Self) -> bool {
        self == dir
            || self
                .0
                .strip_prefix(dir.0.as_slice())
                .is_some_and(|rest| rest.starts_with(b"/"))
    }

    /// This path moved from under `from` to under `to`, or `None` when it is
    /// not `from` nor inside it.
    pub fn rebase(&self, from: &Self, to: &Self) -> Option<Self> {
        if self == from {
            return Some(to.clone());
        }
        let rest = self.0.strip_prefix(from.0.as_slice())?.strip_prefix(b"/")?;
        let mut bytes = to.0.clone();
        bytes.push(b'/');
        bytes.extend_from_slice(rest);
        Some(Self(bytes))
    }

    /// The stored bytes, `/`-separated.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Text for the UI: invalid UTF-8 bytes show as `\u{FFFD}`. Never used to
    /// find the file, which always goes through the stored bytes.
    pub fn display(&self) -> String {
        String::from_utf8_lossy(&self.0).into_owned()
    }

    /// The absolute path of this entry inside the workspace at `root`.
    ///
    /// On Unix the stored `/` is already the system separator. Windows will
    /// convert here, at the edge (spec 02).
    pub fn resolve(&self, root: &Path) -> PathBuf {
        root.join(os_bytes::from_bytes(&self.0))
    }
}

/// Folio's own state directory at the workspace root.
const FOLIO_DIR: &[u8] = b".folio";

fn validate(bytes: &[u8]) -> Result<(), InvalidRelativePath> {
    if bytes.is_empty() {
        return Err(InvalidRelativePath::Empty);
    }
    if bytes.starts_with(b"/") {
        return Err(InvalidRelativePath::Absolute);
    }
    if components(bytes).next() == Some(FOLIO_DIR) {
        return Err(InvalidRelativePath::InsideFolioDir);
    }
    components(bytes).try_for_each(validate_component)
}

fn components(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
    bytes.split(|&b| b == b'/')
}

fn validate_component(component: &[u8]) -> Result<(), InvalidRelativePath> {
    match component {
        b"" => Err(InvalidRelativePath::EmptyComponent),
        b"." | b".." => Err(InvalidRelativePath::DotComponent),
        _ => Ok(()),
    }
}

impl std::fmt::Display for RelativePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display())
    }
}

/// Stored as a `BLOB` with the exact bytes, so non-UTF-8 names survive and a
/// folder's contents are a byte range in the index.
impl ToSql for RelativePath {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(ValueRef::Blob(&self.0)))
    }
}

impl FromSql for RelativePath {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        Self::from_bytes(value.as_blob()?).map_err(|e| FromSqlError::Other(Box::new(e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_path_accepts_nested_path() {
        let path = RelativePath::from_bytes("Faculdade/TCC/Trabalho.pdf").unwrap();

        assert_eq!(path.as_bytes(), b"Faculdade/TCC/Trabalho.pdf");
    }

    fn rejection(bytes: &[u8]) -> InvalidRelativePath {
        RelativePath::from_bytes(bytes).unwrap_err()
    }

    #[test]
    fn relative_path_rejects_empty() {
        assert_eq!(rejection(b""), InvalidRelativePath::Empty);
    }

    #[test]
    fn relative_path_rejects_absolute() {
        assert_eq!(
            rejection(b"/home/caio/Trabalho.pdf"),
            InvalidRelativePath::Absolute
        );
    }

    #[test]
    fn relative_path_rejects_dot_component() {
        assert_eq!(
            rejection(b"Faculdade/./TCC"),
            InvalidRelativePath::DotComponent
        );
    }

    #[test]
    fn relative_path_rejects_dot_dot_component() {
        assert_eq!(
            rejection(b"Faculdade/../../etc"),
            InvalidRelativePath::DotComponent
        );
    }

    #[test]
    fn relative_path_rejects_empty_component() {
        assert_eq!(
            rejection(b"Faculdade//TCC"),
            InvalidRelativePath::EmptyComponent
        );
        assert_eq!(
            rejection(b"Faculdade/TCC/"),
            InvalidRelativePath::EmptyComponent
        );
    }

    #[test]
    fn relative_path_rejects_path_inside_folio_dir() {
        assert_eq!(rejection(b".folio"), InvalidRelativePath::InsideFolioDir);
        assert_eq!(
            rejection(b".folio/folio.db"),
            InvalidRelativePath::InsideFolioDir
        );
    }

    #[test]
    fn relative_path_keeps_non_utf8_bytes() {
        let path = RelativePath::from_bytes(b"Antigos/relat\xf3rio.txt".to_vec()).unwrap();

        assert_eq!(path.as_bytes(), b"Antigos/relat\xf3rio.txt");
    }

    #[test]
    fn relative_path_display_replaces_invalid_bytes_with_replacement_char() {
        let path = RelativePath::from_bytes(b"Antigos/relat\xf3rio.txt".to_vec()).unwrap();

        assert_eq!(path.display(), "Antigos/relat\u{FFFD}rio.txt");
    }

    #[test]
    fn relative_path_resolves_against_workspace_root() {
        let path = RelativePath::from_bytes("Faculdade/TCC/Trabalho.pdf").unwrap();

        let resolved = path.resolve(Path::new("/home/caio/Folio"));

        assert_eq!(
            resolved,
            Path::new("/home/caio/Folio/Faculdade/TCC/Trabalho.pdf")
        );
    }

    #[test]
    fn relative_path_from_absolute_strips_workspace_root() {
        let root = Path::new("/home/caio/Folio");

        let path = RelativePath::from_absolute(root, &root.join("Faculdade/Trabalho.pdf")).unwrap();

        assert_eq!(path.as_bytes(), b"Faculdade/Trabalho.pdf");
    }

    #[test]
    fn relative_path_from_absolute_rejects_path_outside_root() {
        let root = Path::new("/home/caio/Folio");

        let result = RelativePath::from_absolute(root, Path::new("/home/caio/Downloads/a.pdf"));

        assert_eq!(result, Err(InvalidRelativePath::OutsideRoot));
    }

    #[test]
    fn relative_path_round_trips_through_sqlite_blob() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute("CREATE TABLE t (path BLOB NOT NULL)", [])
            .unwrap();
        let path = RelativePath::from_bytes(b"Antigos/relat\xf3rio.txt".to_vec()).unwrap();

        conn.execute("INSERT INTO t VALUES (?1)", [&path]).unwrap();
        let (read, kind): (RelativePath, String) = conn
            .query_row("SELECT path, typeof(path) FROM t", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .unwrap();

        assert_eq!((read, kind.as_str()), (path, "blob"));
    }
}
