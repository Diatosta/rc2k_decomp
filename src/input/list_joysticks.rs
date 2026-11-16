//! Unless the configuration turned them off, lists the attached joysticks into the controllers'
//! table (`EnumDevices` for joysticks, attached only, with `input::joystick_found`; its result is
//! not looked at). Called at the input thread's start.

use crate::input::owner::Input;

pub fn list_joysticks(input: &Input, system: &crate::system::owner::System) {
    if input.devices.no_joysticks.is_set() {
        return;
    }
    crate::input::di_objects::enum_devices(
        input.devices.direct_input.get(),
        4,
        |instance| crate::input::joystick_found::callback(input, system, instance),
        1,
    );
}
