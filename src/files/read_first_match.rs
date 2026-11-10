//! `files::read_first_match` (0x402F82): reads from the first file that matches a path with
//! wildcards.
//!
//! Takes the path with wildcards, buffer, count, offset in the file. The path is resolved and
//! `FindFirstFileA` finds a file, going on with `FindNextFileA` past anything with the attributes
//! of `dword_4E06E6` (0x10: directories). The name found is put after the directory part of the
//! path as it was given (not as resolved) in the scratch block at 0x93E8B0, `read_file` reads that,
//! and the search is closed. Like the original it works in the game's own search record
//! (`FindFileData`, 0x4E05A8) and handle (`hFindFile`, 0x4E05A4). Three call sites: the language's
//! loading and the lexicon compiler.

use crate::errors::BadInput;
use crate::files::read_file::{ReadResult, read_file_into};
use crate::files::resolve_path::resolve;
use crate::files::state::{SKIPPED_ATTRIBUTES, set_last_file_path};
use crate::system::owner::system;

// ---- game data ----

/// None if nothing matched, else what the reading of the file found came to: `pattern`
/// (without its NUL) read into `into` (what the file does not reach over kept, as the
/// game's buffer). Refuses what `resolve_path` and `read_file` refuse.
pub fn first_match_read(
    pattern: &[u8],
    into: &mut [u8],
    offset: u32,
) -> Result<Option<ReadResult>, BadInput> {
    let system = system();
    let resolved = resolve(pattern)?;
    set_last_file_path(Some(&resolved));
    let found = crate::files::search::first(&resolved, SKIPPED_ATTRIBUTES.get());
    let read = if found {
        // Converted: `rc2k_core::files::io::match_path`.
        let name = crate::files::search::found_name();
        let joined = rc2k_core::files::io::match_path(pattern, &name);
        system
            .work
            .files
            .buffer
            .write_at(0, &joined, "the scratch buffer")?;
        Some(read_file_into(
            &joined[..joined.iter().position(|&b| b == 0).unwrap_or(joined.len())],
            into,
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
    pattern: &[u8],
    into: &mut [u8],
    offset: u32,
) -> Result<Option<u32>, BadInput> {
    let read = first_match_read(pattern, into, offset)?;
    Ok(read.filter(|read| read.ok).and_then(|read| read.bytes_read))
}
