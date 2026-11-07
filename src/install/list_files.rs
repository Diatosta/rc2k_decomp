//! `install::list_files` (0x4138B2): the files matching a pattern added to the files-to-copy
//! table. The pattern (eax) is resolved (`files::resolve_path`) into edx and searched
//! (`files::find_first` / `files::find_next_wanted`); each found name is appended at
//! `dword_51BB58` (the cursor) behind its folder (`files::append_directory` of edx) with a
//! NUL, the cursor advanced and `dword_51BB4C` (the count) raised. More than 0x40000 bytes
//! of names is fatal error 0x80AF2206. The search is closed at the end.
//!
//! In:  eax the pattern
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_script` (opcode 3).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{
    append_byte, close_search, fatal, find_first, find_next, resolve_path, str_append,
};
use crate::install::name_tables::{FILES, made_name};
use rc2k_core::install::script;

/// FindFileData.cFileName (the search record: `files::search`).
fn find_name() -> crate::pointers::GamePtr {
    crate::files::search::found_name_place()
}

/// The files of `pattern` (eax) added to the files' table.
pub fn list_files(
    g: &'static crate::install::owner::Install,
    pattern: impl Into<crate::place::Place>,
) -> Result<(), Refused> {
    let pattern = resolve_path(pattern)?;
    if find_first(&pattern) {
        loop {
            // append_directory (edx the pattern's path, ebx the cursor)
            let name = made_name(|at| {
                let (cursor, _) = crate::files::append_directory::append_directory(&pattern, at)?;
                Ok(append_byte(str_append(find_name(), cursor), 0).game_ptr())
            })?;
            let used = FILES.with(|t| t.push(name))?;
            g.run.file_cursor.set(used);
            g.run.file_count.set(g.run.file_count.get().wrapping_add(1));
            // Converted: `rc2k_core::install::script::room_left` (`sub ebx, [table]`).
            script::room_left(used, script::FILES_ROOM, script::FILES_FULL).map_err(fatal)?;
            if !find_next() {
                break;
            }
        }
    }
    close_search();
    Ok(())
}
