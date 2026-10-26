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
//! Caller `install::run_script` (opcode 4).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{
    append_byte, close_search, fatal, find_first, find_next, rb, resolve_path, str_append,
};
use crate::install::vars::INSTALLER_RUN;

/// FindFileData.cFileName (the search record: `files::search`).
fn find_name() -> u32 {
    crate::files::search::found_name_place()
}
/// FindFileData.dwFileAttributes.
fn find_attributes() -> u32 {
    crate::files::search::found_attributes()
}
const DIRS_ROOM: u32 = 0x10000;
const FILES_ROOM: u32 = 0x40000;

fn same_name(a1: u32, a2: u32) -> bool {
    crate::menu::event_flow::refuse_here(
        crate::strings::same_text::same_text(
            crate::pointers::host(a1) as *const u8,
            crate::pointers::host(a2) as *const u8,
        ),
        "install::list_folder_tree: a name",
    )
}

/// The found file's path (the searched folder's) and name appended at `cursor`.
fn found_path(search: u32, cursor: u32) -> Result<u32, Refused> {
    let (cursor, _) = crate::files::append_directory::append_directory(
        crate::pointers::host(search) as *const u8,
        crate::pointers::host(cursor),
    )?;
    Ok(append_byte(
        str_append(find_name(), crate::pointers::game_address(cursor)),
        0,
    ))
}

fn walk(folder: u32) -> Result<(), Refused> {
    let mut folder = resolve_path(folder)?;
    if rb(folder) != 0 {
        let cursor = append_byte(str_append(folder, INSTALLER_RUN.dir_cursor.get()), 0);
        INSTALLER_RUN.dir_cursor.set(cursor);
        INSTALLER_RUN.dir_scan.set(cursor);
        INSTALLER_RUN
            .dir_count
            .set(INSTALLER_RUN.dir_count.get().wrapping_add(1));
    }
    loop {
        // list the folder
        let base = crate::pointers::game_address(crate::files::numbered_path::buffer()); // byte_94E8B0
        let mut cursor = str_append(folder, base);
        if rb(cursor.wrapping_sub(1)) != b'\\' {
            cursor = append_byte(cursor, b'\\');
        }
        append_byte(
            str_append(crate::install::installer::INSTALL_TEXTS.star.addr(), cursor),
            0,
        );
        if find_first(base) {
            loop {
                if find_attributes() & 0x10 != 0 {
                    // "." and ".." passed over
                    let real = !same_name(
                        crate::install::installer::INSTALL_TEXTS.dot.addr(),
                        find_name(),
                    ) && !same_name(
                        crate::install::installer::INSTALL_TEXTS.dotdot.addr(),
                        find_name(),
                    );
                    if real {
                        let cursor = found_path(base, INSTALLER_RUN.dir_cursor.get())?;
                        INSTALLER_RUN.dir_cursor.set(cursor);
                        INSTALLER_RUN
                            .dir_count
                            .set(INSTALLER_RUN.dir_count.get().wrapping_add(1));
                        // `sub ebx, g_dirNames`
                        if cursor.wrapping_sub(INSTALLER_RUN.dirs.get()) >= DIRS_ROOM {
                            return Err(fatal(0x80AF_2201));
                        }
                    }
                } else {
                    let cursor = found_path(base, INSTALLER_RUN.file_cursor.get())?;
                    INSTALLER_RUN.file_cursor.set(cursor);
                    INSTALLER_RUN
                        .file_count
                        .set(INSTALLER_RUN.file_count.get().wrapping_add(1));
                    // `sub ebx, g_fileNames`
                    if cursor.wrapping_sub(INSTALLER_RUN.files.get()) >= FILES_ROOM {
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
        folder = INSTALLER_RUN.dir_scan.get();
        if folder >= INSTALLER_RUN.dir_cursor.get() {
            break;
        }
        let length =
            crate::strings::text_length::text_length(crate::pointers::host(folder) as *const u8)?;
        if length == 0 {
            break;
        }
        INSTALLER_RUN
            .dir_scan
            .set(folder.wrapping_add(length).wrapping_add(1));
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
