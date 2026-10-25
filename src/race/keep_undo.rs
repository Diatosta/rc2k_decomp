//! `race::keep_undo` (0x41E438): the viewed replay kept for one undo before an edit: the
//! block count into `dword_9673F4`, the index's places (`lDistanceToMove`, 0x3840 bytes)
//! into `dword_9673F8`, the blocks' cameras (`unk_963BB4`) into `unk_96AC38`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//!
//! Callers: `race::viewer_action` (six places), `race::edit_camera_1`, `edit_camera_2`.

use crate::physics::flow::a;
use crate::race::replay_file_store::REPLAY_FILE;
use crate::race::viewer::{CAMERAS, TABLE_LEN, UNDO_BLOCKS, UNDO_CAMERAS, UNDO_PLACES, put, table};

/// The Rust version.
pub fn keep_undo() {
    // Converted: `rc2k_core::race::viewer_flow::copy_tables`.
    put(UNDO_BLOCKS, REPLAY_FILE.index_block_count.get());
    let n = TABLE_LEN / 4;
    unsafe {
        rc2k_core::race::viewer_flow::copy_tables(
            (
                table(a(UNDO_PLACES) as usize, n),
                table(a(UNDO_CAMERAS) as usize, n),
            ),
            (
                table(REPLAY_FILE.block_places[0].addr(), n),
                table(a(CAMERAS) as usize, n),
            ),
        );
    }
}
