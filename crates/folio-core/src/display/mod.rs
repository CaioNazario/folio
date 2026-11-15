//! What the core hands the Reader to show each format (spec 07,
//! "Exibição"). The webview renders; the core only prepares the content.

mod text;
mod workspace;

pub use text::{DISPLAY_TEXT_LIMIT, DisplayText, display_text};
