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
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::work_store::{RACE_WORK, RaceWork};
use rc2k_core::drive::replay_frames::{StartPiece, start_layout};

/// Where a piece of the starting state is now: a car's block, the race slots (`g_raceSlots`) or
/// `unk_71BF84` (its 0x400 bytes run on through the computer driver's fields: bounded to
/// the race work).
pub fn piece_place(piece: StartPiece) -> GamePtr {
    match piece {
        StartPiece::Car(k) => GamePtr::to(&CAR_BLOCKS.cars[k as usize]),
        StartPiece::RaceSlots => GamePtr::to(&RACE_WORK.race_slots),
        StartPiece::Extra => {
            GamePtr::to(&RACE_WORK).offset(core::mem::offset_of!(RaceWork, replay_extra) as u32)
        }
    }
}

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::start_layout`.
pub fn save_replay_cars() -> Result<(), BadInput> {
    for (piece, offset, n) in start_layout() {
        let (state, place) = (
            crate::race::blocks_store::replay_state().offset(4 * offset as u32),
            piece_place(piece),
        );
        place.copy_to(state, n * 4, "the cars' starting state")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc2k_core::strings::bytes::Bytes;

    #[test]
    fn every_piece_fits_its_place() {
        for (piece, _, n) in start_layout() {
            let room = piece_place(piece).bytes("a piece").unwrap();
            assert!(room.byte(n * 4 - 1).is_some(), "{piece:?}: {n:#x} dwords");
        }
    }
}
