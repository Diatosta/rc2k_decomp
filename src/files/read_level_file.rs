//! `files::read_level_file` (0x41BA2E): a file read as it is, or, when it is not there,
//! its packed copy under `files\levels`.
//!
//! In:  eax = path (`srclvls\...`), ebx = buffer, ecx = its bytes
//! Out: with the file there, as `files::read_file` leaves them (edx kept); else as
//!      `files::read_packed_file` leaves them, on the path `files\levels\...` + ".sq"
//!      (built in `g_packedPath`, 0x5BE9BD: the path copied to +5, ".sq" after it, and
//!      "files\leve" over its first ten bytes, so that `srclvls\` becomes `files\levels\`)
//!
//! Refuses (since the conversion, 2026-10-21) a path too short for the packed copy's
//! prefix (the original's text then runs on past it). One caller, `sub_4455FC` (loadlevel, the
//! course data).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::files::read_packed_file::{Outcome, read, read_packed_file};
use crate::pointers::GamePtr;

/// `files::read_level_file` (see the header): the file at `path` (or its packed copy) into
/// `buffer` of `room` bytes.
pub fn read_level_file(path: GamePtr, buffer: GamePtr, room: u32) -> Result<Outcome, Refused> {
    if crate::files::file_exists::file_exists_at(path)? {
        return Ok(Outcome::of_read(path, &read(path, buffer, room)?));
    }
    let given = crate::files::cstring(path, "a level file's path")?;
    let packed = rc2k_core::files::io::packed_level_path(&given)?;
    let at = GamePtr::to(&crate::files::state::PACKED_PATH)
        .limited(crate::files::state::PACKED_PATH_LEN as u32);
    at.write_bytes(&packed, "the packed path")?;
    read_packed_file(at, buffer, room)
}
