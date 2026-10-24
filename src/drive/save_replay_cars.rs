//! `drive::save_replay_cars` (0x4600B8): the first 0x1A1 dwords of the four car blocks
//! (`g_carBlockStore`), the race slots (0xF0) and `unk_71BF84` (0x100) copied in that
//! order to `unk_7172EC` (`rep movsd`): the replay's starting state.
//!
//! In:  nothing
//! Out: ecx 0, esi past the last piece, edi past the copy; the rest kept; no flags read
//!
//! Caller: `sub_45FFB6`.
//! Self-test: the original and the Rust version on the pieces filled by pattern (saved
//! and put back).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::physics::flow::a;
use rc2k_core::drive::replay_frames::{StartPiece, start_layout};

const B: usize = 0x70F3EC; // g_carBlockStore (four blocks of 0xFE0)
const SAVED: usize = 0x7172EC; // unk_7172EC

/// Where a piece of the starting state is.
pub fn piece_at(piece: StartPiece) -> usize {
    match piece {
        StartPiece::Car(k) => B + 0xFE0 * k as usize,
        StartPiece::RaceSlots => 0x71BAC8, // g_raceSlots
        StartPiece::Extra => 0x71BF84,     // unk_71BF84
    }
}

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::start_layout`.
pub fn save_replay_cars() {
    for (piece, offset, n) in start_layout() {
        let (state, place) = (a(SAVED) + 4 * offset as u32, a(piece_at(piece)));
        unsafe { std::ptr::copy(place as *const u32, state as *mut u32, n) };
    }
}
