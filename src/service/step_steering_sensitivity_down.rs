//! `service::step_steering_sensitivity_down` (0x46BF15): steps the steering sensitivity, the current car's dword at +0x11C, one
//! down, stopping at 0.
//!
//! In:  nothing
//! Out: edi = the current player; everything else preserved
//! The exact rule at the limits is in `setup_values.rs` (`Stepper`).

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::service::setup_values::{
    Stepper, Target, check_stepper, dispatch_stepper, stepper_unavailable,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "service::step_steering_sensitivity_down",
    rva(0x46BF15),
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
    target: Target::CarField(0x11C),
    max: 100,
    up: false,
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
