//! `gfx::free_picture_buffer` (0x4060B7): the picture buffer given back at the game's end:
//! `g_pictureFlags` and `g_pictureBuffer` 0, its list (`stru_4F49E4`) freed
//! (`memory::free_list`). The other half of `gfx::allocate_picture_buffer`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! One caller, `start`'s way out, which calls it directly (2026-10-23). Refuses (before
//! anything) what `free_list` refuses: the game's own heap with blocks on the list. The
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::framework::{game_ptr, game_read, game_write, rva};
use crate::memory::own_heap;

const FLAGS_RVA: usize = rva(0x4F49CC); // g_pictureFlags
const BUFFER_RVA: usize = rva(0x4F49DC); // g_pictureBuffer
const LIST_RVA: usize = rva(0x4F49E4); // stru_4F49E4

/// The Rust version (see the header).
pub fn free_picture_buffer() -> Result<(), BadInput> {
    unsafe {
        if game_read::<u32>(LIST_RVA) != 0 && own_heap() == 1 {
            return Err(BadInput("the game's own heap is in use (altmemenable)"));
        }
        game_write(FLAGS_RVA, 0u32);
        game_write(BUFFER_RVA, 0u32);
        crate::memory::free_list::free_list(game_ptr(LIST_RVA))
    }
}
