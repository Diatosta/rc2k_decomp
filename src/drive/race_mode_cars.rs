//! `drive::race_mode_cars` (0x436758): by the race's mode `dword_62389C`: 2 runs
//! `drive::start_cars_behind`; 0, 1, 3, 4 and the others nothing (five separate `retn`s).
//!
//! In:  nothing
//! Out: as `start_cars_behind` leaves them (mode 2), else every register kept; no flags
//!      read (its callers return next, theirs go on with `popa` / `mov`)
//!
//! Callers: race init's loop (`sub_4608BB`), `sub_460835`, `sub_460B2F`. Refuses what
//! `start_cars_behind` refuses.

use crate::errors::Refused;
use crate::race::owner::race;

/// The Rust version: `drive::start_cars_behind` in mode 2.
pub fn race_mode_cars() -> Result<(), Refused> {
    let course = &race().course;
    if crate::race::owner::race().shared.vars.run_kind.bits() == 2 {
        { crate::drive::start_cars_behind::start_cars_behind(course) }?;
    }
    Ok(())
}
