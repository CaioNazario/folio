//! The workspace: a vault-style directory marked by `.folio/`, its projects
//! and folders (spec 03).

mod default_dir;
mod icons;
mod listing;
mod meta;
mod name;
mod open;
mod ops;
mod schema;
mod store;
#[cfg(test)]
mod test_env;
mod validate;

pub use default_dir::default_workspace_dir;
pub use listing::{DirEntry, DirKind};
pub use name::{InvalidName, validate_name};
pub use open::Workspace;
pub use store::{FolioDirStore, WorkspaceState, WorkspaceStore};
