//! `race::start_race` (0x4278B7): `dword_621870` 0, then `race::run_race`.
//!
//! In:  nothing
//! Out: the registers as `race::run_race` leaves them
//!
//! See `race/race_end.rs`. One caller, `race::race_from_menu` (a `retn` follows).
//! Shadow mode runs the original alone (it runs the race). Self-test: a copy whose call
//! goes to a stand-in, against the Rust version on the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::race_end::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { start_race(r, &mut start_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
