//! Guarantees that no Folio operation destroys or corrupts a user document
//! (spec 01).

mod create_dir;
mod move_by_copy;
mod move_with_suffix;
mod publish;
mod rename;
mod suffix;
#[cfg(test)]
pub(crate) mod test_support;
mod trash;
mod workspace_guard;

pub use create_dir::create_dir_no_replace;
pub use move_by_copy::move_by_copy;
pub use move_with_suffix::move_with_suffix;
pub use publish::publish_new_file;
pub use rename::rename_no_replace;
pub use suffix::with_suffix;
pub use trash::{SystemTrash, Trash, delete_to_trash};
pub use workspace_guard::WorkspaceGuard;
