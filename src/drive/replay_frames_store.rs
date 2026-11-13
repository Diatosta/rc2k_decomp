//! The replay's frame variables: the first step, the cameras' words, the frame and tenth kept, the step's block and frame, and the last frame `quit_question` keeps (`ReplayFrames.last_kept`, just before `g_carBlockStore`). Names are readings unless IDA's.

use crate::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ReplayFrames {
    /// +0x0 (0x6EF7CC): `dword_6EF7CC`: set while the replay steps to a still (`physics::ease_view_turn`, `race::show_replay_still`).
    pub step_first: PlainDword,
    /// +0x4 (0x6EF7D0): `dword_6EF7D0`: the first car block's first dword (`drive::replay_cameras`).
    pub camera_word_0: PlainDword,
    /// +0x8 (0x6EF7D4): `dword_6EF7D4`: the first car block's second dword (`drive::replay_cameras`).
    pub camera_word_1: PlainDword,
    /// +0xc (0x6EF7D8): `dword_6EF7D8`: the replay's frame (`drive::keep_replay_frame`).
    pub frame: PlainDword,
    /// +0x10 (0x6EF7DC): `dword_6EF7DC`: the tenth within the frame (`drive::keep_replay_frame`, `drive::record_inputs`, `drive::flush_replay_if`).
    pub tenth: PlainDword,
    /// +0x14 (0x6EF7E0): `dword_6EF7E0`: the block the step is in (`drive::load_replay_frame`, `drive::play_replay_frame`, `race::show_replay_still`).
    pub step_block: PlainDword,
    /// +0x18 (0x6EF7E4): `dword_6EF7E4`: the frame the step is at (`drive::play_inputs`, `drive::play_replay_frame`).
    pub step_frame: PlainDword,
    /// (0x70F3E8): `dword_70F3E8`: the last frame `drive::quit_question` kept (-1 from `race::stage_loading`).
    pub last_kept: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl ReplayFrames {
    pub const fn new() -> Self {
        use crate::cells::ZeroCells;
        ZeroCells::ZERO
    }
}
