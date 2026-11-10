//! The maths functions with the same bits on every machine.
//!
//! `f64::sin` and its kin call the platform's maths library (glibc, the Windows CRT,
//! Apple's), whose results may differ in the last bit; a replay or a network game must not
//! depend on which one. These are the `libm` crate's (a port of musl's), built without its
//! processor-specific paths, so every build computes them the same way. The game calls these
//! and never the standard library's (clippy's `disallowed-methods`, `clippy.toml`).

pub fn sin(x: f64) -> f64 {
    libm::sin(x)
}

pub fn cos(x: f64) -> f64 {
    libm::cos(x)
}

/// The sine and the cosine.
pub fn sin_cos(x: f64) -> (f64, f64) {
    libm::sincos(x)
}

/// The angle of (`x`, `y`).
pub fn atan2(y: f64, x: f64) -> f64 {
    libm::atan2(y, x)
}

pub fn exp2(x: f64) -> f64 {
    libm::exp2(x)
}

pub fn log2(x: f64) -> f64 {
    libm::log2(x)
}

pub fn powf(x: f32, y: f32) -> f32 {
    libm::powf(x, y)
}

pub fn sinf(x: f32) -> f32 {
    libm::sinf(x)
}

pub fn cosf(x: f32) -> f32 {
    libm::cosf(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_values() {
        assert_eq!(sin(0.0), 0.0);
        assert_eq!(cos(0.0), 1.0);
        assert_eq!(sin_cos(1.0), (sin(1.0), cos(1.0)));
        assert_eq!(atan2(1.0, 1.0), std::f64::consts::FRAC_PI_4);
        assert_eq!(log2(8.0), 3.0);
        assert_eq!(powf(10.0, 2.0), 100.0);
        // The correctly rounded values, which musl gives.
        assert_eq!(sin(1.0).to_bits(), 0x3FEA_ED54_8F09_0CEE);
        assert_eq!(cos(0.5).to_bits(), 0x3FEC_1528_065B_7D50);
    }
}
