//! `input::start_effects` (0x409659): both device records' effects asked to start
//! (`input::start_device_effects` on `g_inputRecord1`, then `g_inputRecord2`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `input/effects.rs`. One caller, `input::begin_effects`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;

/// Both device records.
pub fn records() -> &'static [crate::input::types::InputDevice; 2] {
    crate::input::devices::records()
}

/// Both devices' effects started.
pub fn start_effects() -> Result<(), Refused> {
    for record in records() {
        crate::input::start_device_effects::start_device_effects(record)?;
    }
    Ok(())
}
