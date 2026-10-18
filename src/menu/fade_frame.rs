//! `menu::fade_frame` (0x4A1E60): a frame of the inspection's fades: the picture copied to the locked surface, then in a 3D scene the fade's see-through quad (`screens::draw_fade_overlay`), the frame shown.
//!
//! In:  nothing
//! Out: carry = the frame could not be made; every register kept (`pusha` .. `popa`)
//!
//! The frame callback of `sub_4A1CFA` and `sub_4A1DA5` (handed to `screens::run_frame`). The steps are the game's
//! (the surface, the 3D scene and the showing are the platform's), through an `Env`
//! (`menu::plain_frame`). A private copy (a call at +1). Shadow mode runs the original
//! alone (a frame cannot be shown twice). Self-test: `plain_frame::test_steps`.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::menu::plain_frame::*;
use crate::menu::screen_loop::Real;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::fade_frame", rva(0x4A1E60), thunk, &ORIGINAL)
    .private_copy(0x31)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FADE_OVERLAY: usize = rva(0x49FC7F); // screens::draw_fade_overlay

const CALLS: [usize; 7] = [0x1, 0x8, 0xD, 0x12, 0x19, 0x1E, 0x23];

/// The steps (see the header).
pub const STEPS: [Step; 7] = [
    (LOCK_SURFACE, true),
    (COPY_PICTURE, false),
    (UNLOCK_SURFACE, false),
    (BEGIN_SCENE, true),
    (FADE_OVERLAY, false),
    (END_SCENE, false),
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
        0x4A1E60,
        0x31,
        &[0x60, 0xE8, 0x47, 0x3A, 0xF6, 0xFF],
        &CALLS,
        &STEPS,
    )
}
