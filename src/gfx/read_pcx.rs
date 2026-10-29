//! `gfx::read_pcx` (0x411A45): decodes a PCX file into the picture buffer.
//!
//! In:  eax = path
//! Out: carry = failed; every register preserved
//! Zeroes the header (`dword_51B5F4`, 0x80 bytes) and reads the file's first 0x80 bytes
//! into it with `files::read_file`; then by the header's number of planes: 1 goes to
//! `read_pcx_8bit`, 3 to `read_pcx_24bit` (each opens the file again), anything else is a
//! failure. Three callers, all in the picture loader `sub_4119F4`.
//! Refuses what `read_file` and the readers refuse. The name is a reading.

use crate::errors::BadInput;
use crate::files::read_file::read_file;
use crate::gfx::pcx::HEADER_LEN;
use crate::gfx::pcx::PCX;
use crate::gfx::read_pcx_8bit::read_pcx_8bit;
use crate::gfx::read_pcx_24bit::read_pcx_24bit;
use crate::system::zero_memory::zero_memory;

// ---- Rust version ----

/// Returns whether the picture was decoded.
pub unsafe fn read_pcx(path: *const u8) -> Result<bool, BadInput> {
    unsafe {
        let header = PCX.header.as_ptr();
        zero_memory(header, HEADER_LEN as u32)?;
        if !read_file(crate::files::game_path(path), header, HEADER_LEN as u32, 0)?.ok {
            return Ok(false);
        }
        // Converted: `rc2k_core::gfx::pcx::kind`.
        use rc2k_core::gfx::pcx::{Kind, kind};
        match kind(&PCX.header.read()) {
            Some(Kind::Palette) => read_pcx_8bit(path),
            Some(Kind::Planes) => read_pcx_24bit(path),
            None => Ok(false),
        }
    }
}
