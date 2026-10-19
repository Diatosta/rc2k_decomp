//! `service::begin_tyre_edit` (0x46B6BD): the item action that opens the tyre edit page, if the
//! item is chosen or its time fits (see `service/begin_edits.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! In the service area's menu table. Self-test: `begin_edits::test_edit`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::service::begin_edits::{TYRE, dispatch_edit, test_edit};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("service::begin_tyre_edit", rva(0x46B6BD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_edit(&HOOK, r, call_original, TYRE);
}

fn self_test() -> (usize, usize) {
    test_edit(&HOOK, call_original, TYRE)
}
