//! Readers: turn a file of one format into the document model (spec 07,
//! "Leitores"). They work on bytes and never touch the disk.

mod decode;
mod docx;
mod error;
mod image;
mod md;
mod txt;

pub use decode::{decode_text, decode_text_start};
#[cfg(test)]
pub(crate) use docx::test_docx;
pub use docx::{package_bytes as docx_package_bytes, read_docx, read_docx_metadata};
pub use error::ReadError;
pub use image::image_mime;
pub(crate) use md::options as markdown_options;
pub use md::read_md;
pub use txt::read_txt;
