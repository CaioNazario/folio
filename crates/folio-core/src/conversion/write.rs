//! The writer half of a conversion: the model as the derivative's file.

use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use super::Control;
use super::md_assets::publish_md;
use crate::documents::Format;
use crate::integrity::publish_new_file;
use crate::model::DocumentModel;
use crate::writers::{write_docx, write_pdf, write_txt};
use crate::{Error, Result};

/// Why a derivative was not published.
pub enum WriteFailure {
    /// The writer's reason, for the log.
    Writer(String),
    /// Publishing failed (disk, collision handling...).
    Publish(Error),
    /// The user cancelled before the derivative was published.
    Cancelled,
}

impl WriteFailure {
    /// The error of converting the document `id`.
    pub fn into_error(self, id: Uuid) -> Error {
        match self {
            Self::Writer(reason) => Error::write(id, &reason),
            Self::Publish(error) => error,
            Self::Cancelled => Error::ConversionCancelled(id),
        }
    }
}

/// Writes `model` in `target` through `tmp_dir`, then publishes it as
/// `dest_dir/name` (with suffix on collision).
pub fn publish_derivative(
    tmp_dir: &Path,
    dest_dir: &Path,
    name: &OsStr,
    model: &DocumentModel,
    target: Format,
    control: &Control,
) -> std::result::Result<PathBuf, WriteFailure> {
    let bytes = match target {
        // A Markdown with images also needs its `.assets` folder.
        Format::Md => {
            return publish_md(tmp_dir, dest_dir, name, model, control);
        }
        Format::Txt => write_txt(model).into_bytes(),
        Format::Docx => write_docx(model).map_err(WriteFailure::Writer)?,
        Format::Pdf => write_pdf(model).map_err(WriteFailure::Writer)?,
    };
    if control.is_cancelled() {
        return Err(WriteFailure::Cancelled);
    }
    publish(tmp_dir, dest_dir, name, &bytes).map_err(WriteFailure::Publish)
}

fn publish(tmp_dir: &Path, dest_dir: &Path, name: &OsStr, bytes: &[u8]) -> Result<PathBuf> {
    publish_new_file(tmp_dir, dest_dir, name, |file| file.write_all(bytes))
}
