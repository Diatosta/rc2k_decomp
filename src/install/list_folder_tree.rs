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
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{
    DIR_COUNT, DIR_CURSOR, DIR_SCAN, DIRS, FILE_COUNT, FILE_CURSOR, FILES, a, append_byte,
    close_search, fatal, find_first, find_next, g, p, rb, resolve_path, str_append,
};

const DOT: usize = 0x5295B7; // "."
const DOTDOT: usize = 0x5295B9; // ".."
const STAR: usize = 0x5295BC; // "*.*"
/// FindFileData.cFileName (the search record: `files::state`).
fn find_name() -> u32 {
    unsafe { (&raw const (*crate::files::state::find_data()).cFileName) as u32 }
}
/// FindFileData.dwFileAttributes.
fn find_attributes() -> u32 {
    unsafe { (*crate::files::state::find_data()).dwFileAttributes }
}
const DIRS_ROOM: u32 = 0x10000;
const FILES_ROOM: u32 = 0x40000;

fn same_name(a1: u32, a2: u32) -> bool {
    unsafe { crate::strings::same_text::same_text(a1 as *const u8, a2 as *const u8) }
}

/// The found file's path (the searched folder's) and name appended at `cursor`.
fn found_path(search: u32, cursor: u32) -> Result<u32, Refused> {
    let (cursor, _) = unsafe {
        crate::files::append_directory::append_directory(search as *const u8, cursor as *mut u8)
    }?;
    Ok(append_byte(str_append(find_name(), cursor as u32), 0))
}

fn walk(folder: u32) -> Result<(), Refused> {
    let mut folder = resolve_path(folder)?;
    if rb(folder) != 0 {
        let cursor = append_byte(str_append(folder, g(DIR_CURSOR)), 0);
        p(DIR_CURSOR, cursor);
        p(DIR_SCAN, cursor);
        p(DIR_COUNT, g(DIR_COUNT).wrapping_add(1));
    }
    loop {
        // list the folder
        let base = crate::files::numbered_path::buffer() as u32; // byte_94E8B0
        let mut cursor = str_append(folder, base);
        if rb(cursor.wrapping_sub(1)) != b'\\' {
            cursor = append_byte(cursor, b'\\');
        }
        append_byte(str_append(a(STAR), cursor), 0);
        if find_first(base) {
            loop {
                if find_attributes() & 0x10 != 0 {
                    // "." and ".." passed over
                    let real =
                        !same_name(a(DOT), find_name()) && !same_name(a(DOTDOT), find_name());
                    if real {
                        let cursor = found_path(base, g(DIR_CURSOR))?;
                        p(DIR_CURSOR, cursor);
                        p(DIR_COUNT, g(DIR_COUNT).wrapping_add(1));
                        // `sub ebx, g_dirNames`
                        if cursor.wrapping_sub(g(DIRS)) >= DIRS_ROOM {
                            return Err(fatal(0x80AF_2201));
                        }
                    }
                } else {
                    let cursor = found_path(base, g(FILE_CURSOR))?;
                    p(FILE_CURSOR, cursor);
                    p(FILE_COUNT, g(FILE_COUNT).wrapping_add(1));
                    // `sub ebx, g_fileNames`
                    if cursor.wrapping_sub(g(FILES)) >= FILES_ROOM {
                        return Err(fatal(0x80AF_2202));
                    }
                }
                if !find_next() {
                    break;
                }
            }
        }
        close_search();
        // the next folder from the scan pointer
        folder = g(DIR_SCAN);
        if folder >= g(DIR_CURSOR) {
            break;
        }
        let length = unsafe { crate::strings::text_length::text_length(folder as *const u8) }?;
        if length == 0 {
            break;
        }
        p(DIR_SCAN, folder.wrapping_add(length).wrapping_add(1));
    }
    Ok(())
}

/// `folder` (eax) and every folder under it added to the folders' table, their files to
/// the files'.
pub fn list_folder_tree(folder: u32) -> Result<(), Refused> {
    use crate::files::state::SKIPPED_ATTRIBUTES;
    let saved_skip = SKIPPED_ATTRIBUTES.load(std::sync::atomic::Ordering::Relaxed);
    SKIPPED_ATTRIBUTES.store(0, std::sync::atomic::Ordering::Relaxed);
    let result = walk(folder);
    SKIPPED_ATTRIBUTES.store(saved_skip, std::sync::atomic::Ordering::Relaxed);
    result
}
