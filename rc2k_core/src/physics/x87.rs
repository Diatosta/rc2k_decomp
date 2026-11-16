//! The x87's habits the physics' float code keeps: a signalling NaN made quiet on load and
//! store, compares that count an unordered pair as below and as equal, rounding to the nearest
//! (ties to even) with 0x80000000 for what does not fit. Stack values are `f64`s, rounded to
//! `f32` where stored.

pub use crate::math::geometry::{fistp, fistp64};
use std::cmp::Ordering;

/// The float as it comes back (a signalling NaN made quiet).
pub fn loaded(x: f32) -> f32 {
    if x.is_nan() {
        f32::from_bits(x.to_bits() | 0x0040_0000)
    } else {
        x
    }
}

/// The float with its sign turned.
pub fn negated(x: f32) -> f32 {
    loaded(-x)
}

/// The float without its sign.
pub fn abs(x: f32) -> f32 {
    loaded(x.abs())
}

/// Whether `a` is below `b`, or unordered.
pub fn below(a: f64, b: f64) -> bool {
    !matches!(a.partial_cmp(&b), Some(Ordering::Greater | Ordering::Equal))
}

/// Whether `a` equals `b`, or unordered.
pub fn equal(a: f64, b: f64) -> bool {
    matches!(a.partial_cmp(&b), Some(Ordering::Equal) | None)
}

/// v raised to lim when below it (or unordered).
pub fn hold_low(v: f32, lim: f32) -> f32 {
    if below(v as f64, lim as f64) {
        loaded(lim)
    } else {
        v
    }
}

/// v lowered to lim unless below it (or unordered).
pub fn hold_high(v: f32, lim: f32) -> f32 {
    if below(v as f64, lim as f64) {
        v
    } else {
        loaded(lim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn habits() {
        let signalling = f32::from_bits(0x7F80_0001);
        assert_eq!(loaded(signalling).to_bits(), 0x7FC0_0001);
        assert_eq!(negated(1.5), -1.5);
        assert_eq!(negated(signalling).to_bits(), 0xFFC0_0001);
        assert_eq!(abs(-0.0).to_bits(), 0);
        assert!(below(1.0, 2.0) && below(f64::NAN, 2.0) && !below(2.0, 2.0));
        assert!(equal(2.0, 2.0) && equal(f64::NAN, 0.0) && !equal(1.0, 2.0));
        assert_eq!(hold_low(-1.0, 0.0), 0.0);
        assert_eq!(hold_low(f32::NAN, 0.0), 0.0);
        assert_eq!(hold_high(2.0, 1.0), 1.0);
        assert_eq!(hold_high(1.0, 1.0), 1.0);
        assert!(hold_high(f32::NAN, 1.0).is_nan());
        assert_eq!(fistp64(2.5), 2);
        assert_eq!(fistp64(3.5), 4);
        assert_eq!(fistp64(f64::NAN), i32::MIN);
    }
}
