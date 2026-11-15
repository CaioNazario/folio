//! Readers: turn a file of one format into the document model (spec 07,
//! "Leitores"). They work on bytes and never touch the disk.

mod decode;
mod txt;

pub use decode::{decode_text, decode_text_start};
pub use txt::read_txt;
