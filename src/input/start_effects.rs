//! `input::start_effects` (0x409659): both device records' effects asked to start
//! (`input::start_device_effects` on `g_inputRecord1`, then `g_inputRecord2`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `input/effects.rs`. One caller, `input::begin_effects`. Its calls through `part`.
//! Shadow mode runs the original alone. Self-test: a copy whose calls go to the noting
//! stand-ins, against the Rust version (`stop_effects` shares it).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::input::effects::{RECORD_1, RECORD_2, START_DEVICE, effects_entry};

/// Both records through `place` (the two functions' body).
pub fn both_records(r: &mut Regs, place: u32) -> Result<(), Refused> {
    let mut g = *r;
    for record in [RECORD_1, RECORD_2] {
        g.esi = game_addr(rva(record)) as u32;
        crate::input::effects::game_part(place, &mut g)?;
    }
    Ok(())
}

/// The Rust version.
pub fn start_effects(r: &mut Regs) -> Result<(), Refused> {
    both_records(r, START_DEVICE)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    effects_entry(r, start_effects)
}
