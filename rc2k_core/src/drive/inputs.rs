//! The race's input words: four words packed into one for the replays, whether any key is down, and
//! a car's words from its device or its replay block.

use crate::Invalid;

/// Three input words packed into one: `a | ((b << 9 | c) << 9)`, << 4 (high bits lost).
pub fn pack_inputs(a: u32, b: u32, c: u32) -> u32 {
    (a | ((b << 9 | c) << 9)) << 4
}

/// A packed word taken apart: [bits 4 .. 12, the top ten (signed, shifted down 22),
/// bits 13 .. 21].
pub fn unpack_inputs(x: u32) -> [u32; 3] {
    [
        (x >> 4) & 0x1FF,
        ((x as i32) >> 22) as u32,
        (x >> 13) & 0x1FF,
    ]
}

/// The number of key bytes.
pub const KEY_COUNT: usize = 0x80;

/// The key bytes or'ed together: not 0 when any key is down.
pub fn keys_or(keys: &[u8; KEY_COUNT]) -> u8 {
    keys.iter().fold(0, |o, &k| o | k)
}

/// The cars a race has (the car tables' length).
pub const CARS: u32 = 8;

/// No device (`FrameWork.listed_car_sounds[0][car]`).
pub const NO_DEVICE: u32 = u32::MAX;

/// Refused: a current car past the tables (checked before anything else).
pub fn check_car(car: u32) -> Result<(), Invalid> {
    if car >= CARS {
        return Err(Invalid("a current car past 7"));
    }
    Ok(())
}

/// The car's input words (`PhysicsTables.input_words[0]` ..): the device's readings for a car
/// with one, else 0.
pub fn device_words(readings: Option<[u32; 4]>) -> [u32; 4] {
    readings.unwrap_or([0; 4])
}

/// With a replay on, which way the words go between the car's block (+0xFD0 ..) and the
/// words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayWords {
    /// The block gives them (a car played back).
    FromBlock,
    /// The block takes them (the car recorded, `NetVars.player`).
    IntoBlock,
}

/// With a replay on: the car recorded writes its words into its block, every other car reads
/// them from it.
pub fn replay_words(car: u32, recorded: u32) -> ReplayWords {
    if car == recorded {
        ReplayWords::IntoBlock
    } else {
        ReplayWords::FromBlock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed() {
        let w = pack_inputs(0x1AB, 0x155, 0x0CD);
        assert_eq!(w, (0x1AB | (0x155 << 18) | (0xCD << 9)) << 4);
        assert_eq!(unpack_inputs(w), [0x1AB, 0x155, 0xCD]);
        assert_eq!(unpack_inputs(0x8000_0000)[1], 0xFFFF_FE00);
    }

    #[test]
    fn keys() {
        let mut k = [0u8; KEY_COUNT];
        assert_eq!(keys_or(&k), 0);
        k[3] = 0x80;
        k[0x7F] = 1;
        assert_eq!(keys_or(&k), 0x81);
    }

    #[test]
    fn car_words() {
        assert!(check_car(7).is_ok() && check_car(8).is_err());
        assert_eq!(device_words(None), [0; 4]);
        assert_eq!(device_words(Some([1, 2, 3, 4])), [1, 2, 3, 4]);
        assert_eq!(replay_words(2, 2), ReplayWords::IntoBlock);
        assert_eq!(replay_words(2, 5), ReplayWords::FromBlock);
    }
}
