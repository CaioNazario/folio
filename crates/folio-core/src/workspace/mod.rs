//! The workspace: a vault-style directory marked by `.folio/`, its projects
//! and folders (spec 03).

mod contents;
mod default_dir;
pub(crate) mod icons;
mod listing;
mod meta;
mod name;
mod open;
pub(crate) mod ops;
mod schema;
mod session;
mod slot;
mod store;
mod summary;
#[cfg(test)]
pub(crate) mod test_env;
mod validate;

pub use contents::{DirContents, FolderEntry};
pub use default_dir::default_workspace_dir;
pub use listing::{DirEntry, DirKind};
pub use name::{InvalidName, validate_name};
pub use open::Workspace;
pub use slot::{Opened, WorkspaceSlot};
pub use store::{FolioDirStore, WorkspaceState, WorkspaceStore};
pub use summary::WorkspaceSummary;
