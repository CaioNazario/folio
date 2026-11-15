//! Operations on projects and folders (spec 03). All of them follow spec 01:
//! real operation on disk, never overwriting, disk before the index.

use std::ffi::OsStr;

use crate::integrity::{create_dir_no_replace, delete_to_trash, rename_no_replace};
use crate::paths::{RelativePath, os_bytes};
use crate::workspace::{Workspace, icons, validate_name};
use crate::{Error, Result};

impl Workspace {
    /// Creates the project `name` at the workspace root.
    pub fn create_project(&self, name: &str) -> Result<RelativePath> {
        self.create_dir_in(None, name)
    }

    /// Creates the folder `name` inside the project or folder `parent`.
    pub fn create_folder(&self, parent: &RelativePath, name: &str) -> Result<RelativePath> {
        self.create_dir_in(Some(parent), name)
    }

    /// Renames the project or folder at `path` to `new_name`, in place.
    pub fn rename_dir(&self, path: &RelativePath, new_name: &str) -> Result<RelativePath> {
        let name = validate_name(new_name)?;
        let new_path = entry_path(path.parent().as_ref(), OsStr::new(&name))?;
        self.relocate(path, new_path)
    }

    /// Moves the project or folder at `path` into `new_parent` (the root for
    /// `None`, where it becomes a project), keeping its name.
    pub fn move_dir(
        &self,
        path: &RelativePath,
        new_parent: Option<&RelativePath>,
    ) -> Result<RelativePath> {
        // Checked before touching the disk: the kernel answers EINVAL, which
        // would otherwise read as a filesystem without atomic rename.
        if new_parent.is_some_and(|parent| parent.is_within(path)) {
            return Err(Error::MoveIntoItself);
        }
        let new_path = entry_path(new_parent, path.file_name())?;
        self.relocate(path, new_path)
    }

    /// Sends the project or folder at `path`, with everything inside, to the
    /// system trash. Never deletes permanently (spec 01).
    pub fn delete_dir(&self, path: &RelativePath) -> Result<()> {
        delete_to_trash(&*self.trash, &path.resolve(self.root()))
    }

    /// Renames on disk without overwriting, then carries what the index ties
    /// to the old path (spec 01: disk before the index).
    fn relocate(&self, from: &RelativePath, to: RelativePath) -> Result<RelativePath> {
        rename_no_replace(&from.resolve(self.root()), &to.resolve(self.root()))?;
        self.update_index_after_disk(|conn| icons::carry_icons(conn, from, &to));
        Ok(to)
    }

    /// A name already taken is an error: the user chose it, so Folio never
    /// renames it with a suffix (spec 01).
    fn create_dir_in(&self, parent: Option<&RelativePath>, name: &str) -> Result<RelativePath> {
        let path = entry_path(parent, OsStr::new(&validate_name(name)?))?;
        create_dir_no_replace(&path.resolve(self.root()))?;
        Ok(path)
    }
}

/// The path of the entry `name` inside `parent`, or at the root for `None`.
pub(super) fn entry_path(parent: Option<&RelativePath>, name: &OsStr) -> Result<RelativePath> {
    let path = match parent {
        Some(parent) => parent.join(name)?,
        None => RelativePath::from_bytes(os_bytes::as_bytes(name))?,
    };
    Ok(path)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::integrity::test_support::snapshot;
    use crate::workspace::InvalidName;
    use crate::workspace::test_env::TestEnv;

    #[test]
    fn create_project_creates_dir_at_root() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();

        let project = workspace.create_project(" Faculdade ").unwrap();

        assert_eq!(project, RelativePath::from_bytes("Faculdade").unwrap());
        assert!(workspace.root().join("Faculdade").is_dir());
    }

    #[test]
    fn create_project_fails_with_already_exists() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        fs::write(workspace.root().join("Faculdade"), "um arquivo").unwrap();

        let result = workspace.create_project("Faculdade");

        assert!(matches!(result, Err(Error::AlreadyExists(_))));
    }

    #[test]
    fn create_project_rejects_invalid_name() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();

        let result = workspace.create_project(".rascunhos");

        assert!(matches!(
            result,
            Err(Error::InvalidName(InvalidName::Hidden))
        ));
        assert!(!workspace.root().join(".rascunhos").exists());
    }

    fn rel(text: &str) -> RelativePath {
        RelativePath::from_bytes(text).unwrap()
    }

    #[test]
    fn create_folder_creates_dir_inside_parent() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let project = workspace.create_project("Faculdade").unwrap();

        let folder = workspace.create_folder(&project, "TCC").unwrap();

        assert_eq!(folder, rel("Faculdade/TCC"));
        assert!(workspace.root().join("Faculdade/TCC").is_dir());
    }

    #[test]
    fn rename_dir_renames_on_disk() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        env.dir("Folio/Faculdade/TCC");

        let renamed = workspace
            .rename_dir(&rel("Faculdade/TCC"), "Monografia")
            .unwrap();

        assert_eq!(renamed, rel("Faculdade/Monografia"));
        assert!(workspace.root().join("Faculdade/Monografia").is_dir());
        assert!(!workspace.root().join("Faculdade/TCC").exists());
    }

    #[test]
    fn rename_dir_fails_with_already_exists_and_changes_nothing() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        env.dir("Folio/Faculdade/TCC");
        env.dir("Folio/Faculdade/Monografia");
        let before = snapshot(workspace.root());

        let result = workspace.rename_dir(&rel("Faculdade/TCC"), "Monografia");

        assert!(matches!(result, Err(Error::AlreadyExists(_))));
        assert_eq!(snapshot(workspace.root()), before);
        assert!(workspace.root().join("Faculdade/TCC").is_dir());
    }

    fn icon(workspace: &Workspace, path: &str) -> Option<String> {
        crate::workspace::icons::icon_of(&workspace.conn(), &rel(path)).unwrap()
    }

    /// Creates each directory and gives it the paired icon.
    fn dirs_with_icons(env: &TestEnv, workspace: &Workspace, icons: &[(&str, &str)]) {
        for (path, name) in icons {
            env.dir(&format!("Folio/{path}"));
            workspace.set_icon(&rel(path), Some(name)).unwrap();
        }
    }

    #[test]
    fn rename_dir_carries_icons_of_dir_and_descendants() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let icons = [
            ("Faculdade", "graduation-cap"),
            ("Faculdade/TCC/Fontes", "book"),
        ];
        dirs_with_icons(&env, &workspace, &icons);

        workspace
            .rename_dir(&rel("Faculdade"), "Universidade")
            .unwrap();

        assert_eq!(
            icon(&workspace, "Universidade"),
            Some("graduation-cap".into())
        );
        assert_eq!(
            icon(&workspace, "Universidade/TCC/Fontes"),
            Some("book".into())
        );
        assert_eq!(icon(&workspace, "Faculdade"), None);
    }

    #[test]
    fn rename_dir_leaves_sibling_with_same_prefix_untouched() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let icons = [
            ("Fac", "Fac"),
            ("Faculdade", "Faculdade"),
            ("Fac-velho", "Fac-velho"),
        ];
        dirs_with_icons(&env, &workspace, &icons);

        workspace.rename_dir(&rel("Fac"), "Novo").unwrap();

        assert_eq!(icon(&workspace, "Faculdade"), Some("Faculdade".into()));
        assert_eq!(icon(&workspace, "Fac-velho"), Some("Fac-velho".into()));
        assert_eq!(icon(&workspace, "Novo"), Some("Fac".into()));
    }

    #[test]
    fn move_dir_into_another_project() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        env.dir("Folio/Faculdade/TCC");
        env.dir("Folio/Trabalho");

        let moved = workspace
            .move_dir(&rel("Faculdade/TCC"), Some(&rel("Trabalho")))
            .unwrap();

        assert_eq!(moved, rel("Trabalho/TCC"));
        assert!(workspace.root().join("Trabalho/TCC").is_dir());
        assert!(!workspace.root().join("Faculdade/TCC").exists());
    }

    #[test]
    fn move_folder_to_root_makes_it_a_project() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        env.dir("Folio/Faculdade/TCC");

        let moved = workspace.move_dir(&rel("Faculdade/TCC"), None).unwrap();

        assert_eq!(moved, rel("TCC"));
        assert!(workspace.root().join("TCC").is_dir());
    }

    #[test]
    fn move_dir_into_itself_or_descendant_fails_with_move_into_itself() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        env.dir("Folio/Faculdade/TCC");
        let before = snapshot(workspace.root());

        let into_itself = workspace.move_dir(&rel("Faculdade"), Some(&rel("Faculdade")));
        let into_child = workspace.move_dir(&rel("Faculdade"), Some(&rel("Faculdade/TCC")));

        assert!(matches!(into_itself, Err(Error::MoveIntoItself)));
        assert!(matches!(into_child, Err(Error::MoveIntoItself)));
        assert_eq!(snapshot(workspace.root()), before);
    }

    #[test]
    fn move_dir_fails_with_already_exists_when_name_taken_in_destination() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        env.dir("Folio/Faculdade/TCC");
        env.dir("Folio/Trabalho/TCC");

        let result = workspace.move_dir(&rel("Faculdade/TCC"), Some(&rel("Trabalho")));

        assert!(matches!(result, Err(Error::AlreadyExists(_))));
        assert!(workspace.root().join("Faculdade/TCC").is_dir());
    }

    #[test]
    fn move_dir_carries_icons_of_dir_and_descendants() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        env.dir("Folio/Trabalho");
        let icons = [
            ("Faculdade/TCC", "scroll"),
            ("Faculdade/TCC/Fontes", "book"),
        ];
        dirs_with_icons(&env, &workspace, &icons);

        let trabalho = rel("Trabalho");
        workspace
            .move_dir(&rel("Faculdade/TCC"), Some(&trabalho))
            .unwrap();

        assert_eq!(icon(&workspace, "Trabalho/TCC"), Some("scroll".into()));
        assert_eq!(icon(&workspace, "Trabalho/TCC/Fontes"), Some("book".into()));
    }

    #[test]
    fn delete_dir_sends_whole_dir_to_trash() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        env.dir("Folio/Faculdade/TCC");

        workspace.delete_dir(&rel("Faculdade")).unwrap();

        assert_eq!(env.trash.sent(), [workspace.root().join("Faculdade")]);
    }

    #[test]
    fn disk_operation_stands_when_index_update_fails() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        env.dir("Folio/Faculdade");
        workspace.conn().execute_batch("DROP TABLE icons").unwrap();

        let renamed = workspace.rename_dir(&rel("Faculdade"), "Universidade");

        assert_eq!(renamed.unwrap(), rel("Universidade"));
        assert!(workspace.root().join("Universidade").is_dir());
    }
}
