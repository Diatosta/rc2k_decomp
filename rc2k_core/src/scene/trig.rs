//! Sine, cosine and sine-and-cosine as `f64`s with the x87's edge cases (used by the scene's
//! fine turns and the effects' kept vectors). Results may differ from the x87's in the last
//! bit of a float at a rounding edge (about 2^-29 a value). A NaN goes through as it is, an
//! infinity gives the "indefinite", and an angle of 2^63 or more is left as it is.

/// The x87's "indefinite" as an `f64` (a float of it is 0xFFC00000).
pub const INDEFINITE: f64 = f64::from_bits(0xFFF8_0000_0000_0000);

/// What `fsin` (`fcos`) leaves for `x`.
fn x87(x: f64, f: fn(f64) -> f64) -> f64 {
    if x.is_infinite() {
        INDEFINITE
    } else if x.is_nan() || x.abs() >= 9_223_372_036_854_775_808.0 {
        x
    } else {
        f(x)
    }
}

/// `fsin`'s result.
pub fn sine(x: f64) -> f64 {
    x87(x, crate::math::portable::sin)
}

/// `fcos`'s result.
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
