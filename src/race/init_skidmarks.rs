//! `race::init_skidmarks` (0x45857D): the skid marks cleared (`skidmarks init`).
//!
//! In:  nothing
//! Out: esi the second skid object, edi past the marks, ecx 0
//!
//! See `race/stage_prefs.rs`. Self-test: the original and the Rust version on the game's
//! variables with values of the test's, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::stage_prefs::*;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::init_skidmarks", rva(0x45857D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    skid_regions()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { init_skidmarks(r) }))
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
    check_state(
        &mut t,
        "filled with 0x5A",
        call_original,
        Regs::SENTINEL,
        &|| unsafe {
            for &(a, n) in &regions {
                std::ptr::write_bytes(a as *mut u8, 0x5A, n);
            }
        },
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
