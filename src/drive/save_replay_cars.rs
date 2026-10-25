//! `drive::save_replay_cars` (0x4600B8): the first 0x1A1 dwords of the four car blocks
//! (`g_carBlockStore`), the race slots (0xF0) and `unk_71BF84` (0x100) copied in that
//! order to `unk_7172EC` (`rep movsd`): the replay's starting state.
//!
//! In:  nothing
//! Out: ecx 0, esi past the last piece, edi past the copy; the rest kept; no flags read
//!
//! Caller: `sub_45FFB6`.

use crate::race::work_store::RACE_WORK;
use rc2k_core::drive::replay_frames::{StartPiece, start_layout};

const B: usize = 0x70F3EC; // g_carBlockStore (four blocks of 0xFE0)

/// Where a piece of the starting state is, as IDA's address (`drive::load_replay_cars`
/// still takes this; it should take `piece_addr`, and this go).
pub fn piece_at(piece: StartPiece) -> usize {
    match piece {
        StartPiece::Car(k) => B + 0xFE0 * k as usize,
        StartPiece::RaceSlots => 0x71BAC8, // g_raceSlots
        StartPiece::Extra => 0x71BF84,     // unk_71BF84
    }
}

/// Where a piece of the starting state is now (an address).
pub fn piece_addr(piece: StartPiece) -> usize {
    match piece {
        StartPiece::Car(k) => crate::race::blocks_store::block_addr(k as usize),
        StartPiece::RaceSlots => RACE_WORK.race_slots[0].part.addr(), // g_raceSlots
        StartPiece::Extra => RACE_WORK.replay_extra.addr(),           // unk_71BF84
    }
}

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::start_layout`.
pub fn save_replay_cars() {
    for (piece, offset, n) in start_layout() {
        let (state, place) = (
            crate::race::blocks_store::block_addr(8) as u32 + 4 * offset as u32,
            piece_addr(piece) as u32,
        );
        unsafe { std::ptr::copy(place as *const u32, state as *mut u32, n) };
    }
}
