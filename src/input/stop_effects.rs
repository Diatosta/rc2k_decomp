//! `input::stop_effects` (0x409685): both device records' effects asked to stop
//! (`input::stop_device_effects` on `g_inputRecord1`, then `g_inputRecord2`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `input/effects.rs` and `start_effects`. One caller, `input::end_effects`. Its calls
//! through `part`. Shadow mode runs the original alone. Self-test: a copy whose calls go
//! to the noting stand-ins, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::input::effects::{STOP_DEVICE, effects_entry};
use crate::input::start_effects::both_records;

/// The Rust version.
pub fn stop_effects(r: &mut Regs) -> Result<(), Refused> {
    both_records(r, STOP_DEVICE)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    effects_entry(r, stop_effects)
}
