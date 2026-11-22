//! The reader half of a conversion: the original's bytes as the model.

use crate::documents::Format;
use crate::model::DocumentModel;
use crate::readers::{ReadError, read_docx, read_md, read_pdf, read_txt};

/// The model of a file in `format`; `password` opens a protected one.
pub fn read_model(
    format: Format,
    bytes: &[u8],
    password: Option<&str>,
) -> Result<DocumentModel, ReadError> {
    match format {
        Format::Txt => Ok(read_txt(bytes)),
        Format::Md => Ok(read_md(bytes)),
        Format::Docx => read_docx(bytes, password),
        Format::Pdf => read_pdf(bytes, password),
    }
}
