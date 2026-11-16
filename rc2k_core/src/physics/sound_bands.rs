//! The car sounds' bands: a value times a factor as a whole number, less a step, held to
//! 0 ..= 0x40 (signed). The product of two floats is exact in an `f64`, so the whole number is
//! bit-identical to the x87's.

use crate::math::geometry::fistp64;

/// The band's top.
pub const TOP: u32 = 0x40;

/// Returns (the whole number kept, the band).
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
