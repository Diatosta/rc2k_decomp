//! `gfx::read_pcx` (0x411A45): decodes a PCX file into the picture buffer.
//!
//! Takes: path
//! Zeroes the header (`header`, 0x80 bytes) and reads the file's first 0x80 bytes
//! into it with `files::read_file`; then by the header's number of planes: 1 goes to
//! `read_pcx_8bit`, 3 to `read_pcx_24bit` (each opens the file again), anything else is a
//! failure. Three callers, all in the picture loader `gfx::read_picture_file`.
//! Refuses what `read_file` and the readers refuse. The name is a reading.

use crate::errors::BadInput;
use crate::gfx::pcx::read_header;
use crate::gfx::read_pcx_8bit::read_pcx_8bit;
use crate::gfx::read_pcx_24bit::read_pcx_24bit;

/// Returns whether the picture was decoded.
pub fn read_pcx(path: &[u8], gfx: &crate::gfx::owner::Gfx) -> Result<bool, BadInput> {
    if !read_header(path, gfx)? {
        return Ok(false);
    }
    // Converted: `rc2k_core::gfx::pcx::kind`.
    use rc2k_core::gfx::pcx::{Kind, kind};
    match kind(&gfx.pictures.pcx.header.read()) {
        Some(Kind::Palette) => read_pcx_8bit(path, gfx),
        Some(Kind::Planes) => read_pcx_24bit(path, gfx),
        None => Ok(false),
    }
}
