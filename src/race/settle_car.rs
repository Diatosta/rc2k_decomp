//! `race::settle_car` (0x43FC32): the work car settled on the ground: pitch and roll
//! found by halving steps against the wheels' heights, then the car lowered onto them.
//!
//! In:  nothing (the registers are passed on to the calls)
//! Out: eax = wheel 3's height, edx = the lowest; the rest as the calls leave them
//!
//! See `race/car_starts.rs`. One caller, `race::reset_car_motion`. It calls
//! `physics::set_handling` (0x43EB5D) and `physics::probe_ground` (0x45D305), stream B's
//! Rust versions, and `race::get_mat_crash`. It begins with its call: private copy, five calls relocated,
//! manual patch. Shadow mode runs the original alone (the calls change more than can be
//! put back). Self-test: a copy whose calls go to stand-ins that note them and
//! give wheel heights of the test's, against the Rust version on the same stand-ins.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::car_starts::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { settle_car(r) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
