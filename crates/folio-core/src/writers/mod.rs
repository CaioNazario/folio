//! Writers: turn the document model into a file of one format (spec 08,
//! "Escritores"). They return bytes or text and never touch the disk.

mod docx;
mod md;
mod txt;

pub use docx::write_docx;
pub use md::{MdDocument, MdImage, write_md};
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
