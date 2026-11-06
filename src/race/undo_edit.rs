//! `race::undo_edit` (0x41E46B): the viewer's undo: with something kept
//! (`dword_9673F4` not 0) the block count, the places and the cameras back from the undo
//! tables (`race::keep_undo`'s), the undo emptied and the marks reset
//! (`race::reset_marks`).
//!
//! In:  nothing
//! Out: nothing kept: every register kept. Else eax the count, ecx 0, esi and edi past the
//!      cameras' copy; no flags read
//!
//! Callers: `race::viewer_action` (action 0x13, and 0x20). Refuses what `reset_marks` refuses
//! (everything put back).

use crate::errors::Refused;
use crate::race::owner_replay::Replay;
use crate::race::replay_file_store::REPLAY_FILE;

/// The Rust version. What the original leaves its one caller, `viewer_action` (which
/// keeps every register), does not read.
pub fn undo_edit(replay: &Replay) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer_flow::{undo_count, copy_tables}`.
    use rc2k_core::race::viewer_flow::undo_count;
    let Some(count) = undo_count(
        crate::race::viewer_buffers_store::VIEWER_BUFFERS
            .undo_blocks
            .get(),
    ) else {
        return Ok(());
    };
    REPLAY_FILE.index_block_count.set(count);
    // As `rc2k_core::race::viewer_flow::copy_tables`, cell by cell.
    let buffers = &crate::race::viewer_buffers_store::VIEWER_BUFFERS;
    for (k, (to, from)) in REPLAY_FILE
        .block_places
        .iter()
        .zip(&buffers.undo_places)
        .enumerate()
    {
        to.set(from.get());
        buffers
            .block_cameras
            .set_u32(4 * k, buffers.undo_cameras[k].get());
    }
    crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .undo_blocks
        .set(0);
    crate::race::reset_marks::reset_marks(replay)
}
