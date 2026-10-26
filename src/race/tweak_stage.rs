//! `race::tweak_stage` (0x439AAB): the three dwords at 0x60A564 0xFF for stages 0x15, 0x36 and 0x3E.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/stage_prefs.rs`. Self-test: the original and the Rust version on the game's
//! variables with values of the test's, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::stage_prefs::*;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::tweak_stage", rva(0x439AAB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    vec![(game_addr(rva(0x60A564)) as u32, 12), at(STAGE_RVA)]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { tweak_stage() };
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
    let _ = (at as fn(usize) -> Region, game_addr as fn(usize) -> usize);
    for stage in [0x15u32, 0x36, 0x3E, 0x16, 0] {
        check_state(
            &mut t,
            &format!("stage {stage:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(rva(0x60A564), [0x11u32, 0x22, 0x33]);
                game_write(STAGE_RVA, stage);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
