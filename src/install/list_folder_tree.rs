//! `install::list_folder_tree` (0x41371C): a folder and everything under it added to the
//! tables, breadth first. The skip mask (`FindFileData+0x13E`) is set to 0 for the walk
//! (and put back at the end). The folder (eax) is resolved; if not empty it goes into the
//! folders table (`dword_51BB54`). Then, starting with that folder and working through every
//! folder recorded from `dword_51BB50` to `dword_51BB54`, each is searched (`<folder>\*.*`
//! built in `byte_94E8B0`): a sub-folder (not `.`/`..`) is appended to the folders table
//! behind its parent, a file to the files table (`dword_51BB58`); the folders table is the
//! queue. Overflow of either table is fatal error 0x80AF2201 or 0x80AF2202.
//!
//! In:  eax the folder
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_script` (opcode 4). Its resolve and search through `part`; the
//! string work here. Shadow mode runs the original alone. Self-test: a copy whose disk
//! calls go to the fake file system, against the Rust version, on a small tree.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{
    CLOSE_SEARCH, DIR_COUNT, DIR_CURSOR, DIR_SCAN, DIRS, FILE_COUNT, FILE_CURSOR, FILES,
    FIND_FIRST, FIND_NEXT, Part, RESOLVE_PATH, a, append_byte, at, fatal, g, p, rb, str_append, wb,
};

const SEARCH_BASE: usize = 0x94E8B0; // byte_94E8B0
const DOT: usize = 0x5295B7; // "."
const DOTDOT: usize = 0x5295B9; // ".."
const STAR: usize = 0x5295BC; // "*.*"
const SKIP_MASK: usize = 0x4E06E6; // FindFileData+0x13E
const FIND_NAME: usize = 0x4E05A8 + 0x2C; // FindFileData.cFileName
const FIND_ATTRS: usize = 0x4E05A8; // FindFileData.dwFileAttributes
const DIRS_ROOM: u32 = 0x10000;
const FILES_ROOM: u32 = 0x40000;

fn same_name(a1: u32, a2: u32) -> bool {
    unsafe { crate::strings::same_text::same_text(a1 as *const u8, a2 as *const u8) }
}

pub fn list_folder_tree(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let saved_skip = g(SKIP_MASK);
    p(SKIP_MASK, 0);
    let result = (|| -> Result<(), Refused> {
        part(at(RESOLVE_PATH), &mut r)?;
        if rb(r.eax) != 0 {
            r.ebx = g(DIR_CURSOR);
            str_append(&mut r);
            append_byte(&mut r, 0);
            p(DIR_CURSOR, r.ebx);
            p(DIR_SCAN, r.ebx);
            p(DIR_COUNT, g(DIR_COUNT).wrapping_add(1));
        }
        loop {
            // list the folder named in eax
            let base = a(SEARCH_BASE);
            r.ebx = base;
            r.edx = base;
            str_append(&mut r); // the folder path
            if rb(r.ebx.wrapping_sub(1)) != b'\\' {
                wb(r.ebx, b'\\');
                r.ebx += 1;
            }
            r.eax = a(STAR);
            str_append(&mut r);
            append_byte(&mut r, 0);
            r.eax = base;
            part(at(FIND_FIRST), &mut r)?;
            if !r.carry() {
                loop {
                    let is_dir = g(FIND_ATTRS) & 0x10 != 0;
                    if is_dir {
                        // eax "." then "..", ebx the found name (`same_text` keeps them)
                        r.ebx = a(FIND_NAME);
                        r.eax = a(DOT);
                        let mut real = !same_name(r.eax, r.ebx);
                        if real {
                            r.eax = a(DOTDOT);
                            real = !same_name(r.eax, r.ebx);
                        }
                        if real {
                            r.ebx = g(DIR_CURSOR);
                            r = unsafe {
                                crate::files::append_directory::append_directory_regs(&r)
                            }?;
                            r.eax = a(FIND_NAME);
                            str_append(&mut r);
                            append_byte(&mut r, 0);
                            p(DIR_CURSOR, r.ebx);
                            p(DIR_COUNT, g(DIR_COUNT).wrapping_add(1));
                            r.ebx = r.ebx.wrapping_sub(g(DIRS)); // `sub ebx, g_dirNames`
                            if r.ebx >= DIRS_ROOM {
                                return Err(fatal(0x80AF_2201));
                            }
                        }
                    } else {
                        r.ebx = g(FILE_CURSOR);
                        r = unsafe { crate::files::append_directory::append_directory_regs(&r) }?;
                        r.eax = a(FIND_NAME);
                        str_append(&mut r);
                        append_byte(&mut r, 0);
                        p(FILE_CURSOR, r.ebx);
                        p(FILE_COUNT, g(FILE_COUNT).wrapping_add(1));
                        r.ebx = r.ebx.wrapping_sub(g(FILES)); // `sub ebx, g_fileNames`
                        if r.ebx >= FILES_ROOM {
                            return Err(fatal(0x80AF_2202));
                        }
                    }
                    part(at(FIND_NEXT), &mut r)?;
                    if r.carry() {
                        break;
                    }
                }
            }
            part(at(CLOSE_SEARCH), &mut r)?;
            // dequeue the next folder from the scan pointer
            r.eax = g(DIR_SCAN);
            if r.eax >= g(DIR_CURSOR) {
                break;
            }
            r.ebx = r.eax;
            r.ecx = unsafe { crate::strings::text_length::text_length(r.ebx as *const u8) }?;
            if r.ecx == 0 {
                break;
            }
            r.ebx = r.ebx.wrapping_add(r.ecx).wrapping_add(1);
            p(DIR_SCAN, r.ebx);
        }
        Ok(())
    })();
    p(SKIP_MASK, saved_skip);
    result?;
    Ok(Regs { ..*entry })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        list_folder_tree(r, &mut crate::install::installer::game_part)
    })
}
