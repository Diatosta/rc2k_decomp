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

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{
    CLOSE_SEARCH, FILE_COUNT, FILE_CURSOR, FILES, FIND_FIRST, FIND_NEXT, RESOLVE_PATH, a,
    append_byte, at, fatal, g, p, str_append,
};
use rc2k_core::install::script;

const FIND_NAME: usize = 0x4E05A8 + 0x2C; // FindFileData.cFileName

pub fn list_files(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    crate::install::installer::game_part(at(RESOLVE_PATH), &mut r)?;
    r.edx = r.eax;
    crate::install::installer::game_part(at(FIND_FIRST), &mut r)?;
    if r.carry() {
        crate::install::installer::game_part(at(CLOSE_SEARCH), &mut r)?;
        return Ok(*entry);
    }
    loop {
        r.ebx = g(FILE_CURSOR);
        // append_directory(edx = the pattern's path, ebx = the cursor): pure, called direct
        r = unsafe { crate::files::append_directory::append_directory_regs(&r) }?;
        r.eax = a(FIND_NAME);
        str_append(&mut r);
        append_byte(&mut r, 0);
        p(FILE_CURSOR, r.ebx);
        p(FILE_COUNT, g(FILE_COUNT).wrapping_add(1));
        // `sub ebx, [table]`: find_next gets the bytes used in ebx.
        r.ebx = r.ebx.wrapping_sub(g(FILES));
        // Converted: `rc2k_core::install::script::room_left`.
        script::room_left(r.ebx, script::FILES_ROOM, script::FILES_FULL).map_err(fatal)?;
        crate::install::installer::game_part(at(FIND_NEXT), &mut r)?;
        if r.carry() {
            break;
        }
    }
    crate::install::installer::game_part(at(CLOSE_SEARCH), &mut r)?;
    Ok(Regs { ..*entry })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| list_files(r))
}
