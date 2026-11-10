//! `race::create_controls` (0x40941A): `dword_4F7DE4` 3 (`race::load_stage`'s "createcontrols";
//! what the 3 stands for is not read).
//!
//! One caller, `race::load_stage`.

pub fn create_controls(input: &crate::input::owner::Input) {
    input.devices.input_command.set(3); // g_inputCommand
}
