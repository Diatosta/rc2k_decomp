//! `gfx::free_picture_buffer` (0x4060B7): the picture buffer given back at the game's end:
//! `g_pictureFlags` and `g_pictureBuffer` 0, its list (`list`) freed
//! (`memory::free_list`; the buffer, Rust's, given back). The other half of
//! `gfx::allocate_picture_buffer`.
//!
//! One caller, `start`'s way out, which calls it directly. Refuses (before
//! anything) what `free_list` refuses: a list holding a block.

use crate::errors::BadInput;
pub fn free_picture_buffer(gfx: &crate::gfx::owner::Gfx) -> Result<(), BadInput> {
    let pictures = &gfx.pictures;
    if pictures.picture.list.get().head != 0 {
        return Err(BadInput("a memory list holding a block"));
    }
    pictures.picture.flags.set(0u32);
    pictures.picture.buffer.set(0u32);
    crate::gfx::picture::give_back(crate::gfx::picture::Buffer::Main);
    crate::memory::free_list::free_list(&pictures.picture.list)
}
