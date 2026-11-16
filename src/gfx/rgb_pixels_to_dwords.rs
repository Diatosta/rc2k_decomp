//! Turns the picture buffer's decoded pixels of three bytes into dwords, in place: width times
//! height pixels, from the last down: the first byte shifted up 16, the second up 8, the third; the
//! top byte 0. Refuses what `pixels_to_dwords::pixel_count` refuses.

use crate::errors::BadInput;
use crate::gfx::picture::{read_bytes, write_bytes};
use crate::gfx::pixels_to_dwords::pixel_count;

pub fn rgb_pixels_to_dwords(gfx: &crate::gfx::owner::Gfx) -> Result<(), BadInput> {
    let pictures = &gfx.pictures;
    let count = pixel_count(gfx)?;
    let mut buffer = read_bytes(
        &pictures.buffers,
        pictures.picture.buffer.get(),
        0,
        count * 4,
    )?;
    rc2k_core::gfx::pixels::rgb_to_dwords(&mut buffer, count)
        .ok_or(BadInput("a picture buffer shorter than its dwords"))?;
    write_bytes(&pictures.buffers, pictures.picture.buffer.get(), 0, &buffer)
}
