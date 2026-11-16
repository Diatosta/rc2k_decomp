//! Reads from the first file that matches a path with wildcards: the path is resolved and the
//! search passes over anything with the skipped attributes (directories); the name found is put
//! after the directory part of the path as it was given (not as resolved), read with
//! `read_file`, and the search is closed.

use crate::errors::BadInput;
use crate::files::read_file::{ReadResult, read_file_into};
use crate::files::resolve_path::resolve;
use crate::files::state::set_last_file_path;
use crate::system::owner::System;

/// None if nothing matched, else what the reading of the file found came to: `pattern`
/// (without its NUL) read into `into` (what the file does not reach over kept, as the
/// game's buffer). Refuses what `resolve_path` and `read_file` refuse.
pub fn first_match_read(
    system: &System,
    pattern: &[u8],
    into: &mut [u8],
    offset: u32,
) -> Result<Option<ReadResult>, BadInput> {
    let resolved = resolve(pattern)?;
    set_last_file_path(Some(&resolved));
    let found = crate::files::search::first(&resolved, crate::files::search::DIRECTORY);
    let read = if found {
        let name = crate::files::search::found_name();
        let joined = rc2k_core::files::io::match_path(pattern, &name);
        system
            .work()
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

/// The bytes read, or None if nothing matched or the reading failed.
pub fn read_first_match(
    system: &System,
    pattern: &[u8],
    into: &mut [u8],
    offset: u32,
) -> Result<Option<u32>, BadInput> {
    let read = first_match_read(system, pattern, into, offset)?;
    Ok(read.filter(|read| read.ok).and_then(|read| read.bytes_read))
}
