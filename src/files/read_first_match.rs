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
use crate::files::resolve_path::resolve;
use crate::files::scratch::SCRATCH;
use crate::files::state::{SKIPPED_ATTRIBUTES, set_last_file_path};
use crate::pointers::GamePtr;

// ---- game data ----

// ---- Rust version ----

/// None if nothing matched, else what the reading of the file found came to.
/// Refuses what `resolve_path` and `read_file` refuse, and a null path.
pub fn first_match_read(
    pattern: GamePtr,
    buffer: GamePtr,
    count: u32,
    offset: u32,
) -> Result<Option<ReadResult>, BadInput> {
    if pattern.is_null() {
        return Err(BadInput("no path to match"));
    }
    let pattern = crate::files::cstring(pattern, "a search pattern")?;
    let resolved = resolve(&pattern)?;
    set_last_file_path(Some(&resolved));
    let found = crate::files::search::first(&resolved, SKIPPED_ATTRIBUTES.get());
    let read = if found {
        // Converted: `rc2k_core::files::io::match_path`.
        let name = crate::files::search::found_name();
        let joined = rc2k_core::files::io::match_path(&pattern, &name);
        SCRATCH.buffer.write_at(0, &joined, "the scratch buffer")?;
        Some(read_file(
            GamePtr::to(&SCRATCH.buffer),
            buffer,
            count,
            offset,
        ))
    } else {
        None
    };
    crate::files::search::close();
    read.transpose()
}

/// For Rust callers: the bytes read, or None if nothing matched or the reading failed.
pub fn read_first_match(
    pattern: GamePtr,
    buffer: GamePtr,
    count: u32,
    offset: u32,
) -> Result<Option<u32>, BadInput> {
    let read = { first_match_read(pattern, buffer, count, offset) }?;
    Ok(read.filter(|read| read.ok).and_then(|read| read.bytes_read))
}
