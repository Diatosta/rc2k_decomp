//! The x87's `fsin`, `fcos` and `fsincos` as `f64`s (with the scene's
//! fine turns and the marks' kept vectors). The x87 works these out to about its own 64
//! bits whatever its precision control says, and the game stores them as floats, so the
//! `f64`'s may differ from the x87's in the last bit of the float at a rounding edge (about
//! 2^-29 a value; `notes/conversion.md`, float pass 2). Kept as the x87 has them: a NaN
//! goes through as it is, an infinity gives the "indefinite" (the invalid operation's
//! NaN), and an angle of 2^63 or more either way is left as it is (C2 set, nothing
//! worked out).

/// The x87's "indefinite" as an `f64` (a float of it is 0xFFC00000).
pub const INDEFINITE: f64 = f64::from_bits(0xFFF8_0000_0000_0000);

/// What `fsin` (`fcos`) leaves for `x`: `f` of it, or as said above.
fn x87(x: f64, f: fn(f64) -> f64) -> f64 {
    if x.is_infinite() {
        INDEFINITE
    } else if x.is_nan() || x.abs() >= 9_223_372_036_854_775_808.0 {
        x
    } else {
        f(x)
    }
}

/// `fsin`.
pub fn sine(x: f64) -> f64 {
    x87(x, crate::math::portable::sin)
}

/// `fcos`.
pub fn cosine(x: f64) -> f64 {
    x87(x, crate::math::portable::cos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_the_x87() {
        assert_eq!(sine(0.0), 0.0);
        assert_eq!(cosine(0.0), 1.0);
        assert_eq!(sine(1e19), 1e19);
        assert_eq!(cosine(-1e19), -1e19);
        assert_eq!(sine(f64::INFINITY).to_bits(), INDEFINITE.to_bits());
        assert!(cosine(f64::NAN).is_nan());
        // The payload of an f64 -> f32 cast of a NaN is left unspecified by Rust (LLVM folds
        // it at opt-level 1), so only the NaN class is checked, not the bits 0xFFC00000.
        assert!((INDEFINITE as f32).is_nan());
        assert!((sine(1.0) - 0.8414709848078965).abs() < 1e-15);
    }
}
