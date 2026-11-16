//! What the DirectInput functions share (`acquire_device`, `list_joysticks`,
//! `joystick_found`, `axis_found`, `begin_force`, `end_force`, `effect_found`,
//! `make_effect`, `effect_objects`): the game's variables they use.

use crate::errors::FatalError;
use crate::input::store::Devices;

pub const CAPS_LEN: usize = std::mem::size_of::<crate::input::types::DiDevCaps>();
pub const CONTROLLERS: u32 = rc2k_core::input::joystick::CONTROLLERS;

/// The controller's entry for a record; a number past the table is refused.
pub fn entry_of<'a>(
    devices: &'a Devices,
    record: &crate::input::types::InputDevice,
) -> Result<&'a crate::input::types::Controller, crate::errors::BadInput> {
    devices
        .controllers
        .get(record.controller.get() as usize)
        .ok_or(crate::errors::BadInput(
            "a controller number past the table's 16",
        ))
}

/// The game's fatal error `code` with no message.
pub const fn fatal(code: u32) -> FatalError {
    FatalError { code, message: "" }
}
