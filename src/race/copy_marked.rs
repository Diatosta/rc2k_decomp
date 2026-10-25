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
use crate::physics::flow::a;
use crate::race::replay_file_store::REPLAY_FILE;
use crate::race::viewer::{
    CAMERAS, CLIP_BLOCKS, CLIP_CAMERAS, CLIP_PLACES, MARK_IN, MARK_OUT, get, move_dwords, put,
};

/// The Rust version: the count copied (edx), none when the marks are not in order. Of
/// what the original leaves, its caller `viewer_action` reads only edx, in the cut.
pub fn copy_marked() -> Result<Option<u32>, Refused> {
    // Converted: `rc2k_core::race::viewer::clip`.
    let first = get(MARK_IN);
    let Some((_, blocks)) = rc2k_core::race::viewer::clip(first, get(MARK_OUT))? else {
        return Ok(None);
    };
    put(CLIP_BLOCKS, blocks);
    let dwords = 2 * blocks as usize;
    move_dwords(
        a(CLIP_PLACES) as usize,
        REPLAY_FILE.block_places[0].addr() + 8 * first as usize,
        dwords,
    );
    move_dwords(
        a(CLIP_CAMERAS) as usize,
        a(CAMERAS) as usize + 8 * first as usize,
        dwords,
    );
    Ok(Some(blocks))
}
