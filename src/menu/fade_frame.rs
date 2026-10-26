//! `menu::fade_frame` (0x4A1E60): a frame of the inspection's fades: the picture copied to the locked surface, then in a 3D scene the fade's see-through quad (`screens::draw_fade_overlay`), the frame shown.
//!
//! In:  nothing
//! Out: carry = the frame could not be made; every register kept (`pusha` .. `popa`)
//!
//! The frame callback of `sub_4A1CFA` and `sub_4A1DA5` (handed to `screens::run_frame`). The steps are the game's
//! (the surface, the 3D scene and the showing are the platform's), through an `Env`
//! (`menu::plain_frame`). A private copy (a call at +1). Shadow mode runs the original
//! alone (a frame cannot be shown twice). Self-test: `plain_frame::test_steps`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, rva};
use crate::menu::plain_frame::*;
use crate::menu::screen_loop::Real;

const FADE_OVERLAY: usize = rva(0x49FC7F); // screens::draw_fade_overlay

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

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let made = unsafe { run_steps(r, &STEPS, &mut Real) };
    r.set_carry(!made);
    Ok(())
}
