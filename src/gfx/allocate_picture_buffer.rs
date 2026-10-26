//! `gfx::allocate_picture_buffer` (0x40608F): takes the memory pictures are decoded in.
//!
//! In:  nothing
//! Out: every register preserved
//! `dword_4F49E0` = 0x1D4C00 (800 x 600 dwords); that and 0xC80 bytes more from
//! `memory::allocate` on the list `stru_4F49E4`; the address to `g_pictureBuffer`
//! (0x4F49DC). A failed allocation would be fatal error 0x80A19090, but the allocator
//! reports its own and does not come back. One caller, `start`. `sub_4060B7`, after it
//! in the listing, gives the memory back at the game's end.
//! Shadow mode runs the original alone (memory is taken).
//! Self-test: a redirected copy whose three variables are the test's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::memory::MemoryList;
use crate::memory::allocate::allocate;

// ---- game data ----

/// dword_4F49E0, stru_4F49E4, g_pictureBuffer.
const PLACES: [usize; 3] = [0x4F49E0, 0x4F49E4, 0x4F49DC];
const BUFFER_LEN: u32 = 0x1D_4C00;
const SPARE: u32 = 0xC80;

// ---- Rust version ----

/// `size`, `list` and `buffer` are the three variables' addresses.
pub unsafe fn allocate_picture_buffer(
    size: *mut u32,
    list: *mut MemoryList,
    buffer: *mut u32,
) -> Result<(), Refused> {
    unsafe {
        size.write_unaligned(BUFFER_LEN);
        let memory = allocate(list, BUFFER_LEN + SPARE)?;
        buffer.write_unaligned(memory as u32);
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    let [size, list, buffer] = PLACES.map(|at| game_addr(rva(at)));
    let done = unsafe { allocate_picture_buffer(size as _, list as _, buffer as _) };
    done?;
    Ok(())
}
