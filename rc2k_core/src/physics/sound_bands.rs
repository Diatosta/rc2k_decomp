//! The car sounds' bands (converted 2026-10-21 from the four twins
//! `physics::sound_band_1` 0x45D067, `sound_band_2` 0x45D097, `sound_band_3` 0x45D0F1
//! and `sound_band_4` 0x45D121): a value times a factor as a whole number (`fistp`, kept
//! in `dword_6D1AEC`), less a step, held to 0 ..= 0x40 (by the sign). A reading: volume
//! or pitch steps.
//!
//! Floats: the product of two floats is exact in an `f64` (48 bits), as on the x87 at
//! double or extended precision, so the whole number is the original's to the bit.

use crate::math::geometry::fistp64;

/// The band's top.
pub const TOP: u32 = 0x40;

/// (the whole number kept, the band).
pub fn sound_band(value: f32, factor: f32, step: u32) -> (i32, u32) {
    let n = fistp64(value as f64 * factor as f64);
    let mut b = (n as u32).wrapping_sub(step);
    if (b as i32) < 0 {
        b = 0;
    }
    if (b.wrapping_sub(TOP) as i32) >= 0 {
        b = TOP;
    }
    (n, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bands() {
        assert_eq!(sound_band(10.4, 2.0, 1), (21, 20));
        assert_eq!(sound_band(2.5, 1.0, 1), (2, 1));
        assert_eq!(sound_band(0.5, 1.0, 1), (0, 0));
        assert_eq!(sound_band(100.0, 1.0, 10), (100, TOP));
        assert_eq!(sound_band(f32::NAN, 1.0, 1), (i32::MIN, TOP));
        assert_eq!(sound_band(-3.0, 1.0, 1), (-3, 0));
    }
}
