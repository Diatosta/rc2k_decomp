//! Turns the picture buffer's decoded palette numbers, a byte a pixel, into the palette's dwords,
//! in place: picture width * picture height pixels, from the last down, each through the decoded
//! palette. Refuses what `pixels_to_dwords::pixel_count` refuses.

use crate::errors::BadInput;
use crate::gfx::picture::{read_bytes, write_bytes};
use crate::gfx::pixels_to_dwords::pixel_count;

pub fn palette_pixels_to_dwords(gfx: &crate::gfx::owner::Gfx) -> Result<(), BadInput> {
    let pictures = &gfx.pictures;
    let count = pixel_count(gfx)?;
    let mut buffer = read_bytes(pictures.picture.buffer.get(), 0, count * 4)?;
    let palette: [u32; 256] = pictures.decoder.palette();
    rc2k_core::gfx::pixels::palette_to_dwords(&mut buffer, count, &palette)
        .ok_or(BadInput("a picture buffer shorter than its dwords"))?;
    write_bytes(pictures.picture.buffer.get(), 0, &buffer)
}
