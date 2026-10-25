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

use crate::physics::flow::{a, g, p};
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::numbers_store::NUMBERS;
use rc2k_core::drive::replay_frames::{
    PLAYING, RECORDING, SLOT_SIZE, frame_and_tenth, slot_offset,
};

const SLOTS: usize = 0x6EF7E8; // unk_6EF7E8: eight of 0x3F80
const SIZE: usize = SLOT_SIZE as usize;

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::{frame_and_tenth, slot_offset}`.
pub unsafe fn keep_replay_frame() {
    p(0x6EF7D8, 0);
    p(0x6EF7DC, 0);
    if NUMBERS.replay_mode.get() != PLAYING {
        let (frame, tenth) = frame_and_tenth(CAR_BLOCKS.cars[0].step_clock.bits());
        p(0x6EF7D8, frame);
        p(0x6EF7DC, tenth);
    }
    if g(0x6EF7DC) == 0 {
        let offset = slot_offset(g(0x6EF7D8));
        let to = a(SLOTS) + offset;
        unsafe {
            std::ptr::copy(
                crate::race::blocks_store::block_addr(0) as *const u8,
                to as *mut u8,
                SIZE,
            )
        };
        if NUMBERS.replay_mode.get() == RECORDING {
            crate::drive::save_replay_cars::save_replay_cars();
        }
    }
}
