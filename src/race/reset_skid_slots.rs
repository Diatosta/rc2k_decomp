//! `race::reset_skid_slots` (0x4585BA): a skid object's four slots (+0xF48 ..) -1.
//!
//! In:  esi = the object
//! Out: every register preserved
//!
//! See `race/stage_prefs.rs`. Self-test: the original and the Rust version on the game's
//! variables with values of the test's, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::stage_prefs::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::reset_skid_slots", rva(0x4585BA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    skid_regions()[..1].to_vec()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { reset_skid_slots(r.esi) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
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
        "the first object",
        call_original,
        Regs {
            esi: regions[0].0 - 0xF48,
            ..Regs::SENTINEL
        },
        &|| unsafe {
            std::ptr::write_bytes(regions[0].0 as *mut u8, 0x5A, 16);
        },
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
