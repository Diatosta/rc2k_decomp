//! `race::choose_arcade_opponents` (0x449B79): the arcade race's three opponents' kinds
//! (`dword_62385C`, `60`, `64`) drawn from the noise seed and then `math::next_random`:
//! usable kinds within five places of the first car's, not taken; each logged.
//!
//! In:  nothing
//! Out: (mode 2) eax = the last place difference, ebx = its place, ecx = as
//!      `index_of_kind`, edx = its kind, ebp = the first car's place; else nothing
//!
//! See `race/race_cars.rs`. One caller, `race::set_up_arcade_cars`. Refuses, before
//! anything, a first kind with fewer than three possible opponents (the original would
//! draw for ever). Shadow mode runs the original alone (it logs). Self-test: a copy whose
//! log calls note themselves (the draws and the kind tests the real ones, from the same
//! states), against the Rust version on the same: first kinds with many and with just
//! three opponents, seeds by lot, mode not 2; the refusal on the Rust version alone.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::race_cars::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { choose_arcade_opponents(r, &mut cars_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
