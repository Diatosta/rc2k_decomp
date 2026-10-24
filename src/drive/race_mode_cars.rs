//! `drive::race_mode_cars` (0x436758): by the race's mode `dword_62389C`: 2 runs
//! `drive::start_cars_behind`; 0, 1, 3, 4 and the others nothing (five separate `retn`s).
//!
//! In:  nothing
//! Out: as `start_cars_behind` leaves them (mode 2), else every register kept; no flags
//!      read (its callers return next, theirs go on with `popa` / `mov`)
//!
//! Callers: race init's loop (`sub_4608BB`), `sub_460835`, `sub_460B2F`.
//! Refuses what `start_cars_behind` refuses.
//! Shadow mode: as `start_cars_behind` (the original alone, in mode 2).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version with the
//! same, modes 0 .. 5.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_read, rva};

const MODE_RVA: usize = rva(0x62389C); // dword_62389C

/// The Rust version: `drive::start_cars_behind` in mode 2.
pub unsafe fn race_mode_cars() -> Result<(), Refused> {
    if unsafe { game_read::<u32>(MODE_RVA) } == 2 {
        unsafe { crate::drive::start_cars_behind::start_cars_behind() }?;
    }
    Ok(())
}
