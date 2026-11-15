//! Identity of a file in the filesystem: device and inode (spec 05).

use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;

/// Device and inode of a file. Not valid across machines: a workspace copied
/// to another computer reconnects only by content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FsId {
    pub dev: i64,
    pub ino: i64,
}

impl FsId {
    /// Stored as SQLite integers: the u64 bits are kept as i64.
    pub fn from_metadata(metadata: &Metadata) -> Self {
        Self {
            dev: metadata.dev() as i64,
            ino: metadata.ino() as i64,
        }
    }
}

/// What stage 1 of reconnection compares about a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileIdentity {
    pub fs_id: FsId,
    pub size: u64,
    pub modified_at: i64,
}

/// Same device and inode, and same size and mtime: `mv` and file managers
/// keep all of them. Requiring size and mtime avoids taking an inode reused
/// by another file for the old one.
pub fn is_same_file(recorded: &FileIdentity, found: &FileIdentity) -> bool {
    recorded == found
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rusqlite::Connection;
    use uuid::Uuid;

    use super::*;
    use crate::documents::test_docs::write_doc;
    use crate::workspace::test_env::TestEnv;

    /// The recorded filesystem identity of the document `id`.
    fn fs_id_of(conn: &Connection, id: Uuid) -> rusqlite::Result<Option<FsId>> {
        conn.query_row(
            "SELECT fs_dev, fs_ino FROM documents WHERE id = ?1",
            [id.to_string()],
            |row| {
                let (dev, ino): (Option<i64>, Option<i64>) = (row.get(0)?, row.get(1)?);
                Ok(dev.zip(ino).map(|(dev, ino)| FsId { dev, ino }))
            },
        )
    }

    #[test]
    fn indexing_records_device_and_inode() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let path = write_doc(&workspace, "Trabalho.pdf", b"%PDF-1.7\n");
        let id = workspace.index_file(&path).unwrap().unwrap().id;

        let recorded = fs_id_of(&workspace.conn(), id).unwrap();

        let metadata = fs::metadata(workspace.root().join("Trabalho.pdf")).unwrap();
        assert_eq!(recorded, Some(FsId::from_metadata(&metadata)));
    }

    #[test]
    fn inode_match_with_different_size_or_mtime_is_not_same_file() {
        let recorded = FileIdentity {
            fs_id: FsId { dev: 1, ino: 42 },
            size: 100,
            modified_at: 5_000,
        };
        let other_size = FileIdentity {
            size: 101,
            ..recorded
        };
        let other_mtime = FileIdentity {
            modified_at: 6_000,
            ..recorded
        };

        assert!(is_same_file(&recorded, &recorded));
        assert!(!is_same_file(&recorded, &other_size));
        assert!(!is_same_file(&recorded, &other_mtime));
    }
}
