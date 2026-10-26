//! `gfx::allocate_picture_buffer` (0x40608F): takes the memory pictures are decoded in.
//!
//! In:  nothing
//! Out: every register preserved
//! `dword_4F49E0` = 0x1D4C00 (800 x 600 dwords); that and 0xC80 bytes more from
//! `memory::allocate` on the list `stru_4F49E4`; the address to `g_pictureBuffer`
//! (0x4F49DC). A failed allocation would be fatal error 0x80A19090, but the allocator
//! reports its own and does not come back. One caller, `start`. `sub_4060B7`, after it
//! in the listing, gives the memory back at the game's end.

use crate::errors::Refused;
use crate::gfx::picture_store::PICTURE;
use crate::memory::allocate::allocate;

// ---- game data ----

const BUFFER_LEN: u32 = 0x1D_4C00;
const SPARE: u32 = 0xC80;

// ---- Rust version ----

/// The Rust version (see the header).
pub fn allocate_picture_buffer() -> Result<(), Refused> {
    PICTURE.size.set(BUFFER_LEN);
    let memory = { allocate(PICTURE.list.as_ptr(), BUFFER_LEN + SPARE) }?;
    PICTURE.buffer.set(crate::pointers::game_address(memory));
    Ok(())
}
