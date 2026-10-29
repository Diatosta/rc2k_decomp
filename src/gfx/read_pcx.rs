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
use crate::gfx::pcx::{PCX, read_header};
use crate::gfx::read_pcx_8bit::read_pcx_8bit;
use crate::gfx::read_pcx_24bit::read_pcx_24bit;

// ---- Rust version ----

/// Returns whether the picture was decoded.
pub fn read_pcx(path: &[u8]) -> Result<bool, BadInput> {
    if !read_header(path)? {
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
