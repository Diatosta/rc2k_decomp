//! `input::stop_device_effects` (0x40969E): the six effects of the record in esi asked to
//! stop (`input::set_effect` with ebx 4, eax 0 .. 5).
//!
//! In:  esi = the device's record
//! Out: eax 6, ebx 4, the rest as `set_effect` keeps them (all)
//!
//! See `input/effects.rs` and `start_device_effects` (the same loop with 2). Two callers,
//! both in `input::stop_effects`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::input::start_device_effects::all_six;
use crate::input::types::InputDevice;

/// The device `record`'s effects stopped.
pub fn stop_device_effects(record: &InputDevice) -> Result<(), Refused> {
    all_six(record, 4)
}
