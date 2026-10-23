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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::split_by_mode::split_by_mode_in;
use crate::physics::splits::{CHECKS, COUNT, DONE, FIGURE, SPLIT, SPLITS, g, guarded, p};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::progress::{FINISH_NOTICE, finished_at, split_due};

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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
