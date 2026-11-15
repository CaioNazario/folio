//! SQLite infrastructure shared by the global and workspace databases
//! (spec 02).

#[cfg(unix)]
mod global;
mod migrate;
mod open;

#[cfg(unix)]
pub use global::{GlobalDb, KnownWorkspace};
pub use migrate::migrate;
pub use open::{OpenedDatabase, open_database};
