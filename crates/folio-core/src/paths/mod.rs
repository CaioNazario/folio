//! Paths as stored in the databases (spec 02).

#[cfg(unix)]
mod location;
mod relative_path;

#[cfg(unix)]
pub use location::Location;
pub use relative_path::{InvalidRelativePath, RelativePath};
