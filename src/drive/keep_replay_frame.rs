//! `drive::keep_replay_frame` (0x45FFB6): the replay's frame and tenth (`dword_6EF7D8`,
//! `dword_6EF7DC`, 0 when playing, `dword_5C5CE0` 2; else the first block's +8 / 100 and
//! its remainder); at a whole frame the four car blocks (0x3F80 bytes from
//! `g_carBlockStore`) kept in slot frame & 7 of `unk_6EF7E8`, and while recording
//! (`dword_5C5CE0` 1) `drive::save_replay_cars`.
//!
//! In:  nothing
//! Out: eax / edx / ebx as the `div` leaves them (not playing), at a whole frame eax the
//!      slot's offset (`imul`), ecx 0, esi / edi past the copy, or as `save_replay_cars` leaves them; the rest kept; no
//!      flags read
//!
//! Caller: `drive::replay_step`.

use crate::errors::BadInput;
use crate::pointers::copy_cells;
use crate::race::blocks_store::CAR_BLOCKS;
use rc2k_core::drive::replay_frames::{
    PLAYING, RECORDING, SLOT_SIZE, frame_and_tenth, slot_offset,
};

const SIZE: usize = SLOT_SIZE as usize;

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::{frame_and_tenth, slot_offset}`.
pub fn keep_replay_frame() -> Result<(), BadInput> {
    crate::race::owner::race().replay.replay_frames.frame.set(0);
    crate::race::owner::race().replay.replay_frames.tenth.set(0);
    if crate::race::owner::race().course.numbers.replay_mode.get() != PLAYING {
        let (frame, tenth) = frame_and_tenth(CAR_BLOCKS.cars[0].step_clock.bits());
        crate::race::owner::race()
            .replay
            .replay_frames
            .frame
            .set(frame);
        crate::race::owner::race()
            .replay
            .replay_frames
            .tenth
            .set(tenth);
    }
    if crate::race::owner::race().replay.replay_frames.tenth.get() == 0 {
        let offset = slot_offset(crate::race::owner::race().replay.replay_frames.frame.get());
        // The first four blocks (`SLOT_SIZE` is four of them).
        copy_cells(
            (&CAR_BLOCKS.cars, 0),
            (
                &crate::race::owner::race().replay.replay_frames.slots,
                offset as usize,
            ),
            SIZE,
        );
        if crate::race::owner::race().course.numbers.replay_mode.get() == RECORDING {
            crate::drive::save_replay_cars::save_replay_cars();
        }
    }
    Ok(())
}
