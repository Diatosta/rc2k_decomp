//! `race::undo_edit` (0x41E46B): the viewer's undo: with something kept
//! (`dword_9673F4` not 0) the block count, the places and the cameras back from the undo
//! tables (`race::keep_undo`'s), the undo emptied and the marks reset
//! (`race::reset_marks`).
//!
//! In:  nothing
//! Out: nothing kept: every register kept. Else eax the count, ecx 0, esi and edi past the
//!      cameras' copy; no flags read
//!
//! Callers: `race::viewer_action` (action 0x13, and 0x20). Refuses what `reset_marks`
//! refuses (everything put back). Shadow mode compares. Self-test: the original against
//! the Rust version with `read_replay_frame`'s test replay: nothing kept, two blocks
//! kept.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::race::viewer::{
    BLOCKS, CAMERAS, PLACES, TABLE_LEN, UNDO_BLOCKS, UNDO_CAMERAS, UNDO_PLACES, get, put, table,
};

/// The Rust version. What the original leaves its one caller, `viewer_action` (which
/// keeps every register), does not read.
pub fn undo_edit() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer_flow::{undo_count, copy_tables}`.
    use rc2k_core::race::viewer_flow::{copy_tables, undo_count};
    let Some(count) = undo_count(get(UNDO_BLOCKS)) else {
        return Ok(());
    };
    put(BLOCKS, count);
    let n = TABLE_LEN / 4;
    unsafe {
        copy_tables(
            (table(PLACES, n), table(CAMERAS, n)),
            (table(UNDO_PLACES, n), table(UNDO_CAMERAS, n)),
        );
    }
    put(UNDO_BLOCKS, 0);
    crate::race::reset_marks::reset_marks()
}
