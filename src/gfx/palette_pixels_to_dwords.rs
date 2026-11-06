//! `gfx::palette_pixels_to_dwords` (0x41CCCB): turns the picture buffer's decoded palette
//! numbers, a byte a pixel, into the palette's dwords, in place.
//!
//! In:  nothing
//! Out: every register preserved
//! `g_pictureWidth` * `g_pictureHeight` pixels, from the last down, each through
//! `g_decodedPalette`. One caller, `gfx::read_png`, for a file with a palette (none of the
//! game's has one).
//! Refuses what `pixels_to_dwords::pixel_count` refuses. The name is a reading.

use crate::errors::BadInput;
use crate::gfx::owner::gfx;
use crate::gfx::picture::{read_bytes, write_bytes};
use crate::gfx::pixels_to_dwords::pixel_count;

// ---- Rust version ----

pub fn palette_pixels_to_dwords() -> Result<(), BadInput> {
    let pictures = &gfx().pictures;
    let count = pixel_count()?;
    // Converted: `rc2k_core::gfx::pixels::palette_to_dwords`, on a copy of the pixels' bytes.
    let mut buffer = read_bytes(pictures.picture.buffer.get(), 0, count * 4)?;
    let palette: [u32; 256] = pictures.decoder.palette();
    rc2k_core::gfx::pixels::palette_to_dwords(&mut buffer, count, &palette)
        .ok_or(BadInput("a picture buffer shorter than its dwords"))?;
    write_bytes(pictures.picture.buffer.get(), 0, &buffer)
}
