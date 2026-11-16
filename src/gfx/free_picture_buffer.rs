//! Gives the picture buffer back at the game's end: the picture flags and the buffer's number are
//! 0, its list freed (`memory::free_list`; the buffer, Rust's, given back). The other half of
//! `gfx::allocate_picture_buffer`. Refuses (before anything) what `free_list` refuses: a list
//! holding a block.

use crate::errors::BadInput;
pub fn free_picture_buffer(gfx: &crate::gfx::owner::Gfx) -> Result<(), BadInput> {
    let pictures = &gfx.pictures;
    if pictures.picture.list.get().head != 0 {
        return Err(BadInput("a memory list holding a block"));
    }
    pictures.picture.flags.set(0u32);
    pictures.picture.buffer.set(0u32);
    crate::gfx::picture::give_back(&pictures.buffers, crate::gfx::picture::Buffer::Main);
    crate::memory::free_list::free_list(&pictures.picture.list)
}
