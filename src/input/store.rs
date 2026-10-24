//! The input module's variables, out of RAL.EXE's image (phase 5, `notes/conversion.md`):
//! - `layout`, IDA 0x4F07C8 .. 0x4F0BC8: the keyboard file's 0x400 bytes as
//!   `strings::load_language` reads it (its head, the layout mode, the current layout,
//!   the layout blocks and the tables they point into; `key_to_char`). One block: the file
//!   is read over all of it and its layouts' offsets are made addresses in it (2026-10-24:
//!   moving only 0x4F07D0 .. 0x4F0820 left the file in the image and the layouts pointing
//!   past the block, "a keyboard layout without its key names" at the redefine page);
//! - `devices`, 0x4F68D8 .. 0x4F7E18: the settings' switches (`nojoysticks` ...), the
//!   DirectInput objects, the joystick's state, caps and properties, the key tables (down,
//!   now, before, pressed, repeat: a byte a key), the mouse word, the two device records
//!   (0x4F8 bytes each; `g_inputDevices` points at them), the controllers' table and the
//!   input command;
//! - `keyboard`, 0x4F7FD8 .. 0x4F8294: the yes/no key, the silent keys' bitmap and the
//!   dead keys' list (0x4F8000) the keyboard file fills;
//! - `mouse`, 0x5C6924 .. 0x5C6948: the mouse's position, clicks and buttons.
//!
//! The effects' GUIDs and the DirectInput identifiers (0x4F7E18 .. 0x4F7FA8) stay in the
//! image: the device records' effect slots point at them, and they are constants.
//!
//! `g_inputDevices` (0x4F6EE8) held two pointers to the records, as the image has them
//! (IDA's addresses); `point_devices_at_records` sets them to where the records are now
//! (the standalone's start-up calls it).
//!
//! Threads: the input thread polls and the game's thread reads, as the image's bytes were
//! shared (volatile reads and writes).

use crate::block;

block!(pub mod layout {
    start: 0x4F07C8,
    end: 0x4F0BC8,
    runs: crate::input::store_data::LAYOUT_RUNS,
});

block!(pub mod devices {
    start: 0x4F68D8,
    end: 0x4F7E18,
    runs: crate::input::store_data::DEVICES_RUNS,
});

block!(pub mod keyboard {
    start: 0x4F7FD8,
    end: 0x4F8294,
    runs: crate::input::store_data::KEYBOARD_RUNS,
});

block!(pub mod mouse {
    start: 0x5C6924,
    end: 0x5C6948,
    runs: crate::input::store_data::MOUSE_RUNS,
});

pub type LayoutSlot = layout::Slot;
pub type DeviceSlot = devices::Slot;
pub type KeyboardSlot = keyboard::Slot;
pub type MouseSlot = mouse::Slot;

/// The two device records' places (IDA 0x4F6EF0 and 0x4F73E8).
pub const RECORD_1: DeviceSlot = DeviceSlot::at(0x4F6EF0);
pub const RECORD_2: DeviceSlot = DeviceSlot::at(0x4F73E8);
/// `g_inputDevices`: the two pointers to the records.
pub const INPUT_DEVICES: DeviceSlot = DeviceSlot::at(0x4F6EE8);

/// Sets `g_inputDevices` to the records' places now.
pub fn point_devices_at_records() {
    unsafe {
        crate::store::game_write(INPUT_DEVICES, RECORD_1.address() as u32);
        crate::store::game_write(INPUT_DEVICES + 4usize, RECORD_2.address() as u32);
    }
}
