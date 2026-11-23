//! Image files a Markdown document points to (spec 07, "Imagens no
//! Markdown").

use std::fs;

use super::markdown::{ImageOutcome, Unshown};
use crate::paths::{RelativePath, href_scheme, resolve_href};
use crate::readers::{ImageLookup, image_mime};
use crate::workspace::Workspace;

/// A single image larger than this is not embedded.
pub const IMAGE_LIMIT: u64 = 10 * 1024 * 1024;
/// Images of one document past this total are not embedded.
pub const DOCUMENT_IMAGES_LIMIT: u64 = 50 * 1024 * 1024;

/// Resolves the images of one document, keeping its running total.
pub struct ImageResolver<'a> {
    workspace: &'a Workspace,
    dir: Option<RelativePath>,
    embedded: u64,
}

impl<'a> ImageResolver<'a> {
    /// Images of the document at `document`, relative to its directory.
    pub fn new(workspace: &'a Workspace, document: &RelativePath) -> Self {
        Self {
            workspace,
            dir: document.parent(),
            embedded: 0,
        }
    }

    /// What the image at `url` becomes.
    pub fn resolve(&mut self, url: &str) -> ImageOutcome {
        if let Some(scheme) = href_scheme(url) {
            return foreign(&scheme, url);
        }
        let Some(path) = resolve_href(self.dir.as_ref(), url) else {
            return unshown(Unshown::Outside);
        };
        self.embed(&path).unwrap_or_else(unshown)
    }

    /// What the image at `url` is for a reader (spec 08, "Imagens"): the
    /// same rules as the display, so a conversion takes exactly the images
    /// the Reader shows.
    pub fn lookup(&mut self, url: &str) -> ImageLookup {
        match self.resolve(url) {
            ImageOutcome::Embedded { mime, bytes } => ImageLookup::Found {
                bytes,
                mime: mime.to_string(),
            },
            ImageOutcome::Placeholder {
                reason: Unshown::Remote,
                ..
            } => ImageLookup::Remote,
            ImageOutcome::Placeholder { .. } => ImageLookup::Unavailable,
        }
    }

    fn embed(&mut self, path: &RelativePath) -> Result<ImageOutcome, Unshown> {
        let full = path.resolve(self.workspace.root());
        // The size comes first: a large file is never read.
        let size = fs::metadata(&full).map_err(|_| Unshown::Missing)?.len();
        if size > IMAGE_LIMIT || self.embedded + size > DOCUMENT_IMAGES_LIMIT {
            return Err(Unshown::TooLarge);
        }
        let bytes = fs::read(&full).map_err(|_| Unshown::Missing)?;
        let mime = image_mime(&bytes).ok_or(Unshown::Unsupported)?;
        self.embedded += bytes.len() as u64;
        Ok(ImageOutcome::Embedded { mime, bytes })
    }
}

/// An address with a scheme is never read: web images are not downloaded
/// (spec 07), and anything else is not an image Folio shows.
fn foreign(scheme: &str, url: &str) -> ImageOutcome {
    if !matches!(scheme, "http" | "https") {
        return unshown(Unshown::Unsupported);
    }
    ImageOutcome::Placeholder {
        reason: Unshown::Remote,
        link: Some(url.to_string()),
    }
}

fn unshown(reason: Unshown) -> ImageOutcome {
    ImageOutcome::Placeholder { reason, link: None }
}
