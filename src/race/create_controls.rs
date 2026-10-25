//! `race::create_controls` (0x40941A): `dword_4F7DE4` 3 (`race::load_stage`'s
//! "createcontrols"; what the 3 stands for is not read).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! One caller, `race::load_stage`.

use crate::input::store::DEVICES;

pub fn create_controls() {
    DEVICES.input_command.set(3); // g_inputCommand
}
