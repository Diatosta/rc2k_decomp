//! `service::edit_tyre` (0x46BD44): the keys while the tyre is being edited
//! (edit page 2): step down, step up, confirm.
//!
//! In:  ebx = 1 to confirm without taking a key
//! Out: every register preserved
//! What confirming does, and what is refused, is in `pages.rs` (`change_page`).
//! Manual patch with a private copy: a short jump or a call sits in the first bytes. The
//! copy's calls are relocated.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_keyed_state;
use crate::service::pages::{
    TYRE, change_page, change_regions, check_change_page, pages_unavailable,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("service::edit_tyre", rva(0x46BD44), thunk, &ORIGINAL)
    .private_copy(0xF9)
    .relocate_calls(&[0x6, 0x21, 0x2A, 0x56, 0x84, 0xA0, 0xCD, 0xE8, 0xED, 0xF2])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn edit_tyre(confirm: bool) -> Result<(), Refused> {
    unsafe { change_page(TYRE, confirm) }
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { edit_tyre(r.ebx == 1) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_keyed_state(&HOOK, r, call_original, &change_regions(TYRE), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = pages_unavailable() {
        return t.skip(why);
    }
    check_change_page(&mut t, call_original, TYRE, &rust);
    t.finish()
}
