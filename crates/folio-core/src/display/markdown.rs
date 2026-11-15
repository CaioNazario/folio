//! Markdown as safe HTML for the stage (spec 07, "Exibição" and "Imagens no
//! Markdown").

use std::io::{self, Read};

use serde::Serialize;
use ts_rs::TS;

use comrak::options::Plugins;
use comrak::{Arena, format_html_with_plugins, parse_document};

use super::display_text;
use super::headings::HeadingIds;
use super::images::{MARKER_SCHEME, mark_images, replace_markers};
use crate::readers::markdown_options;

/// A Markdown document ready for the stage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct MarkdownContent {
    pub html: String,
    /// The file was larger than the limit and only its start is here.
    pub truncated: bool,
}

/// The Markdown in `source`, reading at most `limit` bytes, as HTML.
pub fn markdown_display(
    source: impl Read,
    limit: usize,
    resolve: &mut dyn FnMut(&str) -> ImageOutcome,
) -> io::Result<MarkdownContent> {
    let text = display_text(source, limit)?;
    Ok(MarkdownContent {
        html: render_markdown(&text.text, resolve),
        truncated: text.truncated,
    })
}

/// What the core puts in place of an image of the document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageOutcome {
    /// Shown inline as a `data:` URL the core generated.
    Embedded { mime: &'static str, bytes: Vec<u8> },
    /// Not shown: a placeholder with the alt text; remote images also link
    /// their address.
    Placeholder {
        reason: Unshown,
        link: Option<String>,
    },
}

/// Why an image is not shown, as the placeholder's `data-folio-image`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unshown {
    Remote,
    Missing,
    Outside,
    Unsupported,
    TooLarge,
}

/// `markdown` rendered without embedded HTML and sanitized; `resolve` says
/// what each image address becomes.
pub fn render_markdown(markdown: &str, resolve: &mut dyn FnMut(&str) -> ImageOutcome) -> String {
    let arena = Arena::new();
    let options = markdown_options();
    let root = parse_document(&arena, markdown, &options);
    let images = mark_images(root);
    let mut html = String::new();
    // Writing to a `String` cannot fail.
    let headings = HeadingIds::default();
    let mut plugins = Plugins::default();
    plugins.render.heading_adapter = Some(&headings);
    _ = format_html_with_plugins(root, &options, &mut html, &plugins);
    let outcomes: Vec<ImageOutcome> = images.iter().map(|image| resolve(&image.url)).collect();
    replace_markers(sanitize(&html), &images, &outcomes)
}

const HEADINGS: [&str; 6] = ["h1", "h2", "h3", "h4", "h5", "h6"];

/// Ammonia's defaults, keeping the image markers in `src` and the ids of
/// headings.
fn sanitize(html: &str) -> String {
    let mut builder = ammonia::Builder::default();
    builder.add_url_schemes([MARKER_SCHEME]);
    for heading in HEADINGS {
        builder.add_tag_attributes(heading, ["id"]);
    }
    builder.clean(html).to_string()
}

impl Unshown {
    /// The value of the placeholder's `data-folio-image`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Remote => "remote",
            Self::Missing => "missing",
            Self::Outside => "outside",
            Self::Unsupported => "unsupported",
            Self::TooLarge => "too-large",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_images(url: &str) -> ImageOutcome {
        panic!("unexpected image {url}")
    }

    fn render(markdown: &str) -> String {
        render_markdown(markdown, &mut no_images)
    }

    #[test]
    fn md_html_renders_markdown() {
        let html = render("# Título\n\nUm **forte** e [link](https://exemplo.org).\n");

        assert!(html.contains(r#"<h1 id="folio-h-título">Título"#), "{html}");
        assert!(html.contains("<strong>forte</strong>"), "{html}");
        assert!(html.contains(r#"<a href="https://exemplo.org""#), "{html}");
    }

    #[test]
    fn md_html_gives_headings_an_id_from_their_text() {
        let html = render("# Introdução\n\nTexto\n");

        assert!(html.contains(r#"<h1 id="folio-h-introdução">"#), "{html}");
    }

    #[test]
    fn md_html_numbers_repeated_heading_ids() {
        let html = render("## Notas\n\n## Notas\n");

        assert!(html.contains(r#"<h2 id="folio-h-notas">"#), "{html}");
        assert!(html.contains(r#"<h2 id="folio-h-notas-1">"#), "{html}");
    }

    #[test]
    fn md_html_headings_have_no_anchor_link() {
        let html = render("# Título\n");

        assert_eq!(html.trim(), r#"<h1 id="folio-h-título">Título</h1>"#);
    }

    #[test]
    fn md_html_drops_scripts_and_event_handlers() {
        let html = render(
            "<script>alert(1)</script>\n\nTexto <img src=x onerror=alert(2)> e [clique](javascript:alert(3)).\n",
        );

        for forbidden in ["<script", "onerror", "javascript:", "alert"] {
            assert!(!html.contains(forbidden), "{forbidden} in {html}");
        }
    }

    #[test]
    fn md_html_drops_data_urls_written_in_the_document() {
        let mut unsupported = |_: &str| ImageOutcome::Placeholder {
            reason: Unshown::Unsupported,
            link: None,
        };
        let markdown = "![x](data:image/png;base64,AAAA) [l](data:text/html;base64,PHNjcmlwdD4=)\n";

        let html = render_markdown(markdown, &mut unsupported);

        assert!(!html.contains("data:"), "{html}");
    }

    #[test]
    fn md_html_leaves_out_front_matter() {
        let html = render("---\ntitle: Segredo\n---\n\nCorpo\n");

        assert!(!html.contains("Segredo"), "{html}");
        assert!(html.contains("<p>Corpo</p>"), "{html}");
    }

    #[test]
    fn md_html_of_large_file_is_cut_and_flagged() {
        let markdown = "Um\n\nDois\n\nTrês\n";

        let shown = markdown_display(markdown.as_bytes(), 9, &mut no_images).unwrap();

        assert!(
            shown.html.contains("Dois") && !shown.html.contains("Três"),
            "{}",
            shown.html
        );
        assert!(shown.truncated);
    }
}
