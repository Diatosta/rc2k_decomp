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

use crate::drive::read_replay_frame::{test_regions, test_replay};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, guard};
use crate::race::viewer::{
    BLOCKS, CAMERAS, OPEN, PLACES, Saved, TABLE_LEN, UNDO_BLOCKS, UNDO_CAMERAS, UNDO_PLACES,
    check_original, dispatch_regs, dword, fill, get, put, region, table,
};
use crate::service::harness::Region;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::undo_edit", rva(0x41E46B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the registers it leaves.
pub fn undo_edit(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::viewer_flow::{undo_count, copy_tables}`.
    use rc2k_core::race::viewer_flow::{copy_tables, undo_count};
    let mut r = *entry;
    let Some(count) = undo_count(get(UNDO_BLOCKS)) else {
        return Ok(r);
    };
    guard(&regions(), || {
        r.eax = count;
        put(BLOCKS, r.eax);
        let n = TABLE_LEN / 4;
        unsafe {
            copy_tables(
                (table(PLACES, n), table(CAMERAS, n)),
                (table(UNDO_PLACES, n), table(UNDO_CAMERAS, n)),
            );
        }
        r.ecx = 0;
        r.esi = a(UNDO_CAMERAS) + TABLE_LEN as u32;
        r.edi = a(CAMERAS) + TABLE_LEN as u32;
        put(UNDO_BLOCKS, 0);
        crate::race::reset_marks::reset_marks()?;
        Ok(r)
    })
}

pub fn regions() -> Vec<Region> {
    let mut all = crate::race::reset_marks::regions();
    all.extend([
        dword(BLOCKS),
        dword(UNDO_BLOCKS),
        region(PLACES, TABLE_LEN),
        region(CAMERAS, TABLE_LEN),
    ]);
    all
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_regs(&HOOK, r, call_original, &regions(), 0, &undo_edit);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = test_regions();
    all.extend(regions());
    all.extend(crate::race::keep_undo::regions());
    let saved = Saved::new(&all);
    let Some(file) = test_replay() else {
        drop(saved);
        return t.skip("no test file in the temp folder");
    };
    // The test replay's two places, as the undo would have kept them.
    let places = crate::service::harness::capture(&[region(PLACES, 16)]);
    for kept in [0u32, 2] {
        check_original(
            &mut t,
            &format!("{kept} blocks kept"),
            call_original,
            Regs::SENTINEL,
            &regions(),
            0,
            &|| {
                put(OPEN, 1);
                put(BLOCKS, 5);
                put(crate::race::viewer::BLOCK, 1);
                put(UNDO_BLOCKS, kept);
                fill(UNDO_PLACES, TABLE_LEN, 8);
                crate::service::harness::restore(&[region(UNDO_PLACES, 16)], &places);
                fill(UNDO_CAMERAS, TABLE_LEN, 9);
                fill(CAMERAS, TABLE_LEN, 10);
                file.rewind();
            },
            &undo_edit,
        );
    }
    drop(file);
    drop(saved);
    t.finish()
}
