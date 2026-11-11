//! The x87's 80-bit values as `f64`: where the game leaves a result on
//! the x87 stack, the converted code has an `f64`; the caller writes it back as the ten
//! bytes the x87 would load. At the game's double precision (control word 0x027F) every
//! result the x87 makes fits an `f64` (53 bits of mantissa), so both ways are exact for
//! them; a value with more bits (made at extended precision) is rounded to the nearest.

const BIAS: i32 = 16383;
const QUIET: u64 = 1 << 51;

/// `x * 2^k`, in steps that do not overflow on the way.
fn scale(mut x: f64, mut k: i32) -> f64 {
    let step = |k: i32| f64::from_bits(((k + 1023) as u64) << 52);
    while k > 1000 {
        x *= step(1000);
        k -= 1000;
    }
    while k < -1000 {
        x *= step(-1000);
        k += 1000;
    }
    x * step(k)
}

/// The ten bytes (mantissa, then sign and exponent, little-endian) as an `f64`, rounded to
/// the nearest. A NaN keeps its sign and its payload's top bits, made quiet (as `fstp
/// qword` does).
pub fn to_f64(v: [u8; 10]) -> f64 {
    let mantissa = u64::from_le_bytes(v[..8].try_into().unwrap());
    let top = u16::from_le_bytes([v[8], v[9]]);
    let negative = top & 0x8000 != 0;
    let exponent = (top & 0x7FFF) as i32;
    let magnitude = if exponent == 0x7FFF {
        if mantissa << 1 == 0 {
            f64::INFINITY
        } else {
            let fraction = (mantissa >> 11) & ((1 << 52) - 1);
            return f64::from_bits((negative as u64) << 63 | 0x7FF << 52 | fraction | QUIET);
        }
    } else if mantissa == 0 {
        0.0
    } else {
        // Denormals have the exponent of 1.
        scale(mantissa as f64, exponent.max(1) - BIAS - 63)
    };
    if negative { -magnitude } else { magnitude }
}

/// An `f64` as the ten bytes, exactly (`fld qword; fstp tbyte`).
pub fn from_f64(f: f64) -> [u8; 10] {
    let bits = f.to_bits();
    let sign = ((bits >> 63) as u16) << 15;
    let biased = ((bits >> 52) & 0x7FF) as i32;
    let fraction = bits & ((1 << 52) - 1);
    let (exponent, mantissa) = match biased {
        0x7FF => (0x7FFF, 1 << 63 | fraction << 11),
        0 if fraction == 0 => (0, 0),
        0 => {
            let shift = fraction.leading_zeros();
            (BIAS + 63 - 1074 - shift as i32, fraction << shift)
        }
        _ => (biased - 1023 + BIAS, 1 << 63 | fraction << 11),
    };
    let mut out = [0u8; 10];
    out[..8].copy_from_slice(&mantissa.to_le_bytes());
    out[8..].copy_from_slice(&(sign | exponent as u16).to_le_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        // 1.0: mantissa 0x8000_0000_0000_0000, exponent 0x3FFF.
        assert_eq!(from_f64(1.0), [0, 0, 0, 0, 0, 0, 0, 0x80, 0xFF, 0x3F]);
        for x in [
            0.0,
            -0.0,
            1.5,
            -3.25e-300,
            5e-324,
            1.7e308,
            f64::INFINITY,
            0.1,
        ] {
            assert_eq!(to_f64(from_f64(x)).to_bits(), x.to_bits(), "{x}");
        }
        assert!(to_f64(from_f64(f64::NAN)).is_nan());
        // pi to 64 bits rounds to the f64 pi.
        let pi = [0x35, 0xC2, 0x68, 0x21, 0xA2, 0xDA, 0x0F, 0xC9, 0x00, 0x40];
        assert_eq!(to_f64(pi), std::f64::consts::PI);
    }
}
