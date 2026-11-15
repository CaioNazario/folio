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
];
