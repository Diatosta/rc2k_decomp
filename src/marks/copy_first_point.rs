//! `marks::copy_first_point` (0x4C1163): `dword_769306` .. `dword_76930E` into
//! `flt_811BDF` .. `flt_811BE7`, and `dword_7692F6` into `dword_811BCF` (`push`/`pop`).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4C1148`. Self-test: the four sources by lot, the original and the Rust
//! version compared on the four results; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::marks::store::{Slot, game_read, game_write};

/// (from, to): the point's three floats, then `dword_7692F6`.
const MOVES: [(Slot, Slot); 4] = [
    (Slot::at(0x769306), Slot::at(0x811BDF)),
    (Slot::at(0x76930A), Slot::at(0x811BE3)),
    (Slot::at(0x76930E), Slot::at(0x811BE7)),
    (Slot::at(0x7692F6), Slot::at(0x811BCF)),
];

/// The Rust version (see the header).
pub unsafe fn copy_first_point() {
    for (from, to) in MOVES {
        unsafe { game_write(to, game_read::<u32>(from)) };
    }
}
