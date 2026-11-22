//! Writers: turn the document model into a file of one format (spec 08,
//! "Escritores"). They return bytes or text and never touch the disk.

mod md;
mod txt;

pub use md::{MdDocument, MdImage, write_md};
pub use txt::write_txt;
