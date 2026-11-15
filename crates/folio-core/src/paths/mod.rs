//! Paths as stored in the databases (spec 02).

mod location;
pub(crate) mod os_bytes;
mod relative_path;

pub use location::Location;
pub use relative_path::{InvalidRelativePath, RelativePath};
