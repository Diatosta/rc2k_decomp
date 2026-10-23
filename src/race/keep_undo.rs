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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::viewer::{
    BLOCKS, CAMERAS, PLACES, Saved, TABLE_LEN, UNDO_BLOCKS, UNDO_CAMERAS, UNDO_PLACES,
    check_original, dispatch_regs, dword, fill, get, put, region, regs_entry, table,
};
use crate::service::harness::Region;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::keep_undo", rva(0x41E438), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

pub fn regions() -> Vec<Region> {
    vec![
        dword(UNDO_BLOCKS),
        region(UNDO_PLACES, TABLE_LEN),
        region(UNDO_CAMERAS, TABLE_LEN),
    ]
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    keep_undo();
    Ok(*r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_regs(&HOOK, r, call_original, &regions(), 0, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved = Saved::new(&regions());
    check_original(
        &mut t,
        "the tables as they are",
        call_original,
        Regs::SENTINEL,
        &regions(),
        0,
        &|| {
            put(UNDO_BLOCKS, 0xAAAA_AAAA);
            fill(UNDO_PLACES, TABLE_LEN, 1);
            fill(UNDO_CAMERAS, TABLE_LEN, 2);
        },
        &rust,
    );
    drop(saved);
    t.finish()
}
