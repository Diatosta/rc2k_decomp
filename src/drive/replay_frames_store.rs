//! The replay's frame variables: the first step, the cameras' words, the frame and tenth kept, the step's block and frame, the eight kept frames of the car blocks (`ReplayFrames.slots`, 0x3F80 bytes each) and the last frame `quit_question` keeps (`ReplayFrames.last_kept`, just before `g_carBlockStore`). Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{Bytes, Dword};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ReplayFrames {
    /// +0x0 (0x6EF7CC): `dword_6EF7CC`: set while the replay steps to a still (`physics::ease_view_turn`, `race::show_replay_still`).
    pub step_first: Dword,
    /// +0x4 (0x6EF7D0): `dword_6EF7D0`: the first car block's first dword (`drive::replay_cameras`).
    pub camera_word_0: Dword,
    /// +0x8 (0x6EF7D4): `dword_6EF7D4`: the first car block's second dword (`drive::replay_cameras`).
    pub camera_word_1: Dword,
    /// +0xc (0x6EF7D8): `dword_6EF7D8`: the replay's frame (`drive::keep_replay_frame`).
    pub frame: Dword,
    /// +0x10 (0x6EF7DC): `dword_6EF7DC`: the tenth within the frame (`drive::keep_replay_frame`, `drive::record_inputs`, `drive::flush_replay_if`).
    pub tenth: Dword,
    /// +0x14 (0x6EF7E0): `dword_6EF7E0`: the block the step is in (`drive::load_replay_frame`, `drive::play_replay_frame`, `race::show_replay_still`).
    pub step_block: Dword,
    /// +0x18 (0x6EF7E4): `dword_6EF7E4`: the frame the step is at (`drive::play_inputs`, `drive::play_replay_frame`).
    pub step_frame: Dword,
    /// +0x1c (0x6EF7E8): `unk_6EF7E8`: eight kept frames of 0x3F80 bytes (`drive::keep_replay_frame`).
    pub slots: Bytes<0x1FC00>,
    /// +0x1fc1c (0x70F3E8): `dword_70F3E8`: the last frame `drive::quit_question` kept (-1 from `race::stage_loading`).
    pub last_kept: Dword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
impl ReplayFrames {
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x6EF7CC, the first byte of the piece.
pub const START: usize = 0x6EF7CC;
/// IDA 0x70F3EC, the end of the piece.
pub const END: usize = 0x70F3EC;

const _: () = assert!(END - START == core::mem::size_of::<ReplayFrames>());

/// Where IDA 0x6EF7CC is now (its own static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&race().replay.replay_frames))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &race().replay.replay_frames,
        );
    }

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(
            piece_addr() % (core::mem::align_of::<ReplayFrames>()) as u32,
            0
        );

        assert_eq!(offset_of!(ReplayFrames, step_first), 0x0);
        assert_eq!(offset_of!(ReplayFrames, frame), 0xc);
        assert_eq!(offset_of!(ReplayFrames, tenth), 0x10);
        assert_eq!(offset_of!(ReplayFrames, step_frame), 0x18);
        assert_eq!(offset_of!(ReplayFrames, slots), 0x1c);
        assert_eq!(offset_of!(ReplayFrames, last_kept), 0x1fc1c);
    }
}
