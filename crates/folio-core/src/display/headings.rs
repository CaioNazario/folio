//! Heading ids of the Markdown HTML, the targets of `#anchor` links (spec 07,
//! "Links").

use std::fmt;
use std::sync::Mutex;

use comrak::Anchorizer;
use comrak::adapters::{HeadingAdapter, HeadingMeta};
use comrak::nodes::Sourcepos;

/// Prefix of the heading ids, so a heading never takes an id of the app.
pub const HEADING_ID_PREFIX: &str = "folio-h-";

/// Writes each heading with an id from its text, numbering repeated ones,
/// and without the anchor link comrak would add.
#[derive(Default)]
pub struct HeadingIds {
    anchorizer: Mutex<Anchorizer>,
}

impl HeadingAdapter for HeadingIds {
    fn enter(
        &self,
        output: &mut dyn fmt::Write,
        heading: &HeadingMeta,
        _: Option<Sourcepos>,
    ) -> fmt::Result {
        // A poisoned lock only means another heading panicked: the ids so far
        // are still right.
        let mut anchorizer = self.anchorizer.lock().unwrap_or_else(|e| e.into_inner());
        let id = anchorizer.anchorize(&heading.content);
        write!(
            output,
            r#"<h{} id="{HEADING_ID_PREFIX}{id}">"#,
            heading.level
        )
    }

    fn exit(&self, output: &mut dyn fmt::Write, heading: &HeadingMeta) -> fmt::Result {
        write!(output, "</h{}>", heading.level)
    }
}
