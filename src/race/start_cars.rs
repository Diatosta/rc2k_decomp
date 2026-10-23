//! `race::start_cars` (0x446463): race init's first step: each car of the race
//! (`g_raceCars`) made current, loaded into the work copy (`drive::load_car`), set at
//! its start (`race::start_car`), the work car's +0xA3C (8 dwords) and +0xA5C (0x11)
//! cleared, stored back (`drive::store_car`).
//!
//! In:  nothing
//! Out: ecx = the count; the rest preserved (`pusha` / `popa` round each car)
//!
//! See `race/car_starts.rs`. One caller, `sub_4462B9` (race init). Refuses no cars (the
//! original's `jnz` would run on) or more than 8, before anything. Shadow mode runs the
//! original alone (`settle_car`'s game calls). Self-test: a copy whose calls go to
//! stand-ins (the car's loading and storing among them: nothing copied), against the Rust
//! version on the same; 1, 2, 5 and 8 cars.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::car_starts::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { start_cars(r) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
