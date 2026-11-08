//! `drive::any_key_down` (0x444F55): whether any of the 0x80 key bytes (`g_keysPressed`)
//! is set: their or into eax, CF set when not 0.
//!
//! Caller: `drive::race_frame` (0x444D46, `jb`).

use rc2k_core::drive::inputs::{KEY_COUNT, keys_or};

/// The Rust version: the bytes' or.
// Converted: `rc2k_core::drive::inputs::keys_or`.
pub fn any_key_down(input: &crate::input::owner::Input) -> u32 {
    let keys: [u8; KEY_COUNT] = input.devices.keys_pressed.read();
    keys_or(&keys) as u32
}
