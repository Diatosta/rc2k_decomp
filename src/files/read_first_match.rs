//! `files::read_first_match` (0x402F82): reads from the first file that matches a path with
//! wildcards.
//!
//! In:  eax = the path with wildcards, ebx = buffer, ecx = count, edx = offset in the file
//! Out: nothing matched: eax = 0, carry set, zero clear; else as `files::read_file` leaves
//!      them: carry = failed (zero = ok), eax = bytes read if the read was reached, else
//!      the address of the path read from; everything else preserved
//! The path is resolved and `FindFirstFileA` finds a file, going on with `FindNextFileA`
//! past anything with the attributes of `dword_4E06E6` (0x10: directories). The name found
//! is put after the directory part of the path as it was given (not as resolved) in the
//! scratch block at 0x93E8B0, `read_file` reads that, and the search is closed. Like the
//! original it works in the game's own search record (`FindFileData`, 0x4E05A8) and handle
//! (`hFindFile`, 0x4E05A4).
//! Three call sites: the language's loading and the lexicon compiler.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::files::read_file::{ReadResult, read_file};
use crate::files::resolve_path::resolve_path_in_game;
use crate::files::scratch::SCRATCH;
use crate::files::state::{FIND_HANDLE, LAST_FILE_PATH, SKIPPED_ATTRIBUTES, find_data};
use std::sync::atomic::Ordering::Relaxed;
use windows_sys::Win32::Storage::FileSystem::{FindClose, FindFirstFileA, FindNextFileA};

// ---- game data ----

const NO_SEARCH: u32 = u32::MAX;

// ---- Rust version ----

/// None if nothing matched, else what the reading of the file found came to.
/// Refuses what `resolve_path` and `read_file` refuse, and a null path.
pub unsafe fn first_match_read(
    pattern: *const u8,
    buffer: *mut u8,
    count: u32,
    offset: u32,
) -> Result<Option<ReadResult>, BadInput> {
    unsafe {
        if pattern.is_null() {
            return Err(BadInput("no path to match"));
        }
        let resolved = resolve_path_in_game(pattern)?;
        LAST_FILE_PATH.store(resolved as u32, Relaxed);
        let record = find_data();
        let search = FindFirstFileA(resolved, record);
        FIND_HANDLE.store(search as u32, Relaxed);
        let skipped = || (*record).dwFileAttributes & SKIPPED_ATTRIBUTES.load(Relaxed) != 0;
        let mut found = search as u32 != NO_SEARCH;
        if found && skipped() {
            loop {
                if FindNextFileA(search, record) == 0 {
                    found = false;
                    break;
                }
                if !skipped() {
                    break;
                }
            }
        }
        let read = if found {
            // Converted: `rc2k_core::files::io::match_path`.
            let pattern = std::ffi::CStr::from_ptr(pattern.cast()).to_bytes();
            let name = std::ffi::CStr::from_ptr((&raw const (*record).cFileName).cast()).to_bytes();
            let joined = rc2k_core::files::io::match_path(pattern, name);
            let path = SCRATCH.buffer.addr() as *mut u8;
            std::ptr::copy(joined.as_ptr(), path, joined.len());
            Some(read_file(path, buffer, count, offset))
        } else {
            None
        };
        if FIND_HANDLE.load(Relaxed) != NO_SEARCH {
            FindClose(FIND_HANDLE.load(Relaxed) as _);
            FIND_HANDLE.store(NO_SEARCH, Relaxed);
        }
        read.transpose()
    }
}

/// For Rust callers: the bytes read, or None if nothing matched or the reading failed.
pub unsafe fn read_first_match(
    pattern: *const u8,
    buffer: *mut u8,
    count: u32,
    offset: u32,
) -> Result<Option<u32>, BadInput> {
    let read = unsafe { first_match_read(pattern, buffer, count, offset) }?;
    Ok(read.filter(|read| read.ok).and_then(|read| read.bytes_read))
}
