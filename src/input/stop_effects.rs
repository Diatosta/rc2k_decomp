//! Both device records' effects asked to stop. See `input/effects.rs`.

use crate::errors::Refused;
use crate::input::owner::Input;

/// Both devices' effects stopped.
pub fn stop_effects(input: &Input) -> Result<(), Refused> {
    for record in crate::input::start_effects::records(&input.devices) {
        crate::input::stop_device_effects::stop_device_effects(&input.force_feedback, record)?;
    }
    Ok(())
}
