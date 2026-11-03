//! The installer's tables of names (`g_dirNames`, 0x10100 bytes, and `g_fileNames`,
//! 0x40100 bytes, blocks of the game's heap on the installer's list): Rust's since phase
//! 12 (2026-10-27). Each held the names one after another, each ended by its NUL; here a
//! list of names, and the bytes they would take there (`bytes`), which the original's room
//! checks read as the cursor less the table. `INSTALLER_RUN.dir_scan` holds the number of
//! the next folder to search, `dir_cursor` / `file_cursor` the tables' bytes.

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

/// The folders' names (`g_dirNames`, 0x10100 bytes on the installer's list).
pub static FOLDERS: Owned<NameTable> = Owned::new("no installer folders table");
/// The files' names (`g_fileNames`, 0x40100 bytes on the installer's list).
pub static FILES: Owned<NameTable> = Owned::new("no installer files table");

/// The text at `at` (the game's memory) up to its NUL.
pub fn text_at(at: crate::pointers::GamePtr) -> Result<Vec<u8>, BadInput> {
    crate::files::cstring(at, "an installer name")
}

/// Name `index` of `table` written at `cursor` (the game's memory) without a NUL, as
/// `strings::str_append_indexed` appends it; the cursor past it. Refused past the table.
pub fn append_name(
    table: &Owned<NameTable>,
    index: u32,
    cursor: impl Into<crate::place::Place>,
) -> Result<crate::place::Place, Refused> {
    let cursor = cursor.into();
    let name = table
        .with(|t| t.names.get(index as usize).cloned())?
        .ok_or(BadInput("an installer name past its table"))?;
    cursor.write(&name, "an installer path")?;
    Ok(cursor.add(name.len() as u32))
}

/// A name made by the game's text helpers (which write at an address) in a lent buffer:
/// `make` writes it from the cursor it is given and gives the cursor past its NUL.
pub fn made_name(
    make: impl FnOnce(crate::pointers::GamePtr) -> Result<crate::pointers::GamePtr, Refused>,
) -> Result<Vec<u8>, Refused> {
    let lent = crate::pointers::LentBytes::from_vec(vec![0u8; 0x400]);
    let end = make(lent.ptr())?;
    let len = end.addr().wrapping_sub(lent.addr()) as usize;
    let buffer = lent.to_vec();
    match len.checked_sub(1) {
        Some(n) if len <= buffer.len() => Ok(buffer[..n].to_vec()),
        _ => Err(BadInput("an installer name past its buffer").into()),
    }
}
