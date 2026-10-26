//! `physics::take_split` (0x436900, race modes other than 2): unless finished (+0xA8),
//! when a checkpoint is past the splits taken (+0xAC below +0x90, unsigned):
//! `physics::split_by_mode`, its figure (+0xA0C) kept at +0x140 [split], the splits taken
//! made +0x90; at `g_splitColourCount` the car is finished: +0xA8 1, +0xA0 0, the notice
//! 0xC8 (+0x72C -1, +0x730 0).
//!
//! In:  nothing
//! Out: eax, ebx and what `split_by_mode` leaves; no flags read
//!
//! Caller: `physics::race_timers`.
//! Refuses (before anything) a split past 0x3F, and what `split_by_mode` refuses.
//! Self-test: `physics/splits.rs` (`test_splits`).
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::split_by_mode::split_by_mode_in;
use crate::physics::splits::{
    CHECKS, COUNT, DONE, FIGURE, SPLIT, SPLITS, g, guarded, p, test_splits, written,
};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use rc2k_core::physics::progress::{FINISH_NOTICE, finished_at, split_due};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::take_split", rva(0x436900), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The steps on `r`.
// Converted: `rc2k_core::physics::progress::{split_due, finished_at}`.
pub fn take_split_in(r: &mut Regs) -> Result<(), Refused> {
    if g(DONE) != 0 {
        return Ok(());
    }
    r.ebx = g(SPLIT);
    if !split_due(r.ebx, g(CHECKS))? {
        return Ok(());
    }
    split_by_mode_in(r)?;
    r.ebx = g(SPLIT);
    r.eax = g(FIGURE);
    p(SPLITS + 4 * r.ebx as usize, r.eax);
    r.eax = g(CHECKS);
    p(SPLIT, r.eax);
    if finished_at(g(SPLIT), g(COUNT)) {
        p(DONE, 1);
        p(0x71A078, 0);
        p(0x71A6F4, FINISH_NOTICE);
        p(0x71A704, u32::MAX);
        p(0x71A708, 0);
    }
    Ok(())
}

/// The Rust version.
pub unsafe fn take_split(entry: &Regs) -> Result<Left, Refused> {
    guarded(entry, &take_split_in)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { take_split(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &written(), &rust);
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
    test_splits(&mut t, call_original, &rust, &|_| true);
    t.finish()
}
