//! `files::read_level_file` (0x41BA2E): a file read as it is, or, when it is not there,
//! its packed copy under `files\levels`.
//!
//! Takes path (`srclvls\...`), buffer, its bytes.
//!
//! Refuses a path too short for the packed copy's prefix (the
//! original's text then runs on past it). One caller, `race::load_level` (loadlevel, the course
//! data).

use crate::errors::Refused;
use crate::files::read_packed_file::{Outcome, read, read_packed_file};
use crate::pointers::GamePtr;

/// `files::read_level_file` (see the header): the file at `path` (or its packed copy) into
/// `buffer` of `room` bytes.
/// `path` without its NUL.
pub fn read_level_file(path: &[u8], buffer: GamePtr, room: u32) -> Result<Outcome, Refused> {
    if crate::files::file_exists::file_exists(path)? {
        return Ok(Outcome::of_read(&read(path, buffer, room)?));
    }
    let packed = rc2k_core::files::io::packed_level_path(path)?;
    crate::files::state::PACKED_PATH.write_at(0, &packed, "the packed path")?;
    // The packed path as built (without its NUL).
    let text = &packed[..packed.iter().position(|&b| b == 0).unwrap_or(packed.len())];
    read_packed_file(text, buffer, room)
}
