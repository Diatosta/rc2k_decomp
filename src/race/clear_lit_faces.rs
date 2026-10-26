//! `race::clear_lit_faces` (0x45E9D5): `g_litFaceCount` 0.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/course_data.rs`. One caller, `race::reset_face_lights`. Self-test: the
//! original and the Rust version on the game's count, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::course_data::*;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::clear_lit_faces", rva(0x45E9D5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    vec![at(LIT_COUNT_RVA)]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { clear_lit_faces() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    check_state(
        &mut t,
        "count 0x55",
        call_original,
        Regs::SENTINEL,
        &|| unsafe { game_write(LIT_COUNT_RVA, 0x55u32) },
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
