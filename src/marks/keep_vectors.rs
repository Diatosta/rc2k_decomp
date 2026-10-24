//! `marks::keep_vectors` (0x4C1339): twelve dwords kept (`push`/`pop`): `dword_769252` ..
//! and `dword_76924E` into `dword_76926E` .. and `dword_76926A`; `dword_764FB0` .. and
//! `dword_764FAC` into `dword_7692C2` .. and `dword_7692BE`; `dword_764F94` .. and
//! `dword_764F90` into `dword_7692A6` .. and `dword_7692A2`.
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4C1270`. Self-test: the twelve sources by lot, compared on the twelve
//! results; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_read, game_write, rva};

/// (from, to), in the original's order.
const MOVES: [(usize, usize); 12] = [
    (rva(0x769252), rva(0x76926E)),
    (rva(0x769256), rva(0x769272)),
    (rva(0x76925A), rva(0x769276)),
    (rva(0x76924E), rva(0x76926A)),
    (rva(0x764FB0), rva(0x7692C2)),
    (rva(0x764FB4), rva(0x7692C6)),
    (rva(0x764FB8), rva(0x7692CA)),
    (rva(0x764FAC), rva(0x7692BE)),
    (rva(0x764F94), rva(0x7692A6)),
    (rva(0x764F98), rva(0x7692AA)),
    (rva(0x764F9C), rva(0x7692AE)),
    (rva(0x764F90), rva(0x7692A2)),
];

/// The Rust version (see the header).
pub unsafe fn keep_vectors() {
    for (from, to) in MOVES {
        unsafe { game_write(to, game_read::<u32>(from)) };
    }
}
