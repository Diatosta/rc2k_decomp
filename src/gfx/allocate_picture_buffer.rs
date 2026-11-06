//! `gfx::allocate_picture_buffer` (0x40608F): takes the memory pictures are decoded in.
//!
//! In:  nothing
//! Out: every register preserved
//! `dword_4F49E0` = 0x1D4C00 (800 x 600 dwords); that and 0xC80 bytes more from
//! `memory::allocate` on the list `stru_4F49E4` (Rust's since phase 12: `gfx::picture`); the
//! address to `g_pictureBuffer` (here the buffer's number)
//! (0x4F49DC). A failed allocation would be fatal error 0x80A19090, but the allocator
//! reports its own and does not come back. One caller, `start`. `sub_4060B7`, after it
//! in the listing, gives the memory back at the game's end.

use crate::errors::Refused;
use crate::gfx::picture::{Buffer, take};

// ---- game data ----

const BUFFER_LEN: u32 = 0x1D_4C00;
const SPARE: u32 = 0xC80;

// ---- Rust version ----

/// The Rust version (see the header).
pub fn allocate_picture_buffer() -> Result<(), Refused> {
    crate::gfx::owner::gfx()
        .pictures
        .picture
        .size
        .set(BUFFER_LEN);
    // Rust's since phase 12 (`gfx::picture`), counted on the list as the game's block was.
    crate::gfx::owner::gfx().pictures.picture.buffer.set(take(
        Buffer::Main,
        crate::pointers::GamePtr::to(&crate::gfx::owner::gfx().pictures.picture.list),
        BUFFER_LEN + SPARE,
    )?);
    Ok(())
}
