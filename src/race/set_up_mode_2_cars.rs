//! `race::set_up_mode_2_cars` (0x43613A): `sub_43603C`'s arcade set-up (mode 2): the
//! rally's car count, best and start places, one player and four cars, the arcade cars'
//! types and numbers from the noise seed (`dword_60519C`, shuffled), the race tables
//! cleared, four automatic set-ups, the player's +0x3C / +0x658 .. +0x66C; logged.
//!
//! In:  nothing
//! Out: eax = the cars behind (count - start place), ecx = the count; the rest as the
//!      steps leave them
//!
//! See `race/race_cars.rs`. One caller, `race::set_up_cars`. Refuses a car count of 0
//! or more than 89 (the original's `jnz` runs on / past `dword_604ECC`), before
//! anything. Shadow mode runs the original alone (it logs). Self-test: a copy whose
//! calls note themselves (`scene::next_noise` the real one, from the same seed), against
//! the Rust version on the same: each rally, start place by `dword_60101C` and the
//! stage, the players' numbers in the way; the refusal on the Rust version alone.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::race_cars::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { set_up_mode_2_cars(r, &mut cars_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
