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
use crate::race::vars_store::RACE_VARS;

/// The Rust version: `drive::start_cars_behind` in mode 2.
pub unsafe fn race_mode_cars() -> Result<(), Refused> {
    if RACE_VARS.run_kind.bits() == 2 {
        unsafe { crate::drive::start_cars_behind::start_cars_behind() }?;
    }
    Ok(())
}
