//! `menu::picture_frame` (0x46F3FA): a frame of a still picture: the surface locked, the picture copied to it, unlocked, the frame shown.
//!
//! In:  nothing
//! Out: carry = the frame could not be made; every register kept (`pusha` .. `popa`)
//!
//! The frame callback of `sub_47F314` and `sub_47F3C7` (handed to `screens::run_frame`). The steps are the game's
//! (the surface, the 3D scene and the showing are the platform's), through an `Env`
//! (`menu::plain_frame`). A private copy (a call at +1). Shadow mode runs the original
//! alone (a frame cannot be shown twice). Self-test: `plain_frame::test_steps`.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::menu::plain_frame::*;
use crate::menu::screen_loop::Real;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::picture_frame", rva(0x46F3FA), thunk, &ORIGINAL)
    .private_copy(0x20)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CALLS: [usize; 4] = [0x1, 0x8, 0xD, 0x12];

/// The steps (see the header).
pub const STEPS: [Step; 4] = [
    (LOCK_SURFACE, true),
    (COPY_PICTURE, false),
    (UNLOCK_SURFACE, false),
    (SHOW_FRAME, true),
];

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let made = unsafe { run_steps(r, &STEPS, &mut Real) };
    r.set_carry(!made);
}

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_steps(
        t,
        0x46F3FA,
        0x20,
        &[0x60, 0xE8, 0xAD, 0x64, 0xF9, 0xFF],
        &CALLS,
        &STEPS,
    )
}
