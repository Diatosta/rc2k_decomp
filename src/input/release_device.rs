//! A device given back: with a DirectInput `device`,
//! `Unacquire`, its effects stopped (`input::end_force`) when its controller's entry
//! (`controllers`) has `force_feedback` 1, `Release`, `device` 0; then `acquired`
//! and `effects_on` 0 and its readings cleared (`input::clear_device_readings`).
//!
//! Refuses (before anything) a controller number past 16 with a device.

use crate::errors::Refused;
use crate::input::clear_device_readings::clear_device_readings;
use crate::input::device_effects::controller;
use crate::input::store::Devices;
use crate::input::types::InputDevice;

/// The device `record` given back.
pub(super) fn release_device(devices: &Devices, record: &InputDevice) -> Result<(), Refused> {
    let device = record.device.get();
    if device != 0 {
        let effects = controller(devices, record)?.force_feedback.get() == 1;
        crate::input::di_objects::unacquire(device);
        if effects {
            crate::input::end_force::end_force(devices, record)?;
        }
        crate::input::di_objects::release(device);
        record.device.set(0);
    }
    record.acquired.set(0);
    record.effects_on.set(0);
    clear_device_readings(record);
    Ok(())
}
