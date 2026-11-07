//! `input::clear_device_readings` (0x40883C): a device's state copies (+0x2C .. +0x38) and
//! readings (+0x4DC .. +0x4E4) 0, its bits reading +0x4E8 0xE00 (the hat centred).
//!
//! Takes: the device's record (`input/devices.rs`)
//!
//! Callers: `sub_408473` (an acquiring that failed), `sub_4087F3` (the release).

use crate::input::types::InputDevice;

/// The Rust version (see the header).
pub fn clear_device_readings(record: &InputDevice) {
    // `rc2k_core::input::devices::CLEARED_READINGS`, in its order: the state copies
    // (+0x2C .. +0x38) and the readings 0, the bits reading the hat centred.
    for axis in &record.axes {
        axis.set(0);
    }
    record.buttons.set(0);
    for reading in &record.readings {
        reading.set(0);
    }
    record.bits.set(rc2k_core::input::devices::HAT_CENTRED);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::devices::record_bytes;

    /// The fields cleared are the places the core lists, with its values; the rest of the
    /// record is as it was.
    #[test]
    fn clears_what_the_core_lists() {
        let record = InputDevice::from_bytes(&[0xAA; 0x4F8]);
        let mut expected = [0xAA; 0x4F8];
        for (offset, value) in rc2k_core::input::devices::CLEARED_READINGS {
            expected[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        clear_device_readings(&record);
        assert!(record_bytes(&record) == expected);
    }
}
