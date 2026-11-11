//! The x87's registers in plain Rust: `X`, an `f64` whose arithmetic rounds as this
//! thread's (emulated) control word says, the way the x87 does at double precision.
//!
//! With rounding to nearest (the usual) an `X` operation is the `f64` operation itself.
//! For chop, down and up the exact error of the nearest result (an error-free
//! transformation: two-sum, an fma, the remainder of a division or a root) tells which
//! way the result steps. `st` is `fstp dword`, `to_int` is `fistp`, `ld` and `int` are
//! `fld dword` and `fild`. Every helper of `x87.rs`, `x87/run.rs`, `x87/load.rs` and
//! `normalise_vector.rs` is written with these, so it rounds as the original does under
//! whatever control word the game has set.
//!
//! Limits: the precision bits are not modelled (53, the game's threads' start; after the
//! game's `finit` the x87 would use 64 bits), the exponent range is the double's, an
//! 80-bit value with more than 53 significant bits is rounded to nearest when loaded, and
//! `fsin`, `fcos`, `fpatan` are libm's (within the last bit of the double).

use crate::math::x87::{Rounding, rounding, rounds_to_nearest};
use rc2k_core::math::portable;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// What the x87 holds in a register: an `f64` whose arithmetic rounds as the (emulated)
/// control word says, as the x87 does.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub(crate) struct X(pub(crate) f64);

/// `fld1`.
pub(crate) const ONE: X = X(1.0);

/// The result `r`, rounded to nearest, moved one step where the rounding asks for the
/// neighbour: `error` has the sign of (exact result - `r`).
fn directed(r: f64, error: f64) -> X {
    if !r.is_finite() || error == 0.0 || error.is_nan() {
        return X(r);
    }
    X(match rounding() {
        Rounding::Nearest => r,
        Rounding::Chop if r > 0.0 && error < 0.0 => r.next_down(),
        Rounding::Chop if r < 0.0 && error > 0.0 => r.next_up(),
        Rounding::Down if error < 0.0 => r.next_down(),
        Rounding::Up if error > 0.0 => r.next_up(),
        _ => r,
    })
}

/// A sum under a directed rounding (`r` is the sum rounded to nearest).
#[cold]
fn add_directed(a: f64, b: f64, r: f64) -> X {
    if r == 0.0 && a.is_sign_negative() != b.is_sign_negative() {
        // a sum of opposites is +0, but -0 when rounding down
        return X(if rounding() == Rounding::Down {
            -0.0
        } else {
            0.0
        });
    }
    let bb = r - a;
    directed(r, (a - (r - bb)) + (b - bb))
}

/// A product under a directed rounding.
#[cold]
fn mul_directed(a: f64, b: f64, r: f64) -> X {
    directed(r, a.mul_add(b, -r))
}

/// A quotient under a directed rounding.
#[cold]
fn div_directed(a: f64, b: f64, r: f64) -> X {
    // the remainder of the division has the sign of the error times the divisor's
    let rem = (-r).mul_add(b, a);
    directed(r, if b < 0.0 { -rem } else { rem })
}

impl Add for X {
    type Output = X;
    #[inline]
    fn add(self, o: X) -> X {
        let r = self.0 + o.0;
        if rounds_to_nearest() {
            return X(r);
        }
        add_directed(self.0, o.0, r)
    }
}
impl Sub for X {
    type Output = X;
    #[inline]
    fn sub(self, o: X) -> X {
        self + -o
    }
}
impl Mul for X {
    type Output = X;
    #[inline]
    fn mul(self, o: X) -> X {
        let r = self.0 * o.0;
        if rounds_to_nearest() {
            return X(r);
        }
        mul_directed(self.0, o.0, r)
    }
}
impl Div for X {
    type Output = X;
    #[inline]
    fn div(self, o: X) -> X {
        let r = self.0 / o.0;
        if rounds_to_nearest() {
            return X(r);
        }
        div_directed(self.0, o.0, r)
    }
}
impl AddAssign for X {
    #[inline]
    fn add_assign(&mut self, o: X) {
        *self = *self + o;
    }
}
impl SubAssign for X {
    #[inline]
    fn sub_assign(&mut self, o: X) {
        *self = *self - o;
    }
}
impl DivAssign for X {
    #[inline]
    fn div_assign(&mut self, o: X) {
        *self = *self / o;
    }
}
impl MulAssign for X {
    #[inline]
    fn mul_assign(&mut self, o: X) {
        *self = *self * o;
    }
}
impl Neg for X {
    type Output = X;
    fn neg(self) -> X {
        X(-self.0)
    }
}

/// The low part of `fldpi`'s 64-bit pi over `f64`'s `PI`, in units of 2^-62: the constant is
/// 0xC90FDAA22168C235 (one less when the rounding is down or chop).
fn pi_low() -> f64 {
    let units = match rounding() {
        Rounding::Down | Rounding::Chop => 564.0,
        _ => 565.0,
    };
    units * 2f64.powi(-62)
}

/// A sine or cosine of magnitude 1 that is not exactly 1 (the argument is not 0 for a
/// cosine): the x87 rounds the extended result by the control word, so towards zero (chop,
/// or down for a positive one, up for a negative one) it is one extended step short of 1;
/// libm's 1.0 is stepped one double toward zero, which every later 53-bit rounding sees as
/// the same value. Any other result is libm's.
fn toward_zero_below_one(r: f64, inexact: bool) -> f64 {
    if r.abs() != 1.0 || !inexact {
        return r;
    }
    let toward_zero = match rounding() {
        Rounding::Nearest => false,
        Rounding::Chop => true,
        Rounding::Down => r > 0.0,
        Rounding::Up => r < 0.0,
    };
    if toward_zero {
        if r > 0.0 { r.next_down() } else { r.next_up() }
    } else {
        r
    }
}

impl X {
    pub(crate) fn abs(self) -> X {
        X(self.0.abs())
    }

    /// `fprem` for a complete reduction: the remainder with the quotient truncated.
    pub(crate) fn fprem(self, o: X) -> X {
        X(self.0 % o.0)
    }

    /// `fsqrt`.
    pub(crate) fn sqrt(self) -> X {
        let r = self.0.sqrt();
        if rounding() == Rounding::Nearest {
            return X(r);
        }
        directed(r, (-r).mul_add(r, self.0))
    }

    /// `frndint`: rounded to a whole number as the control word says (exact).
    pub(crate) fn rint(self) -> X {
        X(crate::math::x87::round_f64(self.0))
    }

    /// `fsin` (libm's: within the last bit of the double, the same on every machine).
    pub(crate) fn sin(self) -> X {
        X(toward_zero_below_one(portable::sin(self.0), true))
    }

    /// `fcos` (libm's).
    pub(crate) fn cos(self) -> X {
        X(toward_zero_below_one(portable::cos(self.0), self.0 != 0.0))
    }

    /// `fsincos` (libm's): the sine and the cosine.
    pub(crate) fn sin_cos(self) -> (X, X) {
        (self.sin(), self.cos())
    }

    /// `fpatan` (libm's `atan2`): the angle of (`x`, `self`), `atan2(self, x)`.
    pub(crate) fn atan2(self, x: X) -> X {
        X(portable::atan2(self.0, x.0))
    }

    /// `fldpi; fmulp`: `self` times `fldpi`'s 64-bit pi, rounded once.
    pub(crate) fn times_pi(self) -> X {
        let r = self.0 * std::f64::consts::PI;
        if !r.is_finite() {
            return X(r);
        }
        let error = self.0.mul_add(std::f64::consts::PI, -r) + self.0 * pi_low();
        X(r) + X(error)
    }

    /// `fldpi; fdivrp`: `self` over `fldpi`'s 64-bit pi, rounded once.
    pub(crate) fn over_pi(self) -> X {
        let q = self.0 / std::f64::consts::PI;
        if !q.is_finite() {
            return X(q);
        }
        let rem = (-q).mul_add(std::f64::consts::PI, self.0);
        X(q) + X((rem - q * pi_low()) / std::f64::consts::PI)
    }

    /// `fldpi; fdiv`: `fldpi`'s 64-bit pi over `self`, rounded once.
    pub(crate) fn pi_over(self) -> X {
        let q = std::f64::consts::PI / self.0;
        if !q.is_finite() {
            return X(q);
        }
        let rem = (-q).mul_add(self.0, std::f64::consts::PI);
        X(q) + X((rem + pi_low()) / self.0)
    }
}

/// pi to 128 bits: 0xC90FDAA2_2168C234_C4C6628B_80DC1CD1 times 2^-126 (the digits that follow
/// are not all zero, so every rounding of a multiple of pi below knows it is above this).
const PI128: u128 = 0xC90F_DAA2_2168_C234_C4C6_628B_80DC_1CD1;

/// `n pi / 4` (n 1 ..= 4) as the x87 holds it after `fpatan` (for `negative`) or `fldpi`
/// (n = 4, positive): rounded to 64 bits as the control word says. Gives the 64-bit
/// significand and the exponent of its lowest bit.
fn quarter_pis(n: u32, negative: bool) -> (u128, i32) {
    let t = u128::from(n) * (PI128 >> 2);
    let lz = t.leading_zeros();
    let t = t << lz;
    let mut mant = t >> 64;
    let tail = t & ((1u128 << 64) - 1);
    // the exact value is above `t` (the digits cut off are not zero)
    let up = match rounding() {
        Rounding::Nearest => tail >= 1 << 63,
        Rounding::Chop => false,
        Rounding::Up => !negative,
        Rounding::Down => negative,
    };
    if up {
        mant += 1;
    }
    (mant, -60 - lz as i32)
}

/// `atan2(self, x) / pi` for the angles that are a whole number of quarter turns of
/// the half circle (the axes and the diagonals: `fpatan`'s result is `n pi / 4` rounded to
/// 64 bits, and `fldpi` is pi rounded to 64 bits, so the quotient is not exactly n / 4 when
/// the rounding directions differ); the angle's sign is `self`'s. Gives `None` for the others.
fn exact_angle_over_pi(y: f64, x: f64) -> Option<X> {
    let n = if y == 0.0 {
        if x < 0.0 { 4 } else { return Some(X(y)) }
    } else if x == 0.0 {
        2
    } else if x.abs() == y.abs() {
        if x > 0.0 { 1 } else { 3 }
    } else {
        return None;
    };
    let negative = y.is_sign_negative();
    let (me, ee) = quarter_pis(n, negative);
    let (mp, ep) = quarter_pis(4, false);
    // the quotient of the significands, to 63 fraction bits and a remainder
    let numerator = me << 63;
    let (q, rem) = (numerator / mp, numerator % mp);
    let drop = 128 - q.leading_zeros() - 53;
    let kept = q >> drop;
    let cut = q & ((1u128 << drop) - 1);
    let half = 1u128 << (drop - 1);
    let sticky = rem != 0;
    let inexact = cut != 0 || sticky;
    let up = match rounding() {
        Rounding::Nearest => cut > half || (cut == half && (sticky || kept & 1 == 1)),
        Rounding::Chop => false,
        Rounding::Up => inexact && !negative,
        Rounding::Down => inexact && negative,
    };
    let kept = kept + u128::from(up);
    let exponent = drop as i32 - 63 + ee - ep;
    let magnitude = (kept as f64) * 2f64.powi(exponent);
    Some(X(if negative { -magnitude } else { magnitude }))
}

impl X {
    /// `fpatan; fldpi; fdivp`: the angle of (`x`, `self`) over pi.
    pub(crate) fn atan2_over_pi(self, x: X) -> X {
        exact_angle_over_pi(self.0, x.0).unwrap_or_else(|| self.atan2(x).over_pi())
    }
}

/// `fld dword`: a float given as its bits.
pub(crate) fn ld(bits: u32) -> X {
    X(f64::from(f32::from_bits(bits)))
}

/// `fild dword`: a whole number.
pub(crate) fn int(i: i32) -> X {
    X(f64::from(i))
}

/// `fistp dword` (or `fist`) of `x`.
pub(crate) fn to_int(x: X) -> i32 {
    crate::math::x87::round_to_int(x.0)
}

/// `fstp dword`: the value rounded to a float as the control word says, as its bits.
pub(crate) fn st(x: X) -> u32 {
    let x = x.0;
    let n = x as f32;
    let mode = rounding();
    if mode == Rounding::Nearest || x.is_nan() {
        return n.to_bits();
    }
    let back = f64::from(n);
    if back == x {
        return n.to_bits();
    }
    let r = match mode {
        Rounding::Chop if back.abs() > x.abs() => {
            if n > 0.0 {
                n.next_down()
            } else {
                n.next_up()
            }
        }
        Rounding::Down if back > x => n.next_down(),
        Rounding::Up if back < x => n.next_up(),
        _ => n,
    };
    r.to_bits()
}

/// `fstp tbyte`: the value as the 80-bit extended format, little-endian (8 bytes of
/// significand, then sign and exponent). Exact for every double; a NaN comes out quiet.
pub(crate) fn f80(x: X) -> [u8; 10] {
    let bits = x.0.to_bits();
    let sign = ((bits >> 63) as u16) << 15;
    let e = ((bits >> 52) & 0x7FF) as i32;
    let frac = bits & 0x000F_FFFF_FFFF_FFFF;
    let (exp, mant): (u16, u64) = if e == 0x7FF {
        if frac == 0 {
            (0x7FFF, 0x8000_0000_0000_0000)
        } else {
            (0x7FFF, 0xC000_0000_0000_0000 | (frac << 11))
        }
    } else if e == 0 {
        if frac == 0 {
            (0, 0)
        } else {
            let lead = frac.leading_zeros() as i32 - 11; // shift that puts the top bit at bit 52
            let m = frac << (lead + 11);
            ((16383 - 1022 - lead) as u16, m)
        }
    } else {
        (
            (e - 1023 + 16383) as u16,
            0x8000_0000_0000_0000 | (frac << 11),
        )
    };
    let mut out = [0u8; 10];
    out[..8].copy_from_slice(&mant.to_le_bytes());
    out[8..].copy_from_slice(&(sign | exp).to_le_bytes());
    out
}

/// `fld tbyte`: the 80-bit extended value, rounded to nearest even to 53 bits where it
/// has more (exact for what `f80` made); out of the double's range it becomes an
/// infinity or a denormal.
pub(crate) fn from_f80(v: [u8; 10]) -> X {
    let mant = u64::from_le_bytes(v[..8].try_into().unwrap());
    let se = u16::from_le_bytes([v[8], v[9]]);
    let sign = u64::from(se >> 15) << 63;
    let e = i32::from(se & 0x7FFF);
    if e == 0x7FFF {
        if mant << 1 == 0 {
            return X(f64::from_bits(sign | (0x7FF << 52)));
        }
        return X(f64::from_bits(
            sign | (0x7FF << 52) | (1 << 51) | ((mant << 1) >> 12),
        ));
    }
    if mant == 0 {
        return X(f64::from_bits(sign));
    }
    let lz = mant.leading_zeros();
    let m = mant << lz;
    let exp2 = e.max(1) - 16383 - lz as i32;
    if exp2 > 1023 {
        return X(f64::from_bits(sign | (0x7FF << 52)));
    }
    let round = |value: u128, shift: u32| -> u64 {
        if shift > 127 {
            return 0;
        }
        let q = value >> shift;
        let rem = value & ((1u128 << shift) - 1);
        let half = 1u128 << (shift - 1);
        let up = rem > half || (rem == half && q & 1 == 1);
        (q + u128::from(up)) as u64
    };
    if exp2 >= -1022 {
        let frac = round(u128::from(m), 11);
        // a carry out of the significand (1 << 53) moves the exponent up by one
        let bits = (((exp2 + 1023) as u64) << 52) + (frac - (1 << 52));
        if bits >= 0x7FF << 52 {
            return X(f64::from_bits(sign | (0x7FF << 52)));
        }
        X(f64::from_bits(sign | bits))
    } else {
        let shift = 11 + (-1022 - exp2) as u32;
        X(f64::from_bits(sign | round(u128::from(m), shift)))
    }
}

/// `fcom` / `fcomp`: the status word's C3, C2 and C0 (0x4000, 0x400, 0x100) for `x`
/// against `y`: `x` above none, below C0, equal C3, no number all three.
pub(crate) fn compare(x: X, y: X) -> u16 {
    let (x, y) = (x.0, y.0);
    if x.is_nan() || y.is_nan() {
        0x4500
    } else if x > y {
        0
    } else if x < y {
        0x0100
    } else {
        0x4000
    }
}
