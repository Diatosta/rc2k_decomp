//! The installer's tables of names (folders, 0x10100 bytes; files, 0x40100 bytes): the names one
//! after another, each ended by a NUL, with the bytes they take (`bytes`) for the room checks.
//! `INSTALLER_RUN.dir_scan` holds the number of the next folder to search, `dir_cursor` /
//! `file_cursor` the tables' bytes.

use crate::errors::{BadInput, Refused};
use crate::memory::owned::Owned;

/// A table of names and the bytes they take with their NULs.
#[derive(Default)]
pub struct NameTable {
    pub names: Vec<Vec<u8>>,
    pub bytes: u32,
}

impl NameTable {
    /// `name` (without its NUL) added; the bytes taken after it.
    pub fn push(&mut self, name: Vec<u8>) -> u32 {
        self.bytes = self.bytes.wrapping_add(name.len() as u32 + 1);
        self.names.push(name);
        self.bytes
    }
}

/// Name `index` of `table` (without a NUL, as `strings::str_append_indexed` appends it)
/// added to `out`. Refused past the table.
pub fn push_name(table: &Owned<NameTable>, index: u32, out: &mut Vec<u8>) -> Result<(), Refused> {
    let name = table
        .with(|t| t.names.get(index as usize).cloned())?
        .ok_or(BadInput("an installer name past its table"))?;
    out.extend_from_slice(&name);
    Ok(())
}

/// A found file's name behind its search's folder: the directory of `path` (to its last
/// separator, if it has one; `files::append_directory`) and `name`. Refused: a result longer
/// than 0x400 bytes with its NUL.
pub fn found_name(path: &[u8], name: &[u8]) -> Result<Vec<u8>, Refused> {
    let mut made = rc2k_core::files::paths::directory(path)
        .unwrap_or_default()
        .to_vec();
    made.extend_from_slice(name);
    if made.len() >= 0x400 {
        return Err(BadInput("an installer name past its buffer").into());
    }
    Ok(made)
}
