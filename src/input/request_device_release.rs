//! `input::request_device_release` (0x409386): `g_inputCommand` 2: the input thread ends the
//! effects and gives both devices back (`input::device_command`). The same store as
//! `race::release_controls` (0x409425).
//!
//! In:  nothing
//! Out: every register kept
//! Caller: `menu::release_controller`.

use crate::input::input_tick::COMMAND_RVA;
use crate::store::game_write;

/// The Rust version (see the header).
pub fn request_device_release() {
    unsafe { game_write(COMMAND_RVA, 2u32) };
}
