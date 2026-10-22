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
use crate::files::read_packed_file::{Outcome, Read, read_packed_file};
use crate::framework::{Regs, game_addr, rva};

pub const PACKED_PATH_RVA: usize = rva(0x5BE9BD); // g_packedPath

/// Whether a file can be opened (`files::file_exists`).
pub type Exists<'a> = &'a mut dyn FnMut(u32) -> Result<bool, Refused>;

pub fn game_exists(path: u32) -> Result<bool, Refused> {
    Ok(unsafe { crate::files::file_exists::file_exists(path as *const u8) }?)
}

/// `files::read_level_file` (see the header): eax, carry and zero.
pub unsafe fn read_level_file(
    entry: &Regs,
    exists: Exists,
    read: Read,
) -> Result<Outcome, Refused> {
    if exists(entry.eax)? {
        let mut r = Regs { edx: 0, ..*entry };
        read(&mut r)?;
        return Ok(Outcome::of(&r));
    }
    unsafe {
        // Converted: `rc2k_core::files::io::packed_level_path`.
        let given = std::ffi::CStr::from_ptr(entry.eax as *const std::ffi::c_char).to_bytes();
        let packed = rc2k_core::files::io::packed_level_path(given)?;
        let path = game_addr(PACKED_PATH_RVA) as *mut u8;
        std::ptr::copy(packed.as_ptr(), path, packed.len());
        read_packed_file(
            &Regs {
                eax: path as u32,
                ..*entry
            },
            read,
        )
    }
}
