//! `input::start_device_effects` (0x409672): the six effects of the record in esi asked to
//! start (`input::set_effect` with ebx 2, eax 0 .. 5).
//!
//! In:  esi = the device's record
//! Out: eax 6, ebx 2, the rest as `set_effect` keeps them (all)
//!
//! See `input/effects.rs`. Two callers, both in `input::start_effects`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::input::types::InputDevice;

/// The six effects of the device `record` told `what` (the loop of both functions).
pub fn all_six(record: &InputDevice, what: u32) -> Result<(), Refused> {
    for effect in 0..6 {
        crate::input::set_effect::set_effect(record, effect, what, 0, 0)?;
    }
    Ok(())
}

/// The device `record`'s effects started.
pub fn start_device_effects(record: &InputDevice) -> Result<(), Refused> {
    all_six(record, 2)
}
