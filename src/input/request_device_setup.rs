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
use crate::field;
use crate::input::devices::put;
use crate::input::input_tick::COMMAND_RVA;
use crate::input::store::DeviceSlot;
use crate::input::types::InputDevice;
use crate::store::{game_read, game_write};

pub const DEVICES_RVA: DeviceSlot = DeviceSlot::at(0x4F6EE8); // g_inputDevices
/// The Rust version (see the header).
pub unsafe fn request_device_setup(
    controller: u32,
    sensitivity: u32,
    mode: u32,
    edx: u32,
    ebp: u32,
) -> Result<(), BadInput> {
    let record = unsafe { game_read::<u32>(DEVICES_RVA) };
    if record == 0 {
        return Err(BadInput("no first input device record"));
    }
    // Converted: `rc2k_core::input::joystick::device_setup`.
    let s = rc2k_core::input::joystick::device_setup(controller, sensitivity, mode, edx, ebp);
    put(record, field!(InputDevice, controller), s.controller);
    put(record, field!(InputDevice, sensitivity), s.sensitivity);
    put(record, field!(InputDevice, kind), s.kind);
    put(record, field!(InputDevice, axis_code), s.axis_code);
    put(record, field!(InputDevice, strength), s.strength);
    put(record, field!(InputDevice, dead_zone), s.dead_zone);
    put(record, field!(InputDevice, end_zone), s.end_zone);
    unsafe { game_write(COMMAND_RVA, 1u32) };
    Ok(())
}
