//! Both device records' effects asked to start. See `input/effects.rs`.

use crate::errors::Refused;
use crate::input::owner::Input;
use crate::input::store::Devices;

/// The device records.
pub fn records(devices: &Devices) -> &[crate::input::types::InputDevice; 4] {
    crate::input::devices::records(devices)
}

/// Both devices' effects started.
pub(super) fn start_effects(input: &Input) -> Result<(), Refused> {
    for record in records(&input.devices) {
        crate::input::start_device_effects::start_device_effects(&input.force_feedback, record)?;
    }
    Ok(())
}
