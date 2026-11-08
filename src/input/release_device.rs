//! `input::release_device` (0x4087F3): a device given back: with a DirectInput device (+4),
//! `Unacquire`, its effects stopped (`input::end_force`) when its controller's entry
//! (`dword_4F78E4` + 0x50 its number, +0x1C) is 1, `Release`, +4 0; then +0x24 (acquired)
//! and +0x28 (effects on) 0 and its readings cleared (`input::clear_device_readings`).
//!
//! Takes: the device's record
//!
//! Callers: `input::device_command` (three times), the input thread's end. Refuses (before anything)
//! a controller number past 16 with
//! a device.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::input::clear_device_readings::clear_device_readings;
use crate::input::device_effects::controller;
use crate::input::store::Devices;
use crate::input::types::InputDevice;

/// The device `record` given back.
pub fn release_device(devices: &Devices, record: &InputDevice) -> Result<(), Refused> {
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

/// The Rust version for Rust callers.
pub fn release_device_in_game(devices: &Devices, record: &InputDevice) -> Result<(), Refused> {
    release_device(devices, record)
}
