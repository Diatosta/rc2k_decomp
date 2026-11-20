//! `race::undo_edit`: the viewer's undo. With something kept (`undo_blocks` not 0) the block
//! count, the places and the cameras are restored from the undo tables (`race::keep_undo`'s), the
//! undo is emptied and the marks are reset (`race::reset_marks`).
//!
//! Refuses what `reset_marks` refuses (everything is put back).

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::system::owner::System;

/// Undoes the last edit; see the module header.
pub fn undo_edit(race: &Race, system: &System) -> Result<(), Refused> {
    let shared = &race.shared;
    use rc2k_core::race::viewer_flow::undo_count;
    let Some(count) = undo_count(race.viewer_buffers().undo_blocks.get()) else {
        return Ok(());
    };
    shared.replay_file().index.block_count.set(count);
    // The tables are copied cell by cell.
    let buffers = race.viewer_buffers();
    for (k, (to, from)) in shared
        .replay_file()
        .index
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
