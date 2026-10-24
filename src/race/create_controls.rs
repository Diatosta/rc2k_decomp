//! `race::create_controls` (0x40941A): `dword_4F7DE4` 3 (`race::load_stage`'s
//! "createcontrols"; what the 3 stands for is not read).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! One caller, `race::load_stage`.

use crate::input::store::DeviceSlot;
use crate::store::game_write;

const CONTROLS_RVA: DeviceSlot = DeviceSlot::at(0x4F7DE4); // dword_4F7DE4

pub fn create_controls() {
    unsafe { game_write(CONTROLS_RVA, 3u32) };
}
