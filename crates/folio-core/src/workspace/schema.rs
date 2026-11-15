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
];
