//! Publishing a Markdown derivative with its `.assets` folder (spec 08,
//! "Publicação com pasta de imagens").

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::integrity::{publish_new_file, rename_no_replace, with_suffix};
use crate::model::DocumentModel;
use crate::writers::{MdDocument, write_md};
use crate::{Error, Result};

/// Writes the Markdown of `model` as `dest_dir/name`, with its images in a
/// `.assets` folder next to it, and returns the Markdown's final path.
pub fn publish_md(
    tmp_dir: &Path,
    dest_dir: &Path,
    name: &OsStr,
    model: &DocumentModel,
) -> Result<PathBuf> {
    let plain = write_md(model, "");
    if plain.images.is_empty() {
        return publish_new_file(tmp_dir, dest_dir, name, |file| {
            file.write_all(plain.text.as_bytes())
        });
    }
    let dirs = Dirs {
        tmp: tmp_dir,
        dest: dest_dir,
    };
    publish_with_assets(&dirs, name, model, rename_no_replace)
}

/// Where the Markdown and its folder are born and where they go.
struct Dirs<'a> {
    tmp: &'a Path,
    dest: &'a Path,
}

/// Tries `name`, then `name (1)`... until both the Markdown and its folder
/// are free. `place_md` publishes the Markdown file.
fn publish_with_assets(
    dirs: &Dirs,
    name: &OsStr,
    model: &DocumentModel,
    place_md: impl Fn(&Path, &Path) -> Result<()>,
) -> Result<PathBuf> {
    let mut n = 0;
    loop {
        let names = Names::new(name, n);
        match publish_attempt(dirs, &names, model, &place_md) {
            Err(Error::AlreadyExists(_)) => n += 1,
            other => return other,
        }
    }
}

/// The Markdown's file name and its folder's, with the same suffix.
struct Names {
    md: OsString,
    assets: OsString,
}

impl Names {
    fn new(name: &OsStr, n: u32) -> Self {
        let md = if n == 0 {
            name.to_os_string()
        } else {
            with_suffix(name, n)
        };
        let mut assets = Path::new(&md).file_stem().unwrap_or(&md).to_os_string();
        assets.push(".assets");
        Self { md, assets }
    }
}

/// One try with `names`: the folder goes first, then the Markdown. When the
/// Markdown cannot be published, the folder goes back to the stage.
fn publish_attempt(
    dirs: &Dirs,
    names: &Names,
    model: &DocumentModel,
    place_md: impl Fn(&Path, &Path) -> Result<()>,
) -> Result<PathBuf> {
    let (md_to, assets_to) = (dirs.dest.join(&names.md), dirs.dest.join(&names.assets));
    if [&md_to, &assets_to]
        .iter()
        .any(|path| fs::symlink_metadata(path).is_ok())
    {
        return Err(Error::AlreadyExists(md_to));
    }
    let stage = Stage::write(dirs.tmp, model, &names.assets)?;
    rename_no_replace(&stage.assets(), &assets_to)?;
    if let Err(error) = place_md(&stage.md(), &md_to) {
        stage.take_back(&assets_to);
        return Err(error);
    }
    Ok(md_to)
}

/// The Markdown and its folder, written in a temp directory of `.folio/tmp/`.
/// Whatever is left in it when dropped is deleted.
struct Stage(TempDir);

impl Stage {
    fn write(tmp_dir: &Path, model: &DocumentModel, assets: &OsStr) -> Result<Self> {
        let stage = Self(TempDir::new_in(tmp_dir).map_err(|e| Error::io(tmp_dir, e))?);
        let md = write_md(model, &assets.to_string_lossy());
        stage.fill(&md).map_err(|e| Error::io(stage.0.path(), e))?;
        Ok(stage)
    }

    fn fill(&self, md: &MdDocument) -> std::io::Result<()> {
        fs::create_dir(self.assets())?;
        for image in &md.images {
            write_synced(&self.assets().join(&image.name), &image.bytes)?;
        }
        write_synced(&self.md(), md.text.as_bytes())
    }

    fn md(&self) -> PathBuf {
        self.0.path().join("documento.md")
    }

    fn assets(&self) -> PathBuf {
        self.0.path().join("assets")
    }

    /// Moves the published folder back, so the workspace keeps nothing of a
    /// failed publication.
    fn take_back(&self, published: &Path) {
        if let Err(error) = rename_no_replace(published, &self.assets()) {
            log::warn!("cannot take back {}: {error}", published.display());
        }
    }
}

fn write_synced(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = fs::File::create_new(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Block;

    fn model_with_image() -> DocumentModel {
        let image = Block::Image {
            bytes: b"\x89PNG\r\n\x1a\n".to_vec(),
            mime: "image/png".to_string(),
            alt: "Diagrama".to_string(),
        };
        DocumentModel {
            blocks: vec![image],
            ..DocumentModel::default()
        }
    }

    fn is_empty(dir: &Path) -> bool {
        fs::read_dir(dir).unwrap().next().is_none()
    }

    #[test]
    fn failed_md_publication_leaves_no_folder_in_workspace() {
        let root = tempfile::tempdir().unwrap();
        let (tmp, dest) = (root.path().join("tmp"), root.path().join("Notas"));
        fs::create_dir(&tmp).unwrap();
        fs::create_dir(&dest).unwrap();
        let dirs = Dirs {
            tmp: &tmp,
            dest: &dest,
        };
        let failing = |_: &Path, to: &Path| Err(Error::io(to, std::io::ErrorKind::StorageFull));

        let result =
            publish_with_assets(&dirs, OsStr::new("Aula.md"), &model_with_image(), failing);

        assert!(matches!(result, Err(Error::Io { .. })));
        assert!(is_empty(&dest));
        assert!(is_empty(&tmp));
    }
}
