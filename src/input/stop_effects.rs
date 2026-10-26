//! `input::stop_effects` (0x409685): both device records' effects asked to stop
//! (`input::stop_device_effects` on `g_inputRecord1`, then `g_inputRecord2`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `input/effects.rs` and `start_effects`. One caller, `input::end_effects`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;

/// Both devices' effects stopped.
pub fn stop_effects() -> Result<(), Refused> {
    for record in crate::input::start_effects::records() {
        crate::input::stop_device_effects::stop_device_effects(record)?;
    }
    Ok(())
}
