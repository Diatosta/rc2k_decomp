//! `gfx::rgb_pixels_to_dwords` (0x41CC96): turns the picture buffer's decoded pixels of three
//! bytes into dwords, in place.
//!
//! `g_pictureWidth` * `g_pictureHeight` pixels, from the last down: the first byte shifted
//! up 16, the second up 8, the third; the top byte 0. One caller, `gfx::read_png`, for a
//! file without a palette: every PNG file of the game.
//! Refuses what `pixels_to_dwords::pixel_count` refuses. The name is a reading.

use crate::errors::BadInput;
use crate::gfx::picture::{read_bytes, write_bytes};
use crate::gfx::pixels_to_dwords::pixel_count;

// ---- Rust version ----

pub fn rgb_pixels_to_dwords(gfx: &crate::gfx::owner::Gfx) -> Result<(), BadInput> {
    let pictures = &gfx.pictures;
    // Converted: `rc2k_core::gfx::pixels::rgb_to_dwords`, on a copy of the pixels' bytes.
    let count = pixel_count(gfx)?;
    let mut buffer = read_bytes(pictures.picture.buffer.get(), 0, count * 4)?;
    rc2k_core::gfx::pixels::rgb_to_dwords(&mut buffer, count)
        .ok_or(BadInput("a picture buffer shorter than its dwords"))?;
    write_bytes(pictures.picture.buffer.get(), 0, &buffer)
}
