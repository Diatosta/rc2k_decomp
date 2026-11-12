//! `input::list_joysticks` (0x408216): unless the configuration turned them off
//! (`no_joysticks` not 0), the attached joysticks listed into `g_controllers`
//! (`EnumDevices` of `direct_input`, DirectInput, for joysticks (4), attached only (1), with
//! `input::joystick_found`, context 0; its result not looked at).
//!
//! Caller: `input::input_thread` (its start). The method DirectInput's own (in the
//! standalone program Rust's, `input::di_objects`).

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
