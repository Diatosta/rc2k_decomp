//! `input::request_device_setup` (0x409351): the first input device's record
//! (`g_inputDevices`[0]) given a controller and its settings, and the input thread asked to
//! give the device back and acquire it again (`g_inputCommand` 1, which
//! `input::device_command` takes with device 0): +0 eax (the controller), +0x20 ebx, +8
//! ebx or 2 for 0, +0x18 ecx, +0xC 0, +0x10 edx, +0x14 ebp.
//!
//! In:  eax = the controller, ebx = its sensitivity, ecx = its mode, edx, ebp (0 from the
//!      menus)
//! Out: every register kept (`pusha` .. `popa`)
//! Not DirectInput itself: the request is the input thread's to act on. Caller:
//! `menu::apply_controller`. Refuses (before anything) no first record (null pointer).

use crate::errors::BadInput;

/// The Rust version (see the header).
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
    // Converted: `rc2k_core::input::joystick::device_setup`.
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
