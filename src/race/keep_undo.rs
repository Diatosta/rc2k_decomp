//! `race::keep_undo` (0x41E438): the viewed replay kept for one undo before an edit: the block
//! count into `undo_blocks`, the index's places (`lDistanceToMove`, 0x3840 bytes) into
//! `undo_places`, the blocks' cameras (`block_cameras`) into `undo_cameras`.
//!
//! Callers: `race::viewer_action` (six places), `race::edit_camera_1`, `edit_camera_2`.

use crate::race::owner_shared::RaceShared;
use crate::race::viewer_buffers_store::VIEWER_BUFFERS;

pub fn keep_undo(shared: &RaceShared) {
    // As `rc2k_core::race::viewer_flow::copy_tables`, cell by cell.
    VIEWER_BUFFERS
        .undo_blocks
        .set(shared.replay_file.index_block_count.get());
    for (k, (to, from)) in VIEWER_BUFFERS
        .undo_places
        .iter()
        .zip(&shared.replay_file.block_places)
        .enumerate()
    {
        to.set(from.get());
        VIEWER_BUFFERS.undo_cameras[k].set(VIEWER_BUFFERS.block_cameras.get_u32(4 * k));
    }
}
