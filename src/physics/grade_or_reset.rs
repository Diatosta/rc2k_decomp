//! `physics::grade_or_reset` (0x43BA8E): with `dword_71A234` 2, `physics::reset_current_car`
//! (the work copy's grades reset); else `physics::rate_current_car` (its damage made into
//! grades and failures).
//!
//! In:  nothing
//! Out: as the branch taken leaves them; no flags read
//!
//! Caller: `physics::set_handling` (0x43EB5D), first.
//! Self-test: a copy of the original on a `physics::Mirror` (its calls to copies of the
//! two, and theirs to copies of `reset_car_grades` and `rate_damage`), both branches.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::ESI;
use crate::physics::mirror::Mirror;
use crate::physics::rate_damage::rate_damage_at;
use crate::physics::reset_car_grades::reset_grades_at;
use crate::screens::harness::Left;

pub const MODE: u32 = 0x71A234; // dword_71A234: 2 resets the grades
/// The Rust core on the variables `m` reaches.
pub unsafe fn grade_or_reset_at(m: &Mirror) -> Left {
    unsafe {
        if m.get(MODE) == 2 {
            let car = m.a(0x719FD8);
            reset_grades_at(car, m.a(0x71A00C), m.a(0x71A008)).with(ESI, car)
        } else {
            rate_damage_at(m)
        }
    }
}
