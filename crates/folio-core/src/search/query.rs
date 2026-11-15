//! Searching documents by file name (spec 06, "Busca").

use rusqlite::params_from_iter;
use rusqlite::types::Value;

use super::normalize_name;
use super::statement::Statement;
use crate::Result;
use crate::documents::index::document_row;
use crate::documents::{Document, Format};
use crate::paths::RelativePath;
use crate::workspace::Workspace;

/// Results beyond this are left out: the user narrows the query instead.
pub const MAX_RESULTS: usize = 50;

/// What the user typed and the filters chosen.
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub text: String,
    /// Any of these formats; empty for every format.
    pub formats: Vec<Format>,
    pub project: Option<ProjectFilter>,
    /// Inside this directory, subfolders included.
    pub folder: Option<RelativePath>,
}

/// Where the documents of a project filter live.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectFilter {
    /// Inside this project (a first-level directory).
    Project(RelativePath),
    /// At the root of the workspace, outside every project.
    NoProject,
}

impl SearchQuery {
    fn has_filters(&self) -> bool {
        !self.formats.is_empty() || self.project.is_some() || self.folder.is_some()
    }
}

impl Workspace {
    /// Present documents whose file name holds every word of the query.
    pub fn search(&self, query: &SearchQuery) -> Result<Vec<Document>> {
        let words = words(&query.text);
        if words.is_empty() && !query.has_filters() {
            return Ok(Vec::new());
        }
        let mut statement = statement(&words);
        filter_formats(&mut statement, &query.formats);
        filter_project(&mut statement, query.project.as_ref());
        if let Some(folder) = &query.folder {
            filter_within(&mut statement, folder);
        }
        self.run_search(statement)
    }

    fn run_search(&self, statement: Statement) -> Result<Vec<Document>> {
        let sql = statement.sql();
        let conn = self.conn();
        let run = || -> rusqlite::Result<Vec<Document>> {
            conn.prepare(&sql)?
                .query_map(params_from_iter(statement.params()), document_row)?
                .collect()
        };
        run().map_err(|e| self.db_error(e))
    }
}

/// Every word must appear in the name; the first one decides the rank.
fn statement(words: &[String]) -> Statement {
    let mut statement = Statement::default();
    for word in words {
        statement.filter(LIKE, [text(format!("%{}%", escape_like(word)))]);
    }
    if let Some(first) = words.first() {
        let prefix = format!("{}%", escape_like(first));
        statement.order_by(RANK, [text(prefix), text(first.clone())]);
    }
    // Within a rank: most recently opened first, never opened last.
    statement.order_by("opened_at DESC NULLS LAST", []);
    statement.order_by("name_norm", []);
    statement
}

fn filter_formats(statement: &mut Statement, formats: &[Format]) {
    if formats.is_empty() {
        return;
    }
    let marks = vec!["?"; formats.len()].join(", ");
    let params = formats.iter().map(|f| text(f.as_str().to_string()));
    statement.filter(&format!("format IN ({marks})"), params);
}

fn filter_project(statement: &mut Statement, project: Option<&ProjectFilter>) {
    match project {
        None => {}
        Some(ProjectFilter::Project(dir)) => filter_within(statement, dir),
        // No separator in the path: the document sits at the root.
        Some(ProjectFilter::NoProject) => statement.filter("instr(path, X'2F') = 0", []),
    }
}

/// Documents anywhere inside `dir`, as a byte range of the path (spec 02).
fn filter_within(statement: &mut Statement, dir: &RelativePath) {
    let (lower, upper) = dir.descendant_bounds();
    statement.filter(
        "path > ? AND path < ?",
        [Value::Blob(lower), Value::Blob(upper)],
    );
}

fn text(value: String) -> Value {
    Value::Text(value)
}

/// A `name_norm` pattern match; the pattern goes through [`escape_like`].
const LIKE: &str = r"name_norm LIKE ? ESCAPE '\'";

/// Group of a result: 0 when the name starts with the first word, 1 when a
/// later word does, 2 otherwise. Takes the prefix pattern, then the word.
const RANK: &str = r"CASE WHEN name_norm LIKE ? ESCAPE '\' THEN 0
     WHEN folio_word_start(name_norm, ?) THEN 1 ELSE 2 END";

/// `text` with the `LIKE` wildcards taken literally, for `ESCAPE '\'`.
fn escape_like(text: &str) -> String {
    text.replace('\\', r"\\")
        .replace('%', r"\%")
        .replace('_', r"\_")
}

/// The normalized words of `text`, split by whitespace.
fn words(text: &str) -> Vec<String> {
    normalize_name(text)
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::documents::test_docs::write_doc;
    use crate::workspace::test_env::TestEnv;

    fn indexed(workspace: &Workspace, path: &str) -> Uuid {
        let path = write_doc(workspace, path, b"text");
        workspace.index_file(&path).unwrap().unwrap().id
    }

    fn found(workspace: &Workspace, query: &SearchQuery) -> Vec<String> {
        let docs = workspace.search(query).unwrap();
        docs.into_iter().map(|doc| doc.path.display()).collect()
    }

    fn text(text: &str) -> SearchQuery {
        SearchQuery {
            text: text.to_string(),
            ..SearchQuery::default()
        }
    }

    #[test]
    fn search_matches_name_ignoring_accents_and_case() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        indexed(&workspace, "Faculdade/Relatório Q1.txt");
        indexed(&workspace, "Faculdade/Notas.txt");

        assert_eq!(
            found(&workspace, &text("RELATORIO")),
            ["Faculdade/Relatório Q1.txt"]
        );
    }

    #[test]
    fn search_requires_all_words_in_any_order() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        indexed(&workspace, "Trabalho v3.txt");
        indexed(&workspace, "Trabalho final.txt");

        assert_eq!(
            (
                found(&workspace, &text("trab v3")),
                found(&workspace, &text("v3  trab"))
            ),
            (
                vec!["Trabalho v3.txt".to_string()],
                vec!["Trabalho v3.txt".to_string()]
            )
        );
    }

    #[test]
    fn search_with_blank_text_and_no_filters_returns_nothing() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        indexed(&workspace, "Notas.txt");

        assert_eq!(found(&workspace, &text("   ")), Vec::<String>::new());
    }

    #[test]
    fn search_treats_percent_and_underscore_as_literals() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        indexed(&workspace, "Desconto 10%.txt");
        indexed(&workspace, "Desconto 100.txt");
        indexed(&workspace, "nota_1.txt");
        indexed(&workspace, "notas1.txt");

        assert_eq!(
            (
                found(&workspace, &text("10%")),
                found(&workspace, &text("a_1"))
            ),
            (
                vec!["Desconto 10%.txt".to_string()],
                vec!["nota_1.txt".to_string()]
            )
        );
    }

    #[test]
    fn search_does_not_match_folder_names() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        indexed(&workspace, "Relatorios/Notas.txt");

        assert_eq!(found(&workspace, &text("relatorios")), Vec::<String>::new());
    }

    #[test]
    fn search_ranks_name_starting_with_first_word_first() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        indexed(&workspace, "Meu relatorio.txt");
        indexed(&workspace, "Relatorio final.txt");

        assert_eq!(
            found(&workspace, &text("relat")),
            ["Relatorio final.txt", "Meu relatorio.txt"]
        );
    }

    #[test]
    fn search_ranks_word_starting_with_first_word_before_plain_contains() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        indexed(&workspace, "Contrato.txt");
        indexed(&workspace, "Notas_ratificadas.txt");
        indexed(&workspace, "Ratos.txt");

        assert_eq!(
            found(&workspace, &text("rat")),
            ["Ratos.txt", "Notas_ratificadas.txt", "Contrato.txt"]
        );
    }

    #[test]
    fn search_returns_at_most_50_results() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        for n in 0..51 {
            indexed(&workspace, &format!("Nota {n}.txt"));
        }

        assert_eq!(found(&workspace, &text("nota")).len(), 50);
    }

    #[test]
    fn search_hides_missing_documents() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        indexed(&workspace, "Nota antiga.txt");
        let gone = indexed(&workspace, "Nota velha.txt");
        workspace.trash_document(gone).unwrap();

        assert_eq!(found(&workspace, &text("nota")), ["Nota antiga.txt"]);
    }

    #[test]
    fn search_includes_invalid_documents() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        // Not a PDF inside: indexed as invalid (spec 04).
        indexed(&workspace, "Fatura.pdf");

        let docs = workspace.search(&text("fatura")).unwrap();

        assert_eq!(
            docs.iter().map(|doc| doc.valid).collect::<Vec<_>>(),
            [false]
        );
    }

    #[test]
    fn search_filters_by_formats() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        for path in ["A.pdf", "B.md", "C.txt", "D.docx"] {
            indexed(&workspace, path);
        }
        let query = SearchQuery {
            formats: vec![Format::Pdf, Format::Md],
            ..SearchQuery::default()
        };

        assert_eq!(found(&workspace, &query), ["A.pdf", "B.md"]);
    }

    fn rel(text: &str) -> RelativePath {
        RelativePath::from_bytes(text).unwrap()
    }

    /// One document in each kind of place a filter tells apart.
    fn spread_workspace(env: &TestEnv) -> Workspace {
        let workspace = env.open_workspace();
        for path in [
            "Faculdade/A.txt",
            "Faculdade/Sub/B.pdf",
            "Faculdade2/C.txt",
            "D.txt",
        ] {
            indexed(&workspace, path);
        }
        workspace
    }

    #[test]
    fn search_filters_by_project() {
        let env = TestEnv::new();
        let workspace = spread_workspace(&env);
        let query = SearchQuery {
            project: Some(ProjectFilter::Project(rel("Faculdade"))),
            ..SearchQuery::default()
        };

        assert_eq!(
            found(&workspace, &query),
            ["Faculdade/A.txt", "Faculdade/Sub/B.pdf"]
        );
    }

    #[test]
    fn search_filters_by_no_project() {
        let env = TestEnv::new();
        let workspace = spread_workspace(&env);
        let query = SearchQuery {
            project: Some(ProjectFilter::NoProject),
            ..SearchQuery::default()
        };

        assert_eq!(found(&workspace, &query), ["D.txt"]);
    }

    #[test]
    fn search_filters_by_folder_including_subfolders() {
        let env = TestEnv::new();
        let workspace = spread_workspace(&env);
        indexed(&workspace, "Faculdade/Sub/Mais/E.md");
        let query = SearchQuery {
            folder: Some(rel("Faculdade/Sub")),
            ..SearchQuery::default()
        };

        assert_eq!(
            found(&workspace, &query),
            ["Faculdade/Sub/B.pdf", "Faculdade/Sub/Mais/E.md"]
        );
    }

    #[test]
    fn search_combines_filters_with_text() {
        let env = TestEnv::new();
        let workspace = spread_workspace(&env);
        for path in ["Faculdade/Nota.pdf", "Faculdade/Nota.txt", "Nota.pdf"] {
            indexed(&workspace, path);
        }
        let query = SearchQuery {
            text: "nota".to_string(),
            formats: vec![Format::Pdf],
            project: Some(ProjectFilter::Project(rel("Faculdade"))),
            folder: None,
        };

        assert_eq!(found(&workspace, &query), ["Faculdade/Nota.pdf"]);
    }

    #[test]
    fn search_finds_renamed_document_by_new_name_only() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = indexed(&workspace, "Rascunho.txt");

        workspace.rename_document(id, "Definitivo").unwrap();

        assert_eq!(
            (
                found(&workspace, &text("rascunho")),
                found(&workspace, &text("definitivo"))
            ),
            (vec![], vec!["Definitivo.txt".to_string()])
        );
    }

    #[test]
    fn search_finds_document_renamed_outside_folio_after_reconnection() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        indexed(&workspace, "Rascunho.txt");
        let root = workspace.root();
        std::fs::rename(root.join("Rascunho.txt"), root.join("Definitivo.txt")).unwrap();

        let summary = workspace.reconcile_dir(None).unwrap();

        assert_eq!(
            (summary.reconnected, found(&workspace, &text("definitivo"))),
            (1, vec!["Definitivo.txt".to_string()])
        );
    }

    #[test]
    fn search_orders_group_by_most_recently_opened_then_name() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let ids: Vec<Uuid> = ["Nota c.txt", "Nota a.txt", "Nota b.txt", "Nota d.txt"]
            .map(|path| indexed(&workspace, path))
            .into();
        env.clock.set(2_000);
        workspace.record_opened(ids[3]).unwrap();
        env.clock.set(3_000);
        workspace.record_opened(ids[2]).unwrap();

        assert_eq!(
            found(&workspace, &text("nota")),
            ["Nota b.txt", "Nota d.txt", "Nota a.txt", "Nota c.txt"]
        );
    }
}
