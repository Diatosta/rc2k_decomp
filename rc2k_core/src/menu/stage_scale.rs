//! The stage's record times scaled for the section.
//!
//! The `f64` product is exact: a whole time times a float has at most 31 + 24 significant bits; at
//! the x87's double precision it rounds as an `f64` product does, and at extended precision it is
//! exact in both for a time below 2^29 (about six days in milliseconds), so `fistp64` rounds the
//! same value.

use crate::math::geometry::fistp64;

/// The sections the factor tables hold.
pub const SECTIONS: i32 = 6;

/// `section` held to 0 .. 5 (signed).
pub fn held_section(section: i32) -> u32 {
    section.clamp(0, SECTIONS - 1) as u32
}

/// A time (a signed whole number) times a factor, rounded to the nearest whole number
/// (ties to even; out of range the x87's "integer indefinite").
pub fn scale_time(time: u32, factor: f32) -> u32 {
    fistp64(time as i32 as f64 * factor as f64) as u32
}

/// The four record times: the first by `factors[1]`, the other three by
/// `factors[0]`.
pub fn scale_times(times: [u32; 4], factors: [f32; 2]) -> [u32; 4] {
    std::array::from_fn(|k| scale_time(times[k], factors[usize::from(k == 0)]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaled() {
        assert_eq!(held_section(-2), 0);
        assert_eq!(held_section(9), 5);
        assert_eq!(held_section(3), 3);
        assert_eq!(scale_time(1000, 1.5), 1500);
        assert_eq!(scale_time(5, 0.5), 2); // ties to even
        assert_eq!(scale_time(7, 0.5), 4);
        assert_eq!(scale_time(u32::MAX, 3.0), (-3i32) as u32);
        assert_eq!(scale_time(0x7FFF_FFFF, 2.0), 0x8000_0000);
        assert_eq!(scale_time(1, f32::NAN), 0x8000_0000);
        assert_eq!(scale_times([10, 20, 30, 40], [2.0, 0.5]), [5, 40, 60, 80]);
    }
}
