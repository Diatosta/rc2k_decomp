//! `marks::pick_square` (0x4C0623): one step of the squares' own random (`dword_8BC256`,
//! times 0xC5 plus 0x5D) split into a place (bits 0 .. 10, 11 .. 20) and a kind (bits 21 ..
//! 23) for `marks::spawn_square`.
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `marks::squares_spawn` (twice). The call is `spawn_square`'s Rust version.
//! Self-test: the original and the Rust version on the game's items, randoms by lot.
//!
//! Converted 2026-10-21: the hook's Rust version is `pick_square_plain`, through
//! `rc2k_core::marks::items::pick_square`; compared exactly as before (the earlier version
//! stays for the Rust callers in unconverted code).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

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
