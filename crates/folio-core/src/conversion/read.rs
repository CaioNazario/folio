//! The reader half of a conversion: the original's bytes as the model.

use super::{Control, Stage};
use crate::documents::Format;
use crate::model::DocumentModel;
use crate::readers::{ImageLookup, ReadError, read_docx, read_md_with, read_pdf_with, read_txt};

/// The model of a file in `format`; `password` opens a protected one. A PDF
/// reports the pages read to `control` and stops between pages when it is
/// cancelled. A Markdown's images are resolved by `images`.
pub fn read_model(
    format: Format,
    bytes: &[u8],
    password: Option<&str>,
    control: &Control,
    images: &mut dyn FnMut(&str) -> ImageLookup,
) -> Result<DocumentModel, ReadError> {
    match format {
        Format::Txt => Ok(read_txt(bytes)),
        Format::Md => Ok(read_md_with(bytes, images)),
        Format::Docx => read_docx(bytes, password),
        Format::Pdf => read_pdf_with(bytes, password, &mut |read, total| {
            control.report(Stage::Reading, Some(read as f64 / total as f64));
            !control.is_cancelled()
        }),
    }
}
