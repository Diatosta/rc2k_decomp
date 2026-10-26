//! `marks::reset_mask_cursor` (0x4BE0A3): the masks' cursor at the work block, the runs at
//! their start, their count 0.
//!
//! In:  nothing
//! Out: every register kept
//!
//! See `marks/masks.rs`. One caller, `sub_4BDE9A`. Self-test: the original and the Rust
//! version, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::marks::masks::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::reset_mask_cursor", rva(0x4BE0A3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { reset_mask_cursor() };
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &cursor_regions(), &rust);
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
    let mut regions = cursor_regions();
    regions.push(at(WORK_RVA));
    let saved = capture(&regions);
    for work in [0u32, 0x1234_5678] {
        check_state(
            &mut t,
            &format!("work block {work:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(WORK_RVA, work);
                game_write(CURSOR_RVA, 0xAAAA_AAAAu32);
                game_write(RUNS_RVA, 0xAAAA_AAAAu32);
                game_write(RUN_COUNT_RVA, 0xAAAA_AAAAu32);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
