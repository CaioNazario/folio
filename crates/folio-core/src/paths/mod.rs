//! Paths as stored in the databases (spec 02).

mod location;
mod os_bytes;
mod relative_path;

pub use location::Location;
pub use relative_path::{InvalidRelativePath, RelativePath};
