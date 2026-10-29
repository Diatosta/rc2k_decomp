//! The shape tables of two kinds of the marks' items, windscreen drops and squares (converted
//! 2026-10-21 from `effects::set_up_windscreen_drops` 0x4BAA55 and `effects::set_up_windscreen_flakes`
//! 0x4BF529): a quarter-circle curve of 0x100 floats, 8 circles of points (one a radius),
//! and for the windscreen drops 0x40 fans of four triangles.
//!
//! As in `math::geometry`, what the x87 keeps on its stack is an `f64`, rounded to `f32`
//! where the original stores it. `fldpi`'s 64-bit pi is kept exactly (`pi_over`); the
//! circles' `fsincos` are the `f64`'s (`scene::trig`): the last bit of a point may differ
//! at a rounding edge.

use crate::scene::trig::{cosine, sine};

/// The items each kind clears, the curve's entries, the circles.
pub const ITEMS: u32 = 0x800;
pub const CURVE: usize = 0x100;
pub const RADII: usize = 8;
/// The windscreen drops' fans: 0x40, four triangles (three word indices each) a fan.
pub const FANS: u32 = 0x40;

/// The x87's `fldpi` (rounded to the nearest): pi = this x 2^-62.
const FLDPI: u64 = 0xC90F_DAA2_2168_C235;

/// `fldpi; fidiv c` at the x87's double precision: its 64-bit pi over `c`, rounded once
/// to 53 bits (to the nearest, ties to even); over 0 an infinity.
pub fn pi_over(c: i32) -> f64 {
    if c == 0 {
        return f64::INFINITY;
    }
    let d = c.unsigned_abs() as u128;
    let n = (FLDPI as u128) << 64;
    let (q, sticky) = (n / d, !n.is_multiple_of(d));
    let drop = 128 - q.leading_zeros() - 53;
    let mut m = q >> drop;
    let rest = q & ((1u128 << drop) - 1);
    let half = 1u128 << (drop - 1);
    if rest > half || (rest == half && (sticky || m & 1 == 1)) {
        m += 1;
    }
    let v = m as f64 * 2f64.powi(drop as i32 - 126);
    if c < 0 { -v } else { v }
}

/// `fld1; fild i; fsub a; fmul b; fmul st, st; fsubr st, st(1); fsqrt; fsubp`, negated
/// when `negate`: 1 - sqrt(1 - ((i - a) b)^2), a quarter circle's curve (a NaN past it).
pub fn curve_value(i: i32, a: f32, b: f32, negate: bool) -> f32 {
    let x = (i as f64 - a as f64) * b as f64;
    let v = 1.0 - (1.0 - x * x).sqrt();
    (if negate { -v } else { v }) as f32
}

/// The curve: entry n of 8 n, negated below 0x400.
pub fn curve(a: f32, b: f32) -> [f32; CURVE] {
    std::array::from_fn(|n| {
        let i = 8 * n as i32;
        curve_value(i, a, b, i < 0x400)
    })
}

/// A circle of `sides` points of radius `r`: each (sin, cos) of the angle times `r` (x,
/// y, the third float left), the angle from 0 on by pi / `c` times `d`, kept on the
/// stack.
pub fn circle(r: f32, sides: u32, c: i32, d: f32) -> Vec<[f32; 2]> {
    let step = pi_over(c) * d as f64;
    let mut angle = 0.0f64;
    (0..sides)
        .map(|_| {
            let p = [
                (sine(angle) * r as f64) as f32,
                (cosine(angle) * r as f64) as f32,
            ];
            angle += step;
            p
        })
        .collect()
}

/// The windscreen drops' fans: fan n's four triangles (6 n, 6 n + 1 + k, 6 n + 2 + k).
pub fn fan(n: u32) -> [[u16; 3]; 4] {
    let first = n * 6;
    std::array::from_fn(|k| {
        let b = first + 1 + k as u32;
        [first as u16, b as u16, (b + 1) as u16]
    })
}

/// `fld v; fmul a [; fmul b]; fstp`: the windscreen drops' four scaled floats.
pub fn scaled(v: f32, by: &[f32]) -> f32 {
    by.iter().fold(v as f64, |p, &k| p * k as f64) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes() {
        assert_eq!(pi_over(1), std::f64::consts::PI);
        assert_eq!(pi_over(-2), -std::f64::consts::FRAC_PI_2);
        assert_eq!(pi_over(3), std::f64::consts::FRAC_PI_3);
        assert_eq!(pi_over(0), f64::INFINITY);
        assert_eq!(curve_value(0x400, 1024.0, 0.001, false), 0.0);
        assert_eq!(curve_value(0, 0.0, 1.0, true), -0.0);
        assert!(curve_value(5, 0.0, 1.0, false).is_nan());
        let c = curve(1024.0, 1.0 / 1024.0);
        assert_eq!((c[0], c[0x80]), (-1.0, 0.0));
        let p = circle(2.0, 4, 2, 1.0);
        assert_eq!(p[0], [0.0, 2.0]);
        assert_eq!(
            p[1],
            [2.0, (std::f64::consts::FRAC_PI_2.cos() * 2.0) as f32]
        );
        assert_eq!(fan(1), [[6, 7, 8], [6, 8, 9], [6, 9, 10], [6, 10, 11]]);
        assert_eq!(scaled(2.0, &[3.0, 0.5]), 3.0);
    }
}
