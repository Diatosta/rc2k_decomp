//! `service::step_brake_balance_up` (0x46BF90): steps the brake balance, the current car's dword at +0x110, one
//! up, stopping at 100.
//!
//! In:  nothing
//! Out: edi = the current car object; everything else preserved
//! The exact rule at the limits is in `setup_values.rs` (`Stepper`).

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::service::setup_values::{
    Stepper, Target, check_stepper, dispatch_stepper, stepper_unavailable,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "service::step_brake_balance_up",
    rva(0x46BF90),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version: `setup_values::step` with this ----

pub const STEPPER: Stepper = Stepper {
    target: Target::CarField(0x110),
    max: 100,
    up: true,
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_stepper(&HOOK, r, call_original, STEPPER);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = stepper_unavailable(STEPPER) {
        return t.skip(why);
    }
    check_stepper(&mut t, call_original, STEPPER);
    t.finish()
}
