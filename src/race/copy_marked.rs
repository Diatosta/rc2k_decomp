//! `race::copy_marked` (0x41E2F6): the marked blocks (from `mark_in` up to `mark_out`)
//! onto the clipboard: their count into `clip_blocks`, their places (`lDistanceToMove`) into
//! `clip_places` and their cameras (`block_cameras`) into `clip_cameras`. Nothing when the marks are not
//! in order (`jle`, signed).
//!
//! Callers: `race::viewer_action` (action 0x1B, and in 0x11 before the cut). Refused, before
//! anything is written: a first mark below 0 or a second past 0x708 (the original copies from
//! outside the tables).

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::viewer::move_cells;

/// The count copied, none when the marks are not in order.
pub fn copy_marked(race: &Race) -> Result<Option<u32>, Refused> {
    let shared = &race.shared;
    let v = race.viewer_buffers();
    // Converted: `rc2k_core::race::viewer::clip`.
    let first = v.mark_in.get();
    let Some((_, blocks)) = rc2k_core::race::viewer::clip(first, v.mark_out.get())? else {
        return Ok(None);
    };
    v.clip_blocks.set(blocks);
    let dwords = 2 * blocks as usize;
    let first = 2 * first as usize;
    move_cells(
        &v.clip_places,
        0,
        &shared.replay_file().index.block_places,
        first,
        dwords,
    )?;
    move_cells(&v.clip_cameras, 0, &v.block_cameras, first, dwords)?;
    Ok(Some(blocks))
}
