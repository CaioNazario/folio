//! What a link clicked inside a document does: the UI asks, the core decides
//! (spec 07, "Links").

use percent_encoding::percent_decode_str;
use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

use crate::Result;
use crate::paths::{href_scheme, resolve_href};
use crate::workspace::Workspace;

/// Where a clicked link leads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum LinkTarget {
    /// Web or e-mail: opens in the system's browser or mail app.
    External { url: String },
    /// Another document of the workspace: opens in Folio.
    Document { id: Uuid },
    /// A file Folio does not open, or a path outside the workspace.
    Unsupported,
    /// A point in the same document.
    Anchor { fragment: String },
    /// Any other scheme (`file:`, `javascript:`...): nothing happens.
    Ignored,
}

impl Workspace {
    /// What `href`, clicked inside the document `id`, leads to.
    pub fn resolve_link(&self, id: Uuid, href: &str) -> Result<LinkTarget> {
        if let Some(fragment) = href.strip_prefix('#') {
            return Ok(anchor(fragment));
        }
        if let Some(scheme) = href_scheme(href) {
            return Ok(external(&scheme, href));
        }
        let document = self.document(id)?;
        // Only the file matters: the fragment and query are dropped.
        let file = href.split(['#', '?']).next().unwrap_or_default();
        let target = resolve_href(document.path.parent().as_ref(), file);
        let Some(found) = target
            .map(|path| self.document_at(&path))
            .transpose()?
            .flatten()
        else {
            return Ok(LinkTarget::Unsupported);
        };
        Ok(LinkTarget::Document { id: found.id })
    }
}

/// A link to a heading of the same document, decoded to match its id.
fn anchor(fragment: &str) -> LinkTarget {
    let fragment = percent_decode_str(fragment).decode_utf8_lossy();
    LinkTarget::Anchor {
        fragment: fragment.into_owned(),
    }
}

/// Web and e-mail addresses leave the app; any other scheme does nothing.
fn external(scheme: &str, href: &str) -> LinkTarget {
    match scheme {
        "http" | "https" | "mailto" => LinkTarget::External {
            url: href.to_string(),
        },
        _ => LinkTarget::Ignored,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::test_docs::write_doc;
    use crate::workspace::test_env::TestEnv;

    fn indexed(workspace: &Workspace, path: &str) -> Uuid {
        let path = write_doc(workspace, path, b"texto");
        workspace.index_file(&path).unwrap().unwrap().id
    }

    #[test]
    fn web_and_mail_links_open_externally() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Notas/Doc.md");

        let targets = [
            "https://exemplo.org/a",
            "HTTP://x.org",
            "mailto:ana@exemplo.org",
        ]
        .map(|href| workspace.resolve_link(doc, href).unwrap());

        assert_eq!(
            targets,
            [
                "https://exemplo.org/a",
                "HTTP://x.org",
                "mailto:ana@exemplo.org"
            ]
            .map(|url| LinkTarget::External {
                url: url.to_string()
            })
        );
    }

    #[test]
    fn relative_link_to_workspace_document_opens_it() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Notas/Doc.md");
        let other = indexed(&workspace, "Outros/Ata.md");

        let target = workspace.resolve_link(doc, "../Outros/Ata.md").unwrap();

        assert_eq!(target, LinkTarget::Document { id: other });
    }

    #[test]
    fn relative_link_decodes_percent_escapes() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Notas/Doc.md");
        let other = indexed(&workspace, "Notas/Ata de reunião.md");

        let target = workspace
            .resolve_link(doc, "Ata%20de%20reuni%C3%A3o.md")
            .unwrap();

        assert_eq!(target, LinkTarget::Document { id: other });
    }

    #[test]
    fn relative_link_with_fragment_opens_the_document() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Notas/Doc.md");
        let other = indexed(&workspace, "Notas/Ata.md");

        let target = workspace.resolve_link(doc, "Ata.md#decisoes").unwrap();

        assert_eq!(target, LinkTarget::Document { id: other });
    }

    #[test]
    fn relative_link_to_other_file_is_unsupported() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Notas/Doc.md");
        write_doc(&workspace, "Notas/Planilha.xlsx", b"PK");

        let target = workspace.resolve_link(doc, "Planilha.xlsx").unwrap();

        assert_eq!(target, LinkTarget::Unsupported);
    }

    #[test]
    fn relative_link_escaping_workspace_is_unsupported() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Notas/Doc.md");

        let targets = ["../../fora.md", "/etc/passwd", "../.folio/folio.db"]
            .map(|href| workspace.resolve_link(doc, href).unwrap());

        assert_eq!(
            targets,
            [
                LinkTarget::Unsupported,
                LinkTarget::Unsupported,
                LinkTarget::Unsupported
            ]
        );
    }

    #[test]
    fn anchor_link_returns_fragment() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Notas/Doc.md");

        let target = workspace.resolve_link(doc, "#conclusao").unwrap();

        assert_eq!(
            target,
            LinkTarget::Anchor {
                fragment: "conclusao".to_string()
            }
        );
    }

    #[test]
    fn anchor_link_returns_percent_decoded_fragment() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Notas/Doc.md");

        let target = workspace
            .resolve_link(doc, "#introdu%C3%A7%C3%A3o")
            .unwrap();

        assert_eq!(
            target,
            LinkTarget::Anchor {
                fragment: "introdução".to_string()
            }
        );
    }

    #[test]
    fn link_with_other_scheme_is_ignored() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "Notas/Doc.md");

        let targets = ["file:///etc/passwd", "javascript:alert(1)", "ftp://x.org"]
            .map(|href| workspace.resolve_link(doc, href).unwrap());

        assert_eq!(
            targets,
            [
                LinkTarget::Ignored,
                LinkTarget::Ignored,
                LinkTarget::Ignored
            ]
        );
    }
}
