//! SQLite infrastructure shared by the global and workspace databases
//! (spec 02).

mod global;
mod migrate;
mod open;

pub use global::{GlobalDb, KnownWorkspace};
pub use migrate::migrate;
pub use open::{OpenedDatabase, open_database};
