//! A sound placed around the listener, its whole-number rules (the products and the angle are
//! the caller's, rounded there).
//!
//! Without 3D the volume (the request's x the sample's factor x the request's factor) is
//! held to 0 .. 0x40 and the place between left and right comes from the angle; with Dolby
//! a place on the left has its second byte inverted.

use crate::Invalid;

/// A sound with this bit is none: nothing is played.
pub const NO_SOUND: u32 = 0x8000;
/// The sound mode's bits: 3D, Dolby.
pub const MODE_3D: u32 = 0x10_0000;
pub const MODE_DOLBY: u32 = 0x200_0000;
/// Number of sample records.
pub use crate::audio::voices::SAMPLES;
/// The loudest volume.
pub const LOUDEST: i32 = 0x40;

/// Whether the request's sound is one to play.
pub fn played(sound: u32) -> bool {
    sound & NO_SOUND == 0
}

/// The sample record of a sound without 3D: its low word, signed. Refuses one past the
/// records.
pub fn sample_of(sound: u32) -> Result<u32, Invalid> {
    let number = sound as u16 as i16 as i32;
    if !(0..SAMPLES as i32).contains(&number) {
        return Err(Invalid("a sound past the sample records"));
    }
    Ok(number as u32)
}

/// The volume from its rounded whole number, held to 0 .. 0x40.
pub fn volume(whole: u32) -> u32 {
    (whole as i32).clamp(0, LOUDEST) as u32
}

/// The place sent: 0 when the sound is straight ahead (x is a zero of either sign), else the whole
/// number from the angle; with Dolby (`mode`) and an angle below zero the second byte is inverted.
pub fn place(whole: u32, x: f32, angle: i32, mode: u32) -> u32 {
    let mut place = if x == 0.0 { 0 } else { whole };
    if mode & MODE_DOLBY != 0 && angle < 0 {
        place ^= 0xFF00;
    }
    place
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules() {
        assert!(played(0x7FFF) && !played(0x8001));
        assert_eq!(sample_of(0x1_01CC), Ok(0x1CC));
        assert_eq!(sample_of(0x1CD), Ok(0x1CD));
        assert!(sample_of(SAMPLES).is_err());
        assert!(sample_of(0xFFFF).is_err());
        assert_eq!(volume(0x41), 0x40);
        assert_eq!(volume(0xFFFF_FFFF), 0);
        assert_eq!(volume(0x20), 0x20);
        assert_eq!(place(0x40, -0.0, -1, MODE_DOLBY), 0xFF00);
        assert_eq!(place(0x40, 1.0, -1, MODE_DOLBY), 0xFF40);
        assert_eq!(place(0x40, 1.0, 1, MODE_DOLBY), 0x40);
        assert_eq!(place(0x40, 1.0, -1, 0), 0x40);
    }
}
