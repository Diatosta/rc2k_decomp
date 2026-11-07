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
use crate::install::installer::{fatal, resolve_path};
use crate::install::name_tables::FOLDERS;
use rc2k_core::install::script;

/// The folder `path` (eax) added to the folders' table.
pub fn list_folder(
    g: &'static crate::install::owner::Install,
    path: impl Into<crate::place::Place>,
) -> Result<(), Refused> {
    let path = resolve_path(path)?;
    let (used, scan) = FOLDERS.with(|t| (t.push(path), t.names.len() as u32))?;
    g.run.dir_cursor.set(used);
    g.run.dir_scan.set(scan);
    g.run.dir_count.set(g.run.dir_count.get().wrapping_add(1));
    // Converted: `rc2k_core::install::script::room_left`.
    script::room_left(used, script::FOLDERS_ROOM, script::FOLDERS_FULL).map_err(fatal)?;
    Ok(())
}
