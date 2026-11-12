//! `race::undo_edit` (0x41E46B): the viewer's undo: with something kept (`undo_blocks` not 0) the
//! block count, the places and the cameras back from the undo tables (`race::keep_undo`'s), the
//! undo emptied and the marks reset (`race::reset_marks`).
//!
//! Callers: `race::viewer_action` (action 0x13, and 0x20). Refuses what `reset_marks` refuses
//! (everything put back).

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::system::owner::System;

/// What the original leaves its one caller, `viewer_action`, does not read.
pub fn undo_edit(race: &Race, system: &System) -> Result<(), Refused> {
    let shared = &race.shared;
    // Converted: `rc2k_core::race::viewer_flow::{undo_count, copy_tables}`.
    use rc2k_core::race::viewer_flow::undo_count;
    let Some(count) = undo_count(race.viewer_buffers().undo_blocks.get()) else {
        return Ok(());
    };
    shared.replay_file().index_block_count.set(count);
    // As `rc2k_core::race::viewer_flow::copy_tables`, cell by cell.
    let buffers = race.viewer_buffers();
    for (k, (to, from)) in shared
        .replay_file()
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
    race.viewer_buffers().undo_blocks.set(0);
    crate::race::reset_marks::reset_marks(race, system)
}
