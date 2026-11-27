//! The inbox: the documents of the downloads folder, shown without making
//! it a workspace (spec 09, "Entrada").

mod list;

pub use list::{InboxDocument, list_inbox};
