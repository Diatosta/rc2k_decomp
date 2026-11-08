//! `drive::replay_frame_marks` (0x41D8A3): the replay frame `dword_5C2530`'s two marks
//! (`unk_963BB4[frame]`, 8 bytes) into `dword_9613A4` ...
//!
//! Callers: `race::viewer_frame`, `race::play_replay_range`, `drive::load_replay_frame`.

use crate::errors::BadInput;
use crate::race::owner::Race;

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::marks_offset`.
pub fn replay_frame_marks(race: &Race) -> Result<(), BadInput> {
    let replay = &race.replay;
    let at = rc2k_core::drive::replay_frames::marks_offset(replay.replay_view.replay_block.get());
    let marks: [u32; 2] = crate::pointers::load_value(
        &crate::race::viewer_buffers_store::VIEWER_BUFFERS.block_cameras,
        at as usize,
    )?;
    for (k, mark) in marks.into_iter().enumerate() {
        crate::race::viewer_buffers_store::VIEWER_BUFFERS.block[k].set(mark);
    }
    Ok(())
}
