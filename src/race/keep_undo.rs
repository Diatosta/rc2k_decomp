//! `race::keep_undo`: keeps the viewed replay for one undo before an edit: the block count into
//! `undo_blocks`, the index's places (0x3840 bytes) into `undo_places`, the blocks' cameras
//! (`block_cameras`) into `undo_cameras`.

use crate::race::owner::Race;

pub fn keep_undo(race: &Race) {
    let shared = &race.shared;
    let buffers = race.viewer_buffers();
    // Cell by cell, as `rc2k_core::race::viewer_flow::copy_tables`.
    buffers
        .undo_blocks
        .set(shared.replay_file().index.block_count.get());
    for (k, (to, from)) in buffers
        .undo_places
        .iter()
        .zip(&shared.replay_file().index.block_places)
        .enumerate()
    {
        to.set(from.get());
        buffers.undo_cameras[k].set(buffers.block_cameras.get_u32(4 * k));
    }
}
