//! `race::auto_set_up_car` (0x436F08): car block esi's set-up made for the stage (the
//! table `dword_6043FC` by `g_stageNumber`), with its log ("autosetup", "t", "off", "h",
//! "s", "g", "g" and the ranges).
//!
//! In:  esi = the car block
//! Out: eax = the last value (the stage when it has no set-up), ebx = 10, edx = the
//!      remainder, ebp = the set-up's address; the rest preserved
//!
//! See `race/race_cars.rs`. Callers: `race::set_up_mode_0_cars`, `sub_43613A` (mode 2).
//! Refuses stage 0x14 (the original's index wraps: a read far outside; section 16),
//! before anything. Shadow mode runs the original alone (it logs). Self-test: a copy whose
//! log calls note themselves, on the game's first block (filled, put back), stages round
//! the table's ends; the refusal on the Rust version alone.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::race_cars::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { auto_set_up_car(r, &mut cars_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
