//! The inbox: the documents of the downloads folder, shown without making
//! it a workspace (spec 09, "Entrada").

mod destination;
mod list;
mod location;
mod move_in;

pub use list::{InboxDocument, list_inbox};
pub use location::{default_inbox_dir, inbox_is_hidden};
