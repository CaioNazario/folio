//! Schema of the workspace database. Each spec appends its own migration;
//! a published entry is never edited (spec 02).

pub const MIGRATIONS: &[&str] = &[
    // Spec 02: a single row with the workspace's own id.
    "CREATE TABLE meta (
         id TEXT NOT NULL,
         created_at INTEGER NOT NULL
     );",
    // Spec 03: icons of projects and folders, by directory path.
    "CREATE TABLE icons (
         path BLOB PRIMARY KEY NOT NULL,
         icon TEXT NOT NULL
     );",
    // Spec 04: the document index, the metadata cache and conversion links.
    // `missing_since` marks a document gone from disk (spec 05).
    "CREATE TABLE documents (
         id TEXT PRIMARY KEY NOT NULL,
         path BLOB NOT NULL UNIQUE,
         format TEXT NOT NULL,
         valid INTEGER NOT NULL,
         protected INTEGER NOT NULL,
         size INTEGER NOT NULL,
         modified_at INTEGER NOT NULL,
         created_at INTEGER,
         hash BLOB,
         hash_size INTEGER,
         hash_modified_at INTEGER,
         missing_since INTEGER
     );
     CREATE TABLE document_metadata (
         document_id TEXT PRIMARY KEY NOT NULL
             REFERENCES documents (id) ON DELETE CASCADE,
         title TEXT,
         author TEXT,
         subject TEXT,
         keywords TEXT,
         doc_created_at INTEGER,
         doc_modified_at INTEGER,
         pages INTEGER,
         extra TEXT,
         extracted_size INTEGER NOT NULL,
         extracted_modified_at INTEGER NOT NULL
     );
     CREATE TABLE conversions (
         id TEXT PRIMARY KEY NOT NULL,
         source_id TEXT NOT NULL REFERENCES documents (id) ON DELETE CASCADE,
         target_id TEXT NOT NULL REFERENCES documents (id) ON DELETE CASCADE,
         source_format TEXT NOT NULL,
         target_format TEXT NOT NULL,
         converted_at INTEGER NOT NULL,
         source_modified_at INTEGER NOT NULL,
         target_modified_at INTEGER NOT NULL
     );
     CREATE INDEX conversions_by_source ON conversions (source_id);
     CREATE INDEX conversions_by_target ON conversions (target_id);",
    // Spec 05: filesystem identity (device and inode) for reconnecting files
    // moved outside Folio, and the lookups reconnection needs. The size
    // index only holds hashed documents: the only ones stage 2 can compare.
    "ALTER TABLE documents ADD COLUMN fs_dev INTEGER;
     ALTER TABLE documents ADD COLUMN fs_ino INTEGER;
     CREATE INDEX documents_by_fs_id ON documents (fs_dev, fs_ino);
     CREATE INDEX documents_hashed_by_size ON documents (size) WHERE hash IS NOT NULL;",
    // Spec 06: the normalized file name searches compare against, kept in
    // step with `path` by triggers so no operation can forget it.
    "ALTER TABLE documents ADD COLUMN name_norm TEXT NOT NULL DEFAULT '';
     UPDATE documents SET name_norm = folio_normalize(path);
     CREATE TRIGGER documents_name_norm_on_insert AFTER INSERT ON documents
     BEGIN
         UPDATE documents SET name_norm = folio_normalize(NEW.path) WHERE id = NEW.id;
     END;
     CREATE TRIGGER documents_name_norm_on_path AFTER UPDATE OF path ON documents
     BEGIN
         UPDATE documents SET name_norm = folio_normalize(NEW.path) WHERE id = NEW.id;
     END;
     CREATE TABLE favorites (
         id TEXT PRIMARY KEY NOT NULL,
         document_id TEXT REFERENCES documents (id) ON DELETE CASCADE,
         path BLOB,
         added_at INTEGER NOT NULL,
         CHECK ((document_id IS NULL) <> (path IS NULL))
     );
     CREATE UNIQUE INDEX favorites_by_document ON favorites (document_id)
         WHERE document_id IS NOT NULL;
     CREATE UNIQUE INDEX favorites_by_path ON favorites (path) WHERE path IS NOT NULL;
     CREATE TABLE recent_documents (
         document_id TEXT PRIMARY KEY NOT NULL
             REFERENCES documents (id) ON DELETE CASCADE,
         opened_at INTEGER NOT NULL,
         position REAL
     );
     CREATE INDEX recent_documents_by_opened_at ON recent_documents (opened_at);",
    // Spec 09: the last destination of "Mover para…" and of imports, a
    // single row that follows its directory like the other path tables.
    "CREATE TABLE last_destination (
         id INTEGER PRIMARY KEY NOT NULL CHECK (id = 1),
         path BLOB NOT NULL
     );",
];

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::*;
    use crate::db::functions::register_functions;
    use crate::db::migrate;

    /// Migrations up to and including spec 05.
    const UNTIL_SPEC_05: usize = 4;

    #[test]
    fn migration_fills_name_norm_of_existing_documents() {
        let mut conn = Connection::open_in_memory().unwrap();
        register_functions(&conn).unwrap();
        migrate(&mut conn, &MIGRATIONS[..UNTIL_SPEC_05]).unwrap();
        conn.execute(
            "INSERT INTO documents (id, path, format, valid, protected, size, modified_at)
             VALUES ('a', CAST('Faculdade/Relatório Q1.pdf' AS BLOB), 'pdf', 1, 0, 0, 0)",
            [],
        )
        .unwrap();

        migrate(&mut conn, MIGRATIONS).unwrap();

        let name_norm: String = conn
            .query_row("SELECT name_norm FROM documents", [], |row| row.get(0))
            .unwrap();
        assert_eq!(name_norm, "relatorio q1.pdf");
    }
}
