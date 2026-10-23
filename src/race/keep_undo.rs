//! `race::keep_undo` (0x41E438): the viewed replay kept for one undo before an edit: the
//! block count into `dword_9673F4`, the index's places (`lDistanceToMove`, 0x3840 bytes)
//! into `dword_9673F8`, the blocks' cameras (`unk_963BB4`) into `unk_96AC38`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//!
//! Callers: `race::viewer_action` (six places), `race::edit_camera_1`, `edit_camera_2`.
//! Shadow mode compares. Self-test: the original against the Rust version over the tables
//! as they are, the undo tables filled first; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::viewer::{
    BLOCKS, CAMERAS, PLACES, TABLE_LEN, UNDO_BLOCKS, UNDO_CAMERAS, UNDO_PLACES, get, put,
    regs_entry, table,
};

/// The Rust version.
pub fn keep_undo() {
    // Converted: `rc2k_core::race::viewer_flow::copy_tables`.
    put(UNDO_BLOCKS, get(BLOCKS));
    let n = TABLE_LEN / 4;
    unsafe {
        rc2k_core::race::viewer_flow::copy_tables(
            (table(UNDO_PLACES, n), table(UNDO_CAMERAS, n)),
            (table(PLACES, n), table(CAMERAS, n)),
        );
    }
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    keep_undo();
    Ok(*r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}
