//! `race::grade_car_values` (0x43FE24): car block esi's 34 values graded: each grade the
//! number of its thresholds (50, 75, 90, 100 or fewer; see `GRADES`) below the value,
//! tried in order by `fild` / `fcomp` / `fnstsw` (C0: a NaN passes them all).
//!
//! In:  esi = the car block
//! Out: eax = the last grade; the rest preserved; `dword_62400C` the last threshold tried
//!
//! See `race/car_grades.rs`. One caller, `race::grade_all_cars` (its flags are not read
//! there: `pop` / `inc` / `cmp` follow). Shadow mode: the Rust version, the grades and
//! `dword_62400C` put back, the original's, compared. Self-test: the game's first block's
//! values set to floats round the thresholds (NaNs, infinities, a denormal, negatives),
//! sixteen ways, both versions, put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs};
use crate::race::car_grades::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { grade_car_values(r) };
    Ok(Left::nothing().with(EAX, g.eax))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
