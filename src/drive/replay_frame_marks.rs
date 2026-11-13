//! `drive::replay_frame_marks` (0x41D8A3): the replay frame `replay_block`'s two marks
//! (`block_cameras`[frame]`, 8 bytes) into `block` ...
//!
//! Callers: `race::viewer_frame`, `race::play_replay_range`, `drive::load_replay_frame`.

use crate::errors::BadInput;
use crate::race::owner::Race;
use crate::race::viewer_buffers_store::ViewerBuffers;
use crate::session::{Run, read_run_bytes};

/// The two marks at byte `at` of the cameras. The original does not hold the frame to the
/// table's 0x708: past it the marks are the fields that follow the cameras in the viewer's
/// buffers, in order, to the record's end.
fn marks_at(buffers: &ViewerBuffers, at: usize) -> Result<[u32; 2], BadInput> {
    let runs: [&dyn Run; 12] = [
        &buffers.block_cameras,
        &buffers.undo_blocks,
        &buffers.undo_places,
        &buffers.undo_cameras,
        &buffers.mark_in,
        &buffers.mark_out,
        &buffers.camera_edit,
        &buffers.clip_blocks,
        &buffers.clip_places,
        &buffers.clip_cameras,
        &buffers.from_file,
        &buffers.to_file,
    ];
    let mut bytes = [0; 8];
    read_run_bytes(&runs, at, &mut bytes, "a replay frame's marks")?;
    Ok([
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
    ])
}

// Converted: `rc2k_core::drive::replay_frames::marks_offset`.
pub fn replay_frame_marks(race: &Race) -> Result<(), BadInput> {
    let replay = &race.replay;
    let at = rc2k_core::drive::replay_frames::marks_offset(replay.replay_view().replay_block.get());
    let marks = marks_at(race.viewer_buffers(), at as usize)?;
    for (k, mark) in marks.into_iter().enumerate() {
        race.viewer_buffers().block[k].set(mark);
    }
    Ok(())
}
