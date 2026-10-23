//! `input::stop_device_effects` (0x40969E): the six effects of the record in esi asked to
//! stop (`input::set_effect` with ebx 4, eax 0 .. 5).
//!
//! In:  esi = the device's record
//! Out: eax 6, ebx 4, the rest as `set_effect` keeps them (all)
//!
//! See `input/effects.rs` and `start_device_effects` (the same loop with 2). Two callers,
//! both in `input::stop_effects`. Its calls through `part`. Shadow mode runs the original
//! alone. Self-test: a copy whose calls go to the noting stand-ins, against the Rust
//! version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::input::effects::effects_entry;
use crate::input::start_device_effects::all_six;

/// The Rust version.
pub fn stop_device_effects(r: &mut Regs) -> Result<(), Refused> {
    all_six(r, 4)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    effects_entry(r, stop_device_effects)
}
