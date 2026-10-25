//! `install::list_folder` (0x413876): one folder path added to the folders-to-make table.
//! The path (eax) is resolved (`files::resolve_path`), appended with a NUL at
//! `dword_51BB54` (the cursor), the cursor and scan pointer advanced, and `dword_51BB48`
//! (the count) raised. More than 0x10000 bytes of names is fatal error 0x80AF2205.
//!
//! In:  eax the folder path
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_script` (opcode 1).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{append_byte, fatal, resolve_path, str_append};
use crate::install::vars::INSTALLER_RUN;
use rc2k_core::install::script;

/// The folder `path` (eax) added to the folders' table.
pub fn list_folder(path: u32) -> Result<(), Refused> {
    let path = resolve_path(path)?;
    let cursor = append_byte(str_append(path, INSTALLER_RUN.dir_cursor.get()), 0);
    INSTALLER_RUN.dir_cursor.set(cursor);
    INSTALLER_RUN.dir_scan.set(cursor);
    INSTALLER_RUN
        .dir_count
        .set(INSTALLER_RUN.dir_count.get().wrapping_add(1));
    // Converted: `rc2k_core::install::script::room_left`.
    script::room_left(
        cursor.wrapping_sub(INSTALLER_RUN.dirs.get()),
        script::FOLDERS_ROOM,
        script::FOLDERS_FULL,
    )
    .map_err(fatal)?;
    Ok(())
}
