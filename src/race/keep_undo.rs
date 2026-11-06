//! `race::keep_undo` (0x41E438): the viewed replay kept for one undo before an edit: the
//! block count into `dword_9673F4`, the index's places (`lDistanceToMove`, 0x3840 bytes)
//! into `dword_9673F8`, the blocks' cameras (`unk_963BB4`) into `unk_96AC38`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//!
//! Callers: `race::viewer_action` (six places), `race::edit_camera_1`, `edit_camera_2`.

use crate::race::owner_shared::Shared;
use crate::race::viewer_buffers_store::VIEWER_BUFFERS;

/// The Rust version.
pub fn keep_undo(shared: &Shared) {
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
