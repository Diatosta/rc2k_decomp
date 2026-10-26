//! `race::copy_marked` (0x41E2F6): the marked blocks (from `dword_96E478` up to
//! `dword_96E47C`) onto the clipboard: their count into `dword_96E484`, their places
//! (`lDistanceToMove`) into `unk_96E488` and their cameras (`unk_963BB4`) into
//! `unk_971CC8`. Nothing when the marks are not in order (`jle`, signed).
//!
//! In:  nothing
//! Out: eax the first mark, ebx the second; copied: edx the count, ecx 0, esi and edi past
//!      the cameras' copy; no flags read
//!
//! Callers: `race::viewer_action` (action 0x1B, and in 0x11 before the cut). Refused, before
//! anything is written: a first mark below 0 or a second past 0x708 (the original copies from
//! outside the tables).

use crate::errors::Refused;
use crate::race::replay_file_store::REPLAY_FILE;
use crate::race::viewer::move_dwords;

/// The Rust version: the count copied (edx), none when the marks are not in order. Of
/// what the original leaves, its caller `viewer_action` reads only edx, in the cut.
pub fn copy_marked() -> Result<Option<u32>, Refused> {
    // Converted: `rc2k_core::race::viewer::clip`.
    let first = crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .mark_in
        .get();
    let Some((_, blocks)) = rc2k_core::race::viewer::clip(
        first,
        crate::race::viewer_buffers_store::VIEWER_BUFFERS
            .mark_out
            .get(),
    )?
    else {
        return Ok(None);
    };
    crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .clip_blocks
        .set(blocks);
    let dwords = 2 * blocks as usize;
    move_dwords(
        crate::race::viewer_buffers_store::VIEWER_BUFFERS.clip_places[0].addr() as usize,
        (REPLAY_FILE.block_places[0].addr() + (8 * first as usize) as u32) as usize,
        dwords,
    )?;
    move_dwords(
        crate::race::viewer_buffers_store::VIEWER_BUFFERS.clip_cameras[0].addr() as usize,
        (crate::race::viewer_buffers_store::VIEWER_BUFFERS
            .block_cameras
            .addr()
            + (8 * first as usize) as u32) as usize,
        dwords,
    )?;
    Ok(Some(blocks))
}
