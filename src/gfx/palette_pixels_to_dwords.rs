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
use crate::gfx::block::game_read;
use crate::gfx::pcx::PALETTE_RVA;
use crate::gfx::picture::picture;
use crate::gfx::pixels_to_dwords::pixel_count;

// ---- Rust version ----

pub unsafe fn palette_pixels_to_dwords() -> Result<(), BadInput> {
    unsafe {
        let count = pixel_count()?;
        // Converted: `rc2k_core::gfx::pixels::palette_to_dwords`.
        let buffer = std::slice::from_raw_parts_mut(picture(), count * 4);
        let palette: [u32; 256] = game_read(PALETTE_RVA);
        rc2k_core::gfx::pixels::palette_to_dwords(buffer, count, &palette)
            .ok_or(BadInput("a picture buffer shorter than its dwords"))
    }
}
