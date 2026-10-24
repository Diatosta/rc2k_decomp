//! `install::list_files` (0x4138B2): the files matching a pattern added to the files-to-copy
//! table. The pattern (eax) is resolved (`files::resolve_path`) into edx and searched
//! (`files::find_first` / `files::find_next_wanted`); each found name is appended at
//! `dword_51BB58` (the cursor) behind its folder (`files::append_directory` of edx) with a
//! NUL, the cursor advanced and `dword_51BB4C` (the count) raised. More than 0x40000 bytes
//! of names is fatal error 0x80AF2206. The search is closed at the end.
//!
//! In:  eax the pattern
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_script` (opcode 3). Its resolve, search and `append_directory`
//! through `part`; the appending here. Shadow mode runs the original alone. Self-test: a
//! copy whose disk calls go to the fake file system, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{
    FILE_COUNT, FILE_CURSOR, FILES, append_byte, close_search, fatal, find_first, find_next, g, p,
    resolve_path, str_append,
};
use rc2k_core::install::script;

/// FindFileData.cFileName (the search record: `files::state`).
fn find_name() -> u32 {
    unsafe { (&raw const (*crate::files::state::find_data()).cFileName) as u32 }
}

/// The files of `pattern` (eax) added to the files' table.
pub fn list_files(pattern: u32) -> Result<(), Refused> {
    let pattern = resolve_path(pattern)?;
    if find_first(pattern) {
        loop {
            // append_directory (edx the pattern's path, ebx the cursor)
            let (cursor, _) = unsafe {
                crate::files::append_directory::append_directory(
                    pattern as *const u8,
                    g(FILE_CURSOR) as *mut u8,
                )
            }?;
            let cursor = append_byte(str_append(find_name(), cursor as u32), 0);
            p(FILE_CURSOR, cursor);
            p(FILE_COUNT, g(FILE_COUNT).wrapping_add(1));
            // Converted: `rc2k_core::install::script::room_left` (`sub ebx, [table]`).
            script::room_left(
                cursor.wrapping_sub(g(FILES)),
                script::FILES_ROOM,
                script::FILES_FULL,
            )
            .map_err(fatal)?;
            if !find_next() {
                break;
            }
        }
    }
    close_search();
    Ok(())
}
