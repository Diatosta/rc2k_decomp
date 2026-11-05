//! What the DirectInput functions share (`acquire_device`, `list_joysticks`,
//! `joystick_found`, `axis_found`, `begin_force`, `end_force`, `effect_found`,
//! `make_effect`, `effect_objects`): the game's variables they use.

use crate::errors::FatalError;
use crate::input::store::DEVICES;

pub const CAPS_LEN: usize = std::mem::size_of::<crate::input::types::DiDevCaps>();
pub const CONTROLLER_LEN: u32 = std::mem::size_of::<crate::input::types::Controller>() as u32;
pub const CONTROLLERS: u32 = rc2k_core::input::joystick::CONTROLLERS;

/// The controller's entry for a record (`g_controllers` + 0x50 its number). The original
/// does not check the number (`imul edi, [esi], 50h`); a number past the table is refused.
pub fn entry_of(
    record: &crate::input::types::InputDevice,
) -> Result<&'static crate::session::Bytes<{ CONTROLLER_LEN as usize }>, crate::errors::BadInput> {
    DEVICES
        .controllers
        .get(record.controller.get() as usize)
        .ok_or(crate::errors::BadInput(
            "a controller number past the table's 16",
        ))
}

/// A dword of the record's controller entry (offset by `field!`).
pub fn controller_dword(
    record: &crate::input::types::InputDevice,
    offset: u32,
) -> Result<u32, crate::errors::BadInput> {
    Ok(entry_of(record)?.get_u32(offset as usize))
}

/// The game's fatal error `code` with no message.
pub const fn fatal(code: u32) -> FatalError {
    FatalError { code, message: "" }
}
