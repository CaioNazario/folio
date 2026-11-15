//! Readers: turn a file of one format into the document model (spec 07,
//! "Leitores"). They work on bytes and never touch the disk.

mod decode;
mod md;
mod txt;

pub use decode::{decode_text, decode_text_start};
pub use md::read_md;
pub use txt::read_txt;
