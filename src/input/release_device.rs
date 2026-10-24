//! `input::release_device` (0x4087F3): a device given back: with a DirectInput device (+4),
//! `Unacquire`, its effects stopped (`sub_4095AE`) when its controller's entry
//! (`dword_4F78E4` + 0x50 its number, +0x1C) is 1, `Release`, +4 0; then +0x24 (acquired)
//! and +0x28 (effects on) 0 and its readings cleared (`input::clear_device_readings`).
//!
//! In:  esi = the device's record
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `input::device_command` (three times), the input thread's end. Refuses (before anything)
//! a controller number past 16 with
//! a device.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::field;
use crate::input::clear_device_readings::clear_device_readings;
use crate::input::device_effects::controller;
use crate::input::devices::{at, com_call, put};
use crate::input::types::InputDevice;

/// The device `record` given back.
pub unsafe fn release_device(record: u32) -> Result<(), Refused> {
    let device = at(record, field!(InputDevice, device));
    if device != 0 {
        let effects = controller(record, 0x1C)? == 1;
        unsafe { com_call(device, 0x20, &[]) };
        if effects {
            crate::input::end_force::end_force(record);
        }
        unsafe { com_call(device, 0x08, &[]) };
        put(record, field!(InputDevice, device), 0);
    }
    put(record, field!(InputDevice, acquired), 0);
    put(record, field!(InputDevice, effects_on), 0);
    clear_device_readings(record);
    Ok(())
}

/// The Rust version for Rust callers.
pub unsafe fn release_device_in_game(record: u32) -> Result<(), Refused> {
    unsafe { release_device(record) }
}
