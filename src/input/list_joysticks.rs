//! Unless the configuration turned them off, lists the attached joysticks into the controllers'
//! table (`EnumDevices` for joysticks, attached only, with `input::joystick_found`; its result is
//! not looked at). Called at the input thread's start; `list_new_joysticks` adds those plugged
//! in later.

use crate::input::owner::Input;

pub(super) fn list_joysticks(input: &Input, system: &crate::system::owner::System) {
    if input.devices.no_joysticks.is_set() {
        return;
    }
    crate::input::di_objects::enum_devices(
        input.devices.direct_input.get(),
        4,
        |instance| crate::input::joystick_found::callback(input, system, instance),
        1,
    );
    input
        .devices_listed
        .store(true, std::sync::atomic::Ordering::Release);
}

/// The joysticks plugged in since the listing added to the controllers' table (the input
/// thread calls this when SDL reports a change): each joystick whose instance is not in the
/// table yet, so a pad plugged in after the start can be chosen on the controller pages and
/// driven with. A pad unplugged and plugged back keeps its place (`platform::input`), so its
/// entry, and the players who chose it, stay as they were.
pub(super) fn list_new_joysticks(input: &Input, system: &crate::system::owner::System) {
    if input.devices.no_joysticks.is_set() || input.devices.direct_input.get() == 0 {
        return;
    }
    let before = input.devices.controller_count.get();
    crate::input::di_objects::enum_devices(
        input.devices.direct_input.get(),
        4,
        |instance| {
            let count = input.devices.controller_count.get() as usize;
            let listed = input.devices.controllers[..count]
                .iter()
                .any(|c| c.instance.read() == instance.instance);
            if listed {
                1
            } else {
                crate::input::joystick_found::callback(input, system, instance)
            }
        },
        1,
    );
    let after = input.devices.controller_count.get();
    // Plugged in or out: the players' controllers worked out again (`input::device_ids`).
    input
        .devices_listed
        .store(true, std::sync::atomic::Ordering::Release);
    if after != before {
        crate::program_log::log(&format!(
            "input: {} controller(s) plugged in, {after} listed",
            after - before
        ));
    }
}
