//! `install::list_folder` (0x413876): one folder path added to the folders-to-make table.
//! The path (eax) is resolved (`files::resolve_path`), appended with a NUL at
//! `dword_51BB54` (the cursor), the cursor and scan pointer advanced, and `dword_51BB48`
//! (the count) raised. More than 0x10000 bytes of names is fatal error 0x80AF2205.
//!
//! In:  eax the folder path
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_script` (opcode 1). Its resolve through `part`; the appending here.
//! Shadow mode runs the original alone. Self-test: a copy whose resolve goes to a stand-in,
//! against the Rust version, on paths of the test's; the table, cursors and count compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{
    DIR_COUNT, DIR_CURSOR, DIR_SCAN, DIRS, append_byte, fatal, g, p, resolve_path, str_append,
};
use rc2k_core::install::script;

/// The folder `path` (eax) added to the folders' table.
pub fn list_folder(path: u32) -> Result<(), Refused> {
    let path = resolve_path(path)?;
    let cursor = append_byte(str_append(path, g(DIR_CURSOR)), 0);
    p(DIR_CURSOR, cursor);
    p(DIR_SCAN, cursor);
    p(DIR_COUNT, g(DIR_COUNT).wrapping_add(1));
    // Converted: `rc2k_core::install::script::room_left`.
    script::room_left(
        cursor.wrapping_sub(g(DIRS)),
        script::FOLDERS_ROOM,
        script::FOLDERS_FULL,
    )
    .map_err(fatal)?;
    Ok(())
}
