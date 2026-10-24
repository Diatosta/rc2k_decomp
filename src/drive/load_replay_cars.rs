//! `drive::load_replay_cars` (0x46010D): `drive::save_replay_cars` the other way: the
//! replay's starting state at `unk_7172EC` copied back into the four car blocks, the race
//! slots and `unk_71BF84`.
//!
//! In:  nothing
//! Out: ecx 0, esi past the copy, edi past the last piece; the rest kept; no flags read
//!
//! Callers: `sub_460049`, `sub_46062E` (twice).
//! Self-test: the original and the Rust version on the copy filled by pattern (saved and
//! put back).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::drive::save_replay_cars::piece_at;
use crate::physics::flow::a;
use rc2k_core::drive::replay_frames::start_layout;

const SAVED: usize = 0x7172EC; // unk_7172EC

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::start_layout`.
pub fn load_replay_cars() {
    for (piece, offset, n) in start_layout() {
        let (state, place) = (a(SAVED) + 4 * offset as u32, a(piece_at(piece)));
        unsafe { std::ptr::copy(state as *const u32, place as *mut u32, n) };
    }
}
