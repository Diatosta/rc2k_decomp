//! `race::set_car_times` (0x4377D0): the stage's car times (arcade, mode 2): the first
//! car's kind factor (`flt_601C64`), the stage's difficulty and time percentage, the
//! extra time (`dword_601C50`) added to the first number, every number of
//! `g_stageCarNumbers` (`dword_601830` of them) times the factor; all logged.
//!
//! In:  nothing
//! Out: eax = the extra time, ebx = the count (without a stage, 0x4F or more: eax the
//!      stage); the rest preserved
//!
//! See `race/race_cars.rs`. One caller, `race::set_up_cars` (mode 2). Refuses, for a
//! stage below 0x4F, a count of 0 or more than 0x100 (the original's `jnz` runs on /
//! past the numbers), before anything. Shadow mode runs the original alone (it logs).
//! Self-test: a copy whose log calls note themselves, against the Rust version on the
//! same: stages in each rally, below 0x15 and past 0x4E, kinds round 0 and 0x16, counts
//! 1 and 0x100; the refusal on the Rust version alone.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::race_cars::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { set_car_times(r, &mut cars_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
