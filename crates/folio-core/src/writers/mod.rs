//! Writers: turn the document model into a file of one format (spec 08,
//! "Escritores"). They return bytes or text and never touch the disk.

mod txt;

pub use txt::write_txt;
