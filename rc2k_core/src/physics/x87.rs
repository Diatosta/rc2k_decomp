//! The x87's habits the physics' converted float code keeps (2026-10-21): a float loaded
//! and stored again comes back with a signalling NaN made quiet; the compares the
//! originals read (`fcomp` and the status word's C0 / C3) count an unordered pair as
//! below and as equal; `fistp` rounds to the nearest, ties to even, and stores the
//! "integer indefinite" (0x80000000) for what does not fit.
//!
//! The values the x87 keeps on its stack are `f64`s in the converted code, rounded to
//! `f32` where the original stores (`notes/conversion.md`, the float pass).

pub use crate::math::geometry::{fistp, fistp64};
use std::cmp::Ordering;

/// `fld; fstp`: the float as it comes back (a signalling NaN made quiet).
pub fn loaded(x: f32) -> f32 {
    if x.is_nan() {
        f32::from_bits(x.to_bits() | 0x0040_0000)
    } else {
        x
    }
}

/// `fld; fchs; fstp`: the float with its sign turned (a signalling NaN made quiet).
pub fn negated(x: f32) -> f32 {
    loaded(-x)
}

/// `fld; fabs; fstp`: the float without its sign (a signalling NaN made quiet).
pub fn abs(x: f32) -> f32 {
    loaded(x.abs())
}

/// C0 of `fld a; fcomp b`: a below b, or unordered.
pub fn below(a: f64, b: f64) -> bool {
    !matches!(a.partial_cmp(&b), Some(Ordering::Greater | Ordering::Equal))
}

/// C3 of `fld a; fcomp b`: a equal to b, or unordered.
pub fn equal(a: f64, b: f64) -> bool {
    matches!(a.partial_cmp(&b), Some(Ordering::Equal) | None)
}

/// `fld lim; fld v; fcompp; jnb; fld lim; fstp v`: v raised to lim when below it (or
/// unordered).
pub fn hold_low(v: f32, lim: f32) -> f32 {
    if below(v as f64, lim as f64) {
        loaded(lim)
    } else {
        v
    }
}

/// `fld lim; fld v; fcompp; jb; fld lim; fstp v`: v lowered to lim unless below it (or
/// unordered).
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
