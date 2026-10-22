//! `physics::take_split_counted` (0x436833, race mode 2): for car 0 only, when a
//! checkpoint is past the splits taken (+0xAC below +0x90, unsigned):
//! `physics::keep_split_time` (inside `pusha` / `popa`), `physics::set_flag_51659c`,
//! `physics::split_by_mode`, the figure kept at +0x140 [split]; before the last split
//! the notice 0xC9 (+0x730 0x32, +0x72C 300) and the count-down +0x8C up by
//! `dword_601434[split]` x 100; +0xA4 0; the splits taken made +0x90; at
//! `g_splitColourCount` finished: +0xA8 1, the notice 0xC8 (+0x72C -1, +0x730 0).
//!
//! In:  nothing
//! Out: eax, ebx and what the calls leave; no flags read
//!
//! Caller: `physics::race_timers`.
//! Refuses (before anything) a split past 0x3F, and what the calls refuse.
//! Self-test: `physics/splits.rs` (`test_splits`), mode 2.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::keep_split_time::keep_split_time;
use crate::physics::set_flag_51659c::set_flag_51659c;
use crate::physics::split_by_mode::split_by_mode_in;
use crate::physics::splits::{
    CHECKS, COUNT, DONE, FIGURE, SPLIT, SPLITS, g, guarded, p, test_splits, written,
};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use rc2k_core::physics::progress::{
    FINISH_NOTICE, SPLIT_NOTICE, finished_at, split_bonus, split_due,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::take_split_counted",
    rva(0x436833),
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

/// The steps on `r`.
// Converted: `rc2k_core::physics::progress::{split_due, finished_at}`.
pub fn take_split_counted_in(r: &mut Regs) -> Result<(), Refused> {
    if g(0x71B3F8) != 0 {
        return Ok(());
    }
    r.ebx = g(SPLIT);
    if !split_due(r.ebx, g(CHECKS))? {
        return Ok(());
    }
    unsafe { keep_split_time() }?;
    unsafe { set_flag_51659c() }?.apply(r);
    split_by_mode_in(r)?;
    r.ebx = g(SPLIT);
    r.eax = g(FIGURE);
    p(SPLITS + 4 * r.ebx as usize, r.eax);
    r.eax = g(CHECKS);
    if r.eax < g(COUNT) {
        p(0x71A6F4, SPLIT_NOTICE);
        p(0x71A708, 0x32);
        p(0x71A704, 300);
        r.ebx = g(SPLIT);
        r.eax = split_bonus(g(0x601434 + 4 * r.ebx as usize));
        p(0x71A064, g(0x71A064).wrapping_add(r.eax));
    }
    p(0x71A07C, 0);
    r.eax = g(CHECKS);
    p(SPLIT, r.eax);
    if finished_at(g(SPLIT), g(COUNT)) {
        p(DONE, 1);
        p(0x71A6F4, FINISH_NOTICE);
        p(0x71A704, u32::MAX);
        p(0x71A708, 0);
    }
    Ok(())
}

/// The Rust version.
pub unsafe fn take_split_counted(entry: &Regs) -> Result<Left, Refused> {
    guarded(entry, &take_split_counted_in)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { take_split_counted(r) }
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
    test_splits(&mut t, call_original, &rust, &|c| c.mode == 2);
    t.finish()
}
