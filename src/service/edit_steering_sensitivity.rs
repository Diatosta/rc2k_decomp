//! `service::edit_steering_sensitivity` (0x46BD0E): the keys while the steering sensitivity is being
//! edited (edit page 3): step down, step up, and confirm, which leaves edit mode.
//!
//! In:  nothing
//! Out: every register preserved
//! The value is changed in the player as the keys come. Refuses a missing player.
//! Manual patch with a private copy: a short jump or a call sits in the first bytes. The
//! copy's calls are relocated.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_keyed_state;
use crate::service::pages::{
    STEERING_SENSITIVITY, check_simple_page, pages_unavailable, simple_page, simple_regions,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "service::edit_steering_sensitivity",
    rva(0x46BD0E),
    thunk,
    &ORIGINAL,
)
.private_copy(0x36)
.relocate_calls(&[0x1, 0x18, 0x21])
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn edit_steering_sensitivity() -> Result<(), Refused> {
    unsafe { simple_page(STEERING_SENSITIVITY) }
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { edit_steering_sensitivity() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_keyed_state(
        &HOOK,
        r,
        call_original,
        &simple_regions(STEERING_SENSITIVITY),
        &rust,
    );
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = pages_unavailable() {
        return t.skip(why);
    }
    check_simple_page(&mut t, call_original, STEERING_SENSITIVITY, &rust);
    t.finish()
}
