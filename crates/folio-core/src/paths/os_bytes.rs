//! The one place that turns OS paths into stored bytes and back (spec 02).
//!
//! On Unix the bytes are the raw path bytes (`OsStrExt`), never
//! `as_encoded_bytes`, whose encoding the std does not guarantee.

use std::ffi::OsStr;

#[cfg(not(unix))]
compile_error!("path bytes outside Unix are specified with the Windows version (spec 02)");

/// The exact bytes of `os_str` as stored in the databases.
pub fn as_bytes(os_str: &OsStr) -> &[u8] {
    std::os::unix::ffi::OsStrExt::as_bytes(os_str)
}

/// The OS string for stored `bytes`.
pub fn from_bytes(bytes: &[u8]) -> &OsStr {
    std::os::unix::ffi::OsStrExt::from_bytes(bytes)
}
