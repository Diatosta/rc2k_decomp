//! `drive::save_replay_cars` (0x4600B8): the first 0x1A1 dwords of the four car blocks
//! (`g_carBlockStore`), the race slots (0xF0) and `unk_71BF84` (0x100) copied in that
//! order to `unk_7172EC` (`rep movsd`): the replay's starting state.
//!
//! In:  nothing
//! Out: ecx 0, esi past the last piece, edi past the copy; the rest kept; no flags read
//!
//! Caller: `sub_45FFB6`.

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use crate::race::work_store::RACE_WORK;
use rc2k_core::drive::replay_frames::{StartPiece, start_layout};

/// Where a piece of the starting state is now (an address).
pub fn piece_addr(piece: StartPiece) -> u32 {
    match piece {
        StartPiece::Car(k) => crate::race::blocks_store::block_addr(k as usize),
        StartPiece::RaceSlots => RACE_WORK.race_slots[0].part.addr(), // g_raceSlots
        StartPiece::Extra => RACE_WORK.replay_extra.addr(),           // unk_71BF84
    }
}

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::start_layout`.
pub fn save_replay_cars() -> Result<(), BadInput> {
    for (piece, offset, n) in start_layout() {
        let (state, place) = (
            crate::race::blocks_store::replay_state().offset(4 * offset as u32),
            piece_addr(piece),
        );
        GamePtr::new(place).copy_to(state, n * 4, "the cars' starting state")?;
    }
    Ok(())
}
