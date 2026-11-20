//! A device's state copies (+0x2C .. +0x38) and
//! readings (+0x4DC .. +0x4E4) 0, its bits reading +0x4E8 0xE00 (the hat centred).

use crate::input::types::InputDevice;

/// See the header.
pub(super) fn clear_device_readings(record: &InputDevice) {
    // The state copies and the readings 0, the bits reading the hat centred.
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

    /// The fields cleared are the places the original cleared, with its values; the rest of the
    /// record is as it was.
    #[test]
    fn clears_the_original_places() {
        let record = InputDevice::from_bytes(&[0xAA; 0x4F8]);
        let mut expected = [0xAA; 0x4F8];
        use rc2k_core::input::devices::{BITS, BUTTONS, HAT_CENTRED, READINGS};
        let cleared = [
            (0x2C, 0),
            (0x30, 0),
            (0x34, 0),
            (BUTTONS, 0),
            (READINGS, 0),
            (READINGS + 4, 0),
            (READINGS + 8, 0),
            (BITS, HAT_CENTRED),
        ];
        for (offset, value) in cleared {
            expected[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        clear_device_readings(&record);
        assert!(record_bytes(&record) == expected);
    }
}
