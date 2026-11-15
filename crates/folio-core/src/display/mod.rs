//! What the core hands the Reader to show each format (spec 07,
//! "Exibição"). The webview renders; the core only prepares the content.

mod headings;
mod image_files;
mod images;
mod links;
mod markdown;
mod properties;
mod text;
mod view;
mod workspace;

pub use links::LinkTarget;
pub use markdown::MarkdownContent;
pub use properties::DocumentProperties;
pub use text::{DISPLAY_TEXT_LIMIT, DisplayText, display_text};
pub use view::DocumentView;
