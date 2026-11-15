//! What the core hands the Reader to show each format (spec 07,
//! "Exibição"). The webview renders; the core only prepares the content.

mod image_files;
mod images;
mod links;
mod markdown;
mod text;
mod view;
mod workspace;

pub use links::LinkTarget;
pub use markdown::MarkdownContent;
pub use text::{DISPLAY_TEXT_LIMIT, DisplayText, display_text};
pub use view::DocumentView;
