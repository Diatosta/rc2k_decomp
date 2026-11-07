//! `input::start_effects` (0x409659): both device records' effects asked to start
//! (`input::start_device_effects` on `g_inputRecord1`, then `g_inputRecord2`).
//!
//! See `input/effects.rs`. One caller, `input::begin_effects`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::input::owner::Input;
use crate::input::store::Devices;

/// Both device records.
pub fn records(devices: &Devices) -> &[crate::input::types::InputDevice; 2] {
    crate::input::devices::records(devices)
}

/// Both devices' effects started.
pub fn start_effects(input: &Input) -> Result<(), Refused> {
    for record in records(&input.devices) {
        crate::input::start_device_effects::start_device_effects(&input.force_feedback, record)?;
    }
    Ok(())
}
