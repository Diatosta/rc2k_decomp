//! `service::edit_ride_height` (0x46BBA2): the keys while the ride height is being edited
//! (edit page 5): step down, step up, confirm.
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
    RIDE_HEIGHT, change_page, change_regions, check_change_page, pages_unavailable,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("service::edit_ride_height", rva(0x46BBA2), thunk, &ORIGINAL)
        .private_copy(0x136)
        .relocate_calls(&[
            0x6, 0x21, 0x2A, 0x6F, 0x93, 0xC1, 0xDD, 0x10A, 0x125, 0x12A, 0x12F,
        ])
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn edit_ride_height(confirm: bool) -> Result<(), Refused> {
    unsafe { change_page(RIDE_HEIGHT, confirm) }
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { edit_ride_height(r.ebx == 1) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_keyed_state(&HOOK, r, call_original, &change_regions(RIDE_HEIGHT), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = pages_unavailable() {
        return t.skip(why);
    }
    check_change_page(&mut t, call_original, RIDE_HEIGHT, &rust);
    t.finish()
}
