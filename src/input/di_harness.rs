//! What the DirectInput functions share (`acquire_device`, `list_joysticks`,
//! `joystick_found`, `axis_found`, `begin_force`, `end_force`, `effect_found`,
//! `make_effect`, `effect_objects`): the game's variables they use.

use crate::errors::FatalError;
use crate::framework::rva;
use crate::input::store::DEVICES;
use crate::store::game_addr;

pub const CAPS_LEN: usize = std::mem::size_of::<crate::input::types::DiDevCaps>();
pub const CONTROLLER_LEN: u32 = std::mem::size_of::<crate::input::types::Controller>() as u32;
pub const CONTROLLERS: u32 = rc2k_core::input::joystick::CONTROLLERS;
pub const JOYSTICK_FORMAT_RVA: usize = rva(0x4D7220); // c_dfDIJoystick
/// A record's six effects: where they start, and each one's size.
pub const EFFECTS_AT: u32 = crate::field!(crate::input::types::InputDevice, effects);
pub const EFFECT_LEN: u32 = std::mem::size_of::<crate::input::types::ForceEffect>() as u32;

/// A game variable's address.
pub fn a(rva: impl crate::store::Where) -> u32 {
    game_addr(rva)
}

/// The controller's entry for a record (`g_controllers` + 0x50 its number), unchecked as
/// the original (`imul edi, [esi], 50h`).
pub fn entry_of(record: u32) -> Result<u32, crate::errors::BadInput> {
    Ok((DEVICES.controllers[0].addr()).wrapping_add(
        crate::input::devices::at(
            record,
            crate::field!(crate::input::types::InputDevice, controller),
        )?
        .wrapping_mul(CONTROLLER_LEN),
    ))
}

/// The game's fatal error `code` with no message.
pub const fn fatal(code: u32) -> FatalError {
    FatalError { code, message: "" }
}
