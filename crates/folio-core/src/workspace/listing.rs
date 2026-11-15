//! Projects and folders as they appear in the tree (spec 03).

use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use crate::paths::{RelativePath, os_bytes};
use crate::workspace::ops::entry_path;
use crate::workspace::validate::FOLIO_DIR;
use crate::workspace::{Workspace, icons};
use crate::{Error, Result};

/// A project (top level) or folder (anywhere below).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DirEntry {
    pub path: RelativePath,
    /// Name for the UI; invalid UTF-8 bytes show as U+FFFD.
    pub name: String,
    pub kind: DirKind,
    /// Lucide icon name chosen by the user, if any.
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub enum DirKind {
    Folder,
    /// Another workspace inside this one: shown blocked, never entered.
    OtherWorkspace,
}

impl Workspace {
    /// Subdirectories of `parent` (the workspace root for `None`), sorted by
    /// name. Hidden directories, `.folio/` and symlinks are left out.
    pub fn list_dirs(&self, parent: Option<&RelativePath>) -> Result<Vec<DirEntry>> {
        let dir = parent.map_or_else(|| self.root().to_path_buf(), |p| p.resolve(self.root()));
        let mut entries = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|e| Error::io(&dir, e))? {
            let entry = entry.map_err(|e| Error::io(&dir, e))?;
            entries.extend(self.dir_entry(parent, &entry)?);
        }
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(entries)
    }

    /// `None` for anything that does not appear as a project or folder.
    fn dir_entry(
        &self,
        parent: Option<&RelativePath>,
        entry: &fs::DirEntry,
    ) -> Result<Option<DirEntry>> {
        let name = entry.file_name();
        // `file_type` does not follow symlinks: a link to a directory is left
        // out, since it may leave the workspace or form a cycle.
        let is_dir = entry
            .file_type()
            .map_err(|e| Error::io(&entry.path(), e))?
            .is_dir();
        if !is_dir || is_hidden(&name) {
            return Ok(None);
        }
        let path = entry_path(parent, &name)?;
        let icon = icons::icon_of(&self.conn(), &path).map_err(|e| self.db_error(e))?;
        Ok(Some(DirEntry {
            path,
            name: name.to_string_lossy().into_owned(),
            kind: dir_kind(&entry.path()),
            icon,
        }))
    }
}

/// Only checks that `<dir>/.folio` exists: the other workspace's state is
/// never read nor changed.
fn dir_kind(dir: &Path) -> DirKind {
    if dir.join(FOLIO_DIR).is_dir() {
        return DirKind::OtherWorkspace;
    }
    DirKind::Folder
}

/// Hidden by convention (`.git`, `.obsidian`), and that covers `.folio/`.
fn is_hidden(name: &OsStr) -> bool {
    os_bytes::as_bytes(name).starts_with(b".")
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::symlink;

    use super::*;
    use crate::integrity::test_support::snapshot;
    use crate::workspace::test_env::TestEnv;

    fn names(entries: &[DirEntry]) -> Vec<&str> {
        entries.iter().map(|entry| entry.name.as_str()).collect()
    }

    #[test]
    fn projects_are_top_level_dirs_including_empty_ones() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        env.dir("Folio/Trabalho/Contratos");
        env.dir("Folio/Faculdade");

        let projects = workspace.list_dirs(None).unwrap();

        assert_eq!(names(&projects), ["Faculdade", "Trabalho"]);
        assert_eq!(
            projects[0].path,
            RelativePath::from_bytes("Faculdade").unwrap()
        );
    }

    #[test]
    fn listing_hides_folio_dir_and_hidden_dirs() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        env.dir("Folio/.git");
        env.dir("Folio/Faculdade");

        let projects = workspace.list_dirs(None).unwrap();

        assert_eq!(names(&projects), ["Faculdade"]);
    }

    #[test]
    fn listing_skips_symlinks_to_dirs() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let outside = env.dir("ForaDoWorkspace");
        symlink(&outside, workspace.root().join("Atalho")).unwrap();

        let projects = workspace.list_dirs(None).unwrap();

        assert_eq!(names(&projects), Vec::<&str>::new());
    }

    #[test]
    fn listing_skips_files() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        fs::write(workspace.root().join("leia-me.pdf"), "pdf").unwrap();

        let projects = workspace.list_dirs(None).unwrap();

        assert_eq!(names(&projects), Vec::<&str>::new());
    }

    #[test]
    fn listing_a_project_returns_its_folders() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        env.dir("Folio/Faculdade/TCC");
        env.dir("Folio/Faculdade/Disciplinas");
        let project = RelativePath::from_bytes("Faculdade").unwrap();

        let folders = workspace.list_dirs(Some(&project)).unwrap();

        assert_eq!(names(&folders), ["Disciplinas", "TCC"]);
        assert_eq!(
            folders[1].path,
            RelativePath::from_bytes("Faculdade/TCC").unwrap()
        );
    }

    #[test]
    fn nested_workspace_is_listed_as_blocked_and_left_untouched() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let nested = env.dir("Folio/Copiado/.folio");
        fs::write(nested.join("folio.db"), "banco do outro workspace").unwrap();
        let before = snapshot(&nested);

        let projects = workspace.list_dirs(None).unwrap();

        assert_eq!(projects[0].kind, DirKind::OtherWorkspace);
        assert_eq!(snapshot(&nested), before);
    }

    #[test]
    fn listing_includes_each_dir_icon() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        env.dir("Folio/Faculdade");
        env.dir("Folio/Trabalho");
        let faculdade = RelativePath::from_bytes("Faculdade").unwrap();
        workspace
            .set_icon(&faculdade, Some("graduation-cap"))
            .unwrap();

        let projects = workspace.list_dirs(None).unwrap();

        let icons: Vec<_> = projects.iter().map(|entry| entry.icon.as_deref()).collect();
        assert_eq!(icons, [Some("graduation-cap"), None]);
    }
}
