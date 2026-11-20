//! A file read as it is, or, when it is not there, its packed copy under `files\levels`. Refuses a
//! path too short for the packed copy's prefix.

use crate::errors::Refused;
use crate::files::read_packed_file::{bytes_of_read, read, read_packed_file};

/// The file at `path` (without its NUL), or its packed copy, into `buffer` (its length is the
/// room): the bytes read, or `None` when it could not be read.
pub fn read_level_file(path: &[u8], buffer: &mut [u8]) -> Result<Option<u32>, Refused> {
    if crate::files::file_exists::file_exists(path)? {
        return Ok(bytes_of_read(&read(path, buffer)?));
    }
    let packed = rc2k_core::files::io::packed_level_path(path)?;
    // The packed path as built (without its NUL).
    let text = &packed[..packed.iter().position(|&b| b == 0).unwrap_or(packed.len())];
    read_packed_file(text, buffer)
}
