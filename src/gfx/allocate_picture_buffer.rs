//! Takes the memory pictures are decoded in: `size` (800 x 600 dwords) and 0xC80 bytes more from
//! `memory::allocate` on the list `list` (Rust's memory: `gfx::picture`); the buffer's number goes
//! to the picture buffer variable. A failed allocation is fatal, but the allocator reports its own
//! error. `gfx::free_picture_buffer` gives the memory back at the game's end.

use crate::errors::Refused;
use crate::gfx::picture::{Buffer, take};

const BUFFER_LEN: u32 = 0x1D_4C00;
const SPARE: u32 = 0xC80;
pub fn allocate_picture_buffer(gfx: &crate::gfx::owner::Gfx) -> Result<(), Refused> {
    let pictures = &gfx.pictures;
    pictures.picture.size.set(BUFFER_LEN);
    // Rust's (`gfx::picture`), counted on the list as the game's block was.
    pictures.picture.buffer.set(take(
        Buffer::Main,
        &pictures.picture.list,
        BUFFER_LEN + SPARE,
    )?);
    Ok(())
}
