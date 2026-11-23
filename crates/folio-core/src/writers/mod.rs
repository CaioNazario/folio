//! Writers: turn the document model into a file of one format (spec 08,
//! "Escritores"). They return bytes or text and never touch the disk.

mod docx;
mod image_size;
mod md;
mod pdf;
mod txt;

pub use docx::write_docx;
pub use md::{MdDocument, MdImage, write_md};
#[cfg(test)]
pub(crate) use pdf::drawn::drawn as pdf_drawn;
pub use pdf::write_pdf;
pub use txt::write_txt;

/// The file name extension of an image of type `mime`, for the files a
/// writer names (`imagem-1.png`).
fn image_extension(mime: &str) -> &str {
    match mime {
        "image/jpeg" => "jpg",
        "image/svg+xml" => "svg",
        _ => mime.rsplit('/').next().unwrap_or(mime),
    }
}
