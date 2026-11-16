//! The six effects of the record asked to
//! stop (`input::set_effect` with 4, effects 0 .. 5).
//!
//! See `input/effects.rs` and `start_device_effects` (the same loop with 2).

use crate::errors::Refused;
use crate::input::start_device_effects::all_six;
use crate::input::store::ForceFeedback;
use crate::input::types::InputDevice;

/// The device `record`'s effects stopped.
pub fn stop_device_effects(feedback: &ForceFeedback, record: &InputDevice) -> Result<(), Refused> {
    all_six(feedback, record, 4)
}
