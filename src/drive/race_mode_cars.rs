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

use crate::drive::start_cars_behind::{game_part, start_cars_behind};
use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};
use crate::race::car_starts::Part;

const MODE_RVA: usize = rva(0x62389C); // dword_62389C

/// The Rust core, `start_cars_behind` through `part`.
pub unsafe fn race_mode_cars(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    if unsafe { game_read::<u32>(MODE_RVA) } == 2 {
        let mut r = *entry;
        part(0, &mut r)?;
        return Ok(r);
    }
    Ok(*entry)
}

fn game(_: u32, r: &mut Regs) -> Result<(), Refused> {
    *r = unsafe { start_cars_behind(r, &mut game_part) }?;
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let g = unsafe { race_mode_cars(r, &mut game) }?;
    *r = Regs {
        esp: r.esp,
        eflags: r.eflags,
        ..g
    };
    Ok(())
}
