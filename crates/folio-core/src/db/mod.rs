//! SQLite infrastructure shared by the global and workspace databases
//! (spec 02).

pub(crate) mod columns;
mod global;
mod migrate;
mod open;
pub(crate) mod rebase;

pub use global::{GlobalDb, KnownWorkspace};
pub use migrate::migrate;
pub use open::{OpenedDatabase, open_database};
