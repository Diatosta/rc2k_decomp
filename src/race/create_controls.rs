//! Sets `Devices.input_command` to 3 (the "createcontrols" step of `load_stage`, its only caller).
//! The meaning of the 3 is not read anywhere.

pub fn create_controls(input: &crate::input::owner::Input) {
    input.devices.input_command.set(3);
}
