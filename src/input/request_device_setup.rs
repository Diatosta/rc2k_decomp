//! The first input device's record is given a controller and its settings, and the input thread
//! is asked to give the device back and acquire it again (command 1, device 0): `controller` the
//! given controller, `sensitivity` the given sensitivity, `kind` the sensitivity (2 for 0),
//! `axis_code` the given axes' code, `strength` 0, `dead_zone` and `end_zone` the given zones.
//! Not DirectInput itself: the request is the input thread's to act on. Refuses (before
//! anything) no first record.

use crate::errors::BadInput;

/// See the header.
pub fn request_device_setup(
    controller: u32,
    sensitivity: u32,
    mode: u32,
    edx: u32,
    ebp: u32,
    input: &crate::input::owner::Input,
) -> Result<(), BadInput> {
    let Some(record) = input.devices.device(0) else {
        return Err(BadInput("no first input device record"));
    };
    let s = rc2k_core::input::joystick::device_setup(controller, sensitivity, mode, edx, ebp);
    record.controller.set(s.controller);
    record.sensitivity.set(s.sensitivity);
    record.kind.set(s.kind);
    record.axis_code.set(s.axis_code);
    record.strength.set(s.strength);
    record.dead_zone.set(s.dead_zone);
    record.end_zone.set(s.end_zone);
    input.devices.input_command.set(1);
    Ok(())
}
