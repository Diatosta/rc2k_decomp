//! `drive::save_replay_cars` (0x4600B8): the first 0x1A1 dwords of the four car blocks
//! (`g_carBlockStore`), the race slots (0xF0) and `unk_71BF84` (0x100) copied in that order to
//! `unk_7172EC` (`rep movsd`): the replay's starting state.
//!
//! Caller: `drive::keep_replay_frame`.

use crate::pointers::{Cells, copy_cells};
use crate::race::blocks_store::{CAR_BLOCKS, REPLAY_STATE_OFFSET};
use crate::race::groups_store::CarsAndReplay;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::race::work_store::RaceWork;
use rc2k_core::drive::replay_frames::{StartPiece, start_layout};

/// Where a piece of the starting state is now: a car's block, the race slots (`g_raceSlots`) or
/// `unk_71BF84` (its 0x400 bytes run on through the computer driver's fields: bounded to
/// the race work). For the replay check, which reads it dword by dword.
pub fn piece_place(shared: &RaceShared, piece: StartPiece) -> (&dyn crate::pointers::Store, usize) {
    match piece {
        StartPiece::Car(k) => (&CAR_BLOCKS.cars[k as usize], 0),
        StartPiece::RaceSlots => (&shared.work.race_slots, 0),
        StartPiece::Extra => (&shared.work, core::mem::offset_of!(RaceWork, replay_extra)),
    }
}

/// Copies `n` dwords between a piece of the starting state and its place: a car's block, the
/// race slots (`g_raceSlots`) or `unk_71BF84` (its 0x400 bytes run on through the computer
/// driver's fields: bounded to the race work). `offset` is the dword where the piece begins in
/// the state (`CARS_AND_REPLAY` from car block 8); `save` copies the piece into the state,
/// else the state back into the piece.
pub fn copy_piece(race: &Race, piece: StartPiece, offset: usize, n: usize, save: bool) {
    let shared = &race.shared;
    let state = (
        &race.replay.cars_and_replay,
        REPLAY_STATE_OFFSET + 4 * offset,
    );
    let len = 4 * n;
    match piece {
        StartPiece::Car(k) => copy_between((&CAR_BLOCKS.cars[k as usize], 0), state, len, save),
        StartPiece::RaceSlots => copy_between(
            (&shared.work, core::mem::offset_of!(RaceWork, race_slots)),
            state,
            len,
            save,
        ),
        StartPiece::Extra => copy_between(
            (&shared.work, core::mem::offset_of!(RaceWork, replay_extra)),
            state,
            len,
            save,
        ),
    }
}

/// `len` bytes from `piece` to `state` when `save`, else back.
fn copy_between<P: Cells>(
    piece: (&P, usize),
    state: (&CarsAndReplay, usize),
    len: usize,
    save: bool,
) {
    if save {
        copy_cells(piece, state, len);
    } else {
        copy_cells(state, piece, len);
    }
}

// Converted: `rc2k_core::drive::replay_frames::start_layout`.
pub fn save_replay_cars(race: &Race) {
    for (piece, offset, n) in start_layout() {
        copy_piece(race, piece, offset, n, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc2k_core::strings::bytes::Bytes;

    #[test]
    fn every_piece_fits_its_place() {
        for (piece, _, n) in start_layout() {
            let (store, at) = piece_place(&crate::race::owner::race().shared, piece);
            let room = crate::pointers::GamePtr::of_store(store)
                .offset(at as u32)
                .bytes("a piece")
                .unwrap();
            assert!(room.byte(n * 4 - 1).is_some(), "{piece:?}: {n:#x} dwords");
        }
    }

    #[test]
    fn every_piece_copies_inside_its_records() {
        // A piece past its record's end would panic in `copy_cells`. Checked by size, as a
        // copy here would change the stores other tests read.
        use core::mem::{offset_of, size_of};
        for (piece, offset, n) in start_layout() {
            let len = 4 * n;
            assert!(
                REPLAY_STATE_OFFSET + 4 * offset + len <= size_of::<CarsAndReplay>(),
                "{piece:?}: the state"
            );
            let (at, room) = match piece {
                StartPiece::Car(_) => (0, size_of::<crate::physics::car_store::Car>()),
                StartPiece::RaceSlots => (offset_of!(RaceWork, race_slots), size_of::<RaceWork>()),
                StartPiece::Extra => (offset_of!(RaceWork, replay_extra), size_of::<RaceWork>()),
            };
            assert!(at + len <= room, "{piece:?}: {n:#x} dwords");
        }
    }
}
