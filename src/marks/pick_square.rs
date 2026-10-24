//! `marks::pick_square` (0x4C0623): one step of the squares' own random (`dword_8BC256`,
//! times 0xC5 plus 0x5D) split into a place (bits 0 .. 10, 11 .. 20) and a kind (bits 21 ..
//! 23) for `marks::spawn_square`.
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `marks::squares_spawn` (twice). The call is `spawn_square`'s Rust version.
//!
//! compared exactly as before (the earlier version stays for the Rust callers in
//! unconverted code).

use crate::marks::spawn_square::spawn_square;
use crate::marks::store::{Slot, game_read, game_write};

const RANDOM_RVA: Slot = Slot::at(0x8BC256); // dword_8BC256

/// The Rust version (see the header).
pub unsafe fn pick_square() {
    unsafe {
        let s = game_read::<u32>(RANDOM_RVA)
            .wrapping_mul(0xC5)
            .wrapping_add(0x5D);
        game_write(RANDOM_RVA, s);
        spawn_square(s & 0x7FF, (s >> 11) & 0x3FF, (s >> 21) & 7);
    }
}
