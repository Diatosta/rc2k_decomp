//! The SSE estimates `rcpss` and `rsqrtss` as tables, so every machine gets the same bits.
//!
//! The game's code uses the two instructions (a reciprocal and a reciprocal square root,
//! good to about 12 bits). Their results are not fixed by any standard and differ between
//! processors, so a replay or a network game drawn from them would differ too. These
//! functions give one processor's results everywhere: an AMD Ryzen 7 5800X's, whose
//! estimates depend only on the input's exponent and its top 12 mantissa bits. The tables
//! (`estimate_tables`) hold its results for one octave (two for the square root);
//! `tools/estimate_tables` makes them and checks these functions against the processor it
//! runs on over all 2^32 inputs (0 differences on the 5800X, 2026-11-10).

use super::estimate_tables::{RECIPROCAL, ROOT_EVEN, ROOT_ODD};

const SIGN: u32 = 0x8000_0000;
const INFINITY: u32 = 0x7F80_0000;
const QUIET: u32 = 0x0040_0000;
/// The default NaN the instructions give for a negative input.
const DEFAULT_NAN: u32 = 0xFFC0_0000;

fn parts(x: u32) -> (u32, u32, u32) {
    (x & SIGN, (x >> 23) & 0xFF, x & 0x7F_FFFF)
}

/// `rcpss`: about 1 / `a`. Zeros and denormals give an infinity of their sign, infinities
/// a zero, a NaN itself made quiet; results below the normal range are flushed to zero.
pub fn reciprocal_estimate(a: f32) -> f32 {
    let x = a.to_bits();
    let (sign, exponent, mantissa) = parts(x);
    let bits = match exponent {
        0xFF if mantissa != 0 => x | QUIET,
        0xFF => sign,
        0 => sign | INFINITY,
        _ => {
            let t = RECIPROCAL[(mantissa >> 11) as usize];
            match ((t >> 23) & 0xFF) as i32 + 127 - exponent as i32 {
                ..=0 => sign,
                255.. => sign | INFINITY,
                e => sign | ((e as u32) << 23) | (t & 0x7F_FFFF),
            }
        }
    };
    f32::from_bits(bits)
}

/// `rsqrtss`: about 1 / sqrt(`a`). Zeros and denormals give an infinity of their sign,
/// +infinity zero, a negative number the default NaN, a NaN itself made quiet.
pub fn reciprocal_root_estimate(a: f32) -> f32 {
    let x = a.to_bits();
    let (sign, exponent, mantissa) = parts(x);
    let bits = match exponent {
        0xFF if mantissa != 0 => x | QUIET,
        0xFF if sign != 0 => DEFAULT_NAN,
        0xFF => 0,
        0 => sign | INFINITY,
        _ if sign != 0 => DEFAULT_NAN,
        _ => {
            // An even power of two scales the root by a whole power; an odd one is the
            // even one's octave above (the table for 2 x 1.m).
            let odd = (exponent as i32 - 127) & 1 != 0;
            let (table, base) = if odd {
                (&ROOT_ODD, 128)
            } else {
                (&ROOT_EVEN, 127)
            };
            let t = table[(mantissa >> 11) as usize];
            let e = ((t >> 23) & 0xFF) as i32 - (exponent as i32 - base) / 2;
            ((e as u32) << 23) | (t & 0x7F_FFFF)
        }
    };
    f32::from_bits(bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn special_values() {
        let r = |x: f32| reciprocal_estimate(x).to_bits();
        let s = |x: f32| reciprocal_root_estimate(x).to_bits();
        assert_eq!(r(0.0), INFINITY);
        assert_eq!(r(-0.0), SIGN | INFINITY);
        assert_eq!(r(f32::from_bits(1)), INFINITY);
        assert_eq!(r(f32::INFINITY), 0);
        assert_eq!(r(f32::NEG_INFINITY), SIGN);
        assert_eq!(r(f32::from_bits(0x7F80_0001)), 0x7FC0_0001);
        assert_eq!(r(f32::MAX), 0);
        assert_eq!(s(0.0), INFINITY);
        assert_eq!(s(-0.0), SIGN | INFINITY);
        assert_eq!(s(f32::INFINITY), 0);
        assert_eq!(s(f32::NEG_INFINITY), DEFAULT_NAN);
        assert_eq!(s(-1.0), DEFAULT_NAN);
        assert_eq!(s(f32::from_bits(0x7F80_0001)), 0x7FC0_0001);
    }

    /// Intel documents both as within 1.5 x 2^-12 relative error; check that on a spread
    /// of inputs over the whole normal range.
    #[test]
    fn within_the_documented_error() {
        let bound = 1.5 / 4096.0;
        let mut x = 0x0080_0000u32;
        while x < 0x7F80_0000 {
            let a = f32::from_bits(x) as f64;
            let r = reciprocal_estimate(a as f32) as f64;
            if r != 0.0 {
                assert!((r * a - 1.0).abs() <= bound, "rcp {x:#010x}: {r}");
            }
            let s = reciprocal_root_estimate(a as f32) as f64;
            assert!((s * a.sqrt() - 1.0).abs() <= bound, "rsqrt {x:#010x}: {s}");
            x += 0x1357;
        }
    }

    #[test]
    fn known_results() {
        // The 5800X's results, from the tables' first entries.
        assert_eq!(reciprocal_estimate(1.0).to_bits(), 0x3F7F_F000);
        assert_eq!(reciprocal_estimate(-2.0).to_bits(), 0xBEFF_F000);
        assert_eq!(reciprocal_root_estimate(1.0).to_bits(), ROOT_EVEN[0]);
        assert_eq!(
            reciprocal_root_estimate(4.0).to_bits(),
            ROOT_EVEN[0] - (1 << 23)
        );
        assert_eq!(reciprocal_root_estimate(2.0).to_bits(), ROOT_ODD[0]);
    }
}
