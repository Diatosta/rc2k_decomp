//! `drive::keep_replay_frame` (0x45FFB6): the replay's frame and tenth (`frame`,
//! `tenth`, 0 when playing, `replay_mode` 2; else the first block's +8 / 100 and its
//! remainder); at a whole frame the four car blocks (0x3F80 bytes from `g_carBlockStore`) kept in
//! slot frame & 7 of `slots`, and while recording (`replay_mode` 1)
//! `drive::save_replay_cars`.
//!
//! Caller: `drive::replay_step`.

use crate::errors::BadInput;
use crate::pointers::copy_cells;
use crate::race::owner::Race;
use rc2k_core::drive::replay_frames::{
    PLAYING, RECORDING, SLOT_SIZE, frame_and_tenth, slot_offset,
};

const SIZE: usize = SLOT_SIZE as usize;

// Converted: `rc2k_core::drive::replay_frames::{frame_and_tenth, slot_offset}`.
pub fn keep_replay_frame(race: &Race) -> Result<(), BadInput> {
    race.replay.replay_frames.frame.set(0);
    race.replay.replay_frames.tenth.set(0);
    if race.course.numbers.replay_mode.get() != PLAYING {
        let (frame, tenth) = frame_and_tenth(race.car_blocks().cars[0].step_clock.get());
        race.replay.replay_frames.frame.set(frame);
        race.replay.replay_frames.tenth.set(tenth);
    }
    if race.replay.replay_frames.tenth.get() == 0 {
        let offset = slot_offset(race.replay.replay_frames.frame.get());
        // The first four blocks (`SLOT_SIZE` is four of them).
        copy_cells(
            (&race.car_blocks().cars, 0),
            (&race.replay.replay_frames.slots, offset as usize),
            SIZE,
        );
        if race.course.numbers.replay_mode.get() == RECORDING {
            crate::drive::save_replay_cars::save_replay_cars(race);
        }
    }
    Ok(())
}
