//! The replay's frame variables: the first step, the cameras' words, the frame and tenth kept,
//! the step's block and frame, and the last frame `quit_question` keeps.

use crate::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ReplayFrames {
    /// Set while the replay steps to a still (`physics::ease_view_turn`,
    /// `race::show_replay_still`).
    pub step_first: PlainDword,
    /// The first car block's first dword (`drive::replay_cameras`).
    pub camera_word_0: PlainDword,
    /// The first car block's second dword.
    pub camera_word_1: PlainDword,
    pub frame: PlainDword,
    /// The tenth within the frame (`drive::keep_replay_frame`, `drive::record_inputs`,
    /// `drive::flush_replay_if`).
    pub tenth: PlainDword,
    /// The block the step is in (`drive::load_replay_frame`, `drive::play_replay_frame`,
    /// `race::show_replay_still`).
    pub step_block: PlainDword,
    /// The frame the step is at (`drive::play_inputs`, `drive::play_replay_frame`).
    pub step_frame: PlainDword,
    /// The last frame `drive::quit_question` kept (-1 from `race::stage_loading`).
    pub last_kept: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl ReplayFrames {
    pub const fn new() -> Self {
        use crate::cells::ZeroCells;
        ZeroCells::ZERO
    }
}
