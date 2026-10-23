//! `race::reset_car_motion` (0x43FB47): `race::settle_car`, `race::invert_car_matrix`,
//! then the work car's motion cleared and its angles kept as the start's.
//!
//! In:  nothing (the registers are passed on to the calls)
//! Out: eax = the work car's +0x68C, less one when not 0; the rest as the calls leave
//!      them
//!
//! See `race/car_starts.rs`. Callers: `race::start_car` and three more (0x429492,
//! 0x43768C, 0x43FB23). It begins with its call: private copy, both calls relocated,
//! manual patch. Shadow mode runs the original alone (`settle_car`'s game calls).
//! Self-test: a copy whose calls go to stand-ins, against the Rust version on the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::car_starts::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { reset_car_motion(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
