//! `scene::restart_clock` (0x450381): `g_sceneClock` = the tick count.
//!
//! In:  nothing
//! Out: eax = the tick count; every other register preserved
//!
//! 21 callers (`race::load_stage` and the race's code).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{Regs, game_write, rva};

pub const CLOCK_RVA: usize = rva(0x613830); // g_sceneClock

/// The clock set to `now`; eax.
pub fn restart_clock(now: u32) -> u32 {
    unsafe { game_write(CLOCK_RVA, now) };
    now
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    r.eax = restart_clock(crate::system::tick_count::tick_count());
    Ok(())
}
