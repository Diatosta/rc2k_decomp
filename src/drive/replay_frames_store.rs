//! The replay's frame variables: the first step, the cameras' words, the frame and tenth kept,
//! the step's block and frame, and the last frame `quit_question` keeps.

pub struct ReplayFrames {
    /// Set while the replay steps to a still (`physics::ease_view_turn`,
    /// `race::show_replay_still`).
    pub step_first: u32,
    /// The first car block's first dword (`drive::replay_cameras`).
    pub camera_word_0: u32,
    /// The first car block's second dword.
    pub camera_word_1: u32,
    pub frame: u32,
    /// The tenth within the frame (`drive::keep_replay_frame`, `drive::record_inputs`,
    /// `drive::flush_replay_if`).
    pub tenth: u32,
    /// The block the step is in (`drive::load_replay_frame`, `drive::play_replay_frame`,
    /// `race::show_replay_still`).
    pub step_block: u32,
    /// The frame the step is at (`drive::play_inputs`, `drive::play_replay_frame`).
    pub step_frame: u32,
    /// The last frame `drive::quit_question` kept (-1 from `race::stage_loading`).
    pub last_kept: u32,
}

impl ReplayFrames {
    pub const fn new() -> Self {
        Self {
            step_first: 0,
            camera_word_0: 0,
            camera_word_1: 0,
            frame: 0,
            tenth: 0,
            step_block: 0,
            step_frame: 0,
            last_kept: 0,
        }
    }
}
