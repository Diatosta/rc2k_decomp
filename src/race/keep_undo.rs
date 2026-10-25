//! `race::keep_undo` (0x41E438): the viewed replay kept for one undo before an edit: the
//! block count into `dword_9673F4`, the index's places (`lDistanceToMove`, 0x3840 bytes)
//! into `dword_9673F8`, the blocks' cameras (`unk_963BB4`) into `unk_96AC38`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//!
//! Callers: `race::viewer_action` (six places), `race::edit_camera_1`, `edit_camera_2`.

use crate::race::replay_file_store::REPLAY_FILE;
use crate::race::viewer::{TABLE_LEN, table};

/// The Rust version.
pub fn keep_undo() {
    // Converted: `rc2k_core::race::viewer_flow::copy_tables`.
    crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .undo_blocks
        .set(REPLAY_FILE.index_block_count.get());
    let n = TABLE_LEN / 4;
    unsafe {
        rc2k_core::race::viewer_flow::copy_tables(
            (
                table(
                    crate::race::viewer_buffers_store::VIEWER_BUFFERS.undo_places[0].addr(),
                    n,
                ),
                table(
                    crate::race::viewer_buffers_store::VIEWER_BUFFERS.undo_cameras[0].addr(),
                    n,
                ),
            ),
            (
                table(REPLAY_FILE.block_places[0].addr(), n),
                table(
                    crate::race::viewer_buffers_store::VIEWER_BUFFERS
                        .block_cameras
                        .addr(),
                    n,
                ),
            ),
        );
    }
}
