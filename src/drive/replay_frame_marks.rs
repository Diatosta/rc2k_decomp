//! `drive::replay_frame_marks` (0x41D8A3): the replay frame `replay_block`'s two marks
//! (`block_cameras`[frame]`, 8 bytes) into `block` ...
//!
//! Callers: `race::viewer_frame`, `race::play_replay_range`, `drive::load_replay_frame`.

use crate::errors::BadInput;
use crate::race::owner::Race;

// Converted: `rc2k_core::drive::replay_frames::marks_offset`.
pub fn replay_frame_marks(race: &Race) -> Result<(), BadInput> {
    let replay = &race.replay;
    let at = rc2k_core::drive::replay_frames::marks_offset(replay.replay_view.replay_block.get());
    let marks: [u32; 2] =
        crate::cells::load_value(&race.viewer_buffers().block_cameras, at as usize)?;
    for (k, mark) in marks.into_iter().enumerate() {
        race.viewer_buffers().block[k].set(mark);
    }
    Ok(())
}
