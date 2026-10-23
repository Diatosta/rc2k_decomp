//! `gfx::read_pcx` (0x411A45): decodes a PCX file into the picture buffer.
//!
//! In:  eax = path
//! Out: carry = failed; every register preserved
//! Zeroes the header (`dword_51B5F4`, 0x80 bytes) and reads the file's first 0x80 bytes
//! into it with `files::read_file`; then by the header's number of planes: 1 goes to
//! `read_pcx_8bit`, 3 to `read_pcx_24bit` (each opens the file again), anything else is a
//! failure. Three callers, all in the picture loader `sub_4119F4`.
//! Shadow mode runs the original alone: the two readers it calls are compared in their own
//! hooks. Refuses what `read_file` and the readers refuse. The name is a reading.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::files::read_file::read_file;
use crate::framework::{game_addr, game_read};
use crate::gfx::pcx::{HEADER_LEN, HEADER_RVA};
use crate::gfx::read_pcx_8bit::read_pcx_8bit;
use crate::gfx::read_pcx_24bit::read_pcx_24bit;
use crate::system::zero_memory::zero_memory;

// ---- Rust version ----

/// Returns whether the picture was decoded.
pub unsafe fn read_pcx(path: *const u8) -> Result<bool, BadInput> {
    unsafe {
        let header = game_addr(HEADER_RVA) as *mut u8;
        zero_memory(header, HEADER_LEN as u32)?;
        if !read_file(path, header, HEADER_LEN as u32, 0)?.ok {
            return Ok(false);
        }
        // Converted: `rc2k_core::gfx::pcx::kind`.
        use rc2k_core::gfx::pcx::{Kind, kind};
        match kind(&game_read(HEADER_RVA)) {
            Some(Kind::Palette) => read_pcx_8bit(path),
            Some(Kind::Planes) => read_pcx_24bit(path),
            None => Ok(false),
        }
    }
}
