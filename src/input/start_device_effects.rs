//! `input::start_device_effects` (0x409672): the six effects of the record asked to
//! start (`input::set_effect` with 2, effects 0 .. 5).
//!
//! Takes: the device's record
//!
//! See `input/effects.rs`. Two callers, both in `input::start_effects`.
//!
//! Its arguments are what it reads.

use crate::errors::Refused;
use crate::input::store::ForceFeedback;
use crate::input::types::InputDevice;

/// The six effects of the device `record` told `what` (the loop of both functions).
pub fn all_six(feedback: &ForceFeedback, record: &InputDevice, what: u32) -> Result<(), Refused> {
    for effect in 0..6 {
        crate::input::set_effect::set_effect(feedback, record, effect, what, 0, 0)?;
    }
    Ok(())
}

/// The device `record`'s effects started.
pub fn start_device_effects(feedback: &ForceFeedback, record: &InputDevice) -> Result<(), Refused> {
    all_six(feedback, record, 2)
}
