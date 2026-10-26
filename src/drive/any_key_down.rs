//! `drive::any_key_down` (0x444F55): whether any of the 0x80 key bytes (`g_keysPressed`)
//! is set: their or into eax, CF set when not 0.
//!
//! In:  nothing
//! Out: CF; eax the bytes' or, ecx 0x80, esi `g_keysPressed`; the rest kept
//!
//! Caller: `drive::race_frame` (`sub_444D46`, `jb`).

use crate::input::store::DEVICES;
use rc2k_core::drive::inputs::{KEY_COUNT, keys_or};

/// The Rust version: (the bytes' or, esi).
// Converted: `rc2k_core::drive::inputs::keys_or`.
pub fn any_key_down() -> (u32, u32) {
    let keys: [u8; KEY_COUNT] = DEVICES.keys_pressed.read();
    (keys_or(&keys) as u32, DEVICES.keys_pressed.addr())
}
