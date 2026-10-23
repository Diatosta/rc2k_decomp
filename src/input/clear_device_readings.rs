//! `input::clear_device_readings` (0x40883C): a device's state copies (+0x2C .. +0x38) and
//! readings (+0x4DC .. +0x4E4) 0, its bits reading +0x4E8 0xE00 (the hat centred).
//!
//! In:  esi = the device's record (`input/devices.rs`)
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_408473` (an acquiring that failed), `sub_4087F3` (the release).
//! Self-test: the original and the Rust version on records of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::input::devices::put;

/// The Rust version (see the header).
pub fn clear_device_readings(record: u32) {
    // Converted: `rc2k_core::input::devices::CLEARED_READINGS`.
    for (offset, value) in rc2k_core::input::devices::CLEARED_READINGS {
        put(record, offset as u32, value);
    }
}
