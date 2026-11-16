//! Whether any of the 0x80 key bytes is set
//! (their bitwise or is not 0).

use rc2k_core::drive::inputs::{KEY_COUNT, keys_or};

pub fn any_key_down(input: &crate::input::owner::Input) -> u32 {
    let keys: [u8; KEY_COUNT] = input.devices.keys_pressed.read();
    keys_or(&keys) as u32
}
