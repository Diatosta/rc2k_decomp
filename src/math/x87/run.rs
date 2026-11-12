//! More x87 helpers (see `math/x87.rs`), kept apart so that work on the race's run does not
//! edit the shared file.
//!
//! Plain Rust since phase 6 (2026-10-24): what the x87 holds on its stack is an `X` (an
//! `f64` that rounds as the control word says, `x87_float.rs`), what it stores as a dword
//! float `st`, `fild` is `int`, `fistp` / `fist` `to_int`. The 80-bit values some
//! functions carry between steps (`[u8; 10]`, the signatures stay) are `X` inside: at
//! double precision every step's result has 53 bits, so the conversion at the boundary is
//! exact. The order of the operations is the original's. The assembly versions are the
//! reference the tests compare with (`x87_reference_c.rs`).

use super::{Flt, out, set_chop, set_control};
use crate::math::x87_float::{ONE, compare, f80, from_f80, int, ld, st, to_int};

/// The length of three floats, `sqrt(v0 v0 + (v1 v1 + v2 v2))`, as an 80-bit value (not
/// rounded to a float).
pub fn length_80(v: [u32; 3]) -> [u8; 10] {
    let [a, b, c] = v.map(ld);
    let (a2, b2, c2) = (a * a, b * b, c * c);
    f80((a2 + (b2 + c2)).sqrt())
}

/// The three floats `v` each times the 80-bit `s` and stored as floats.
pub fn scale_by_80(v: [u32; 3], s: [u8; 10]) -> [u32; 3] {
    let s = from_f80(s);
    v.map(|part| st(ld(part) * s))
}

/// `((a + b) h) / d`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn sum_times_over(a: u32, b: u32, h: u32, d: u32) -> u32 {
    st((ld(a) + ld(b)) * ld(h) / ld(d))
}

/// `c + (a - c) f + (b - c) g` as a whole number, summed as `(a - c) f + (c + (b - c) g)`.
pub fn plane_height(a: i32, b: i32, c: i32, f: impl Flt, g: impl Flt) -> i32 {
    let f = f.value();
    let g = g.value();
    let (a, b, c) = (int(a), int(b), int(c));
    let p = (a - c) * ld(f);
    let q = (b - c) * ld(g);
    to_int(p + (q + c))
}

/// What a face's shares are worked out from: the corners' differences from the first corner,
/// whole numbers along x and along z, and the scale (`Numbers.k65536`).
pub struct FaceDifferences<S: Flt> {
    pub x: [i32; 3],
    pub z: [i32; 3],
    pub scale: S,
}

/// What `face_shares` leaves: the three cross products, the two quotients with the whole
/// numbers they give times the scale, and with `sse` the reciprocal that was used.
pub struct FaceShares {
    pub share_a: f32,
    pub share_b: f32,
    pub share_c: f32,
    pub plane_f: f32,
    pub share_s: i32,
    pub plane_g: f32,
    pub share_t: i32,
    pub share_scale: Option<f32>,
}

/// `face_shares_bits` on typed values.
pub fn face_shares<S: Flt>(d: &FaceDifferences<S>, sse: bool) -> FaceShares {
    let i = [
        d.x[0] as u32,
        d.x[1] as u32,
        d.x[2] as u32,
        d.z[0] as u32,
        d.z[1] as u32,
        d.z[2] as u32,
        d.scale.bits(),
    ];
    let mut o = [0u32; 8];
    face_shares_bits(&i, &mut o, sse);
    FaceShares {
        share_a: f32::from_bits(o[0]),
        share_b: f32::from_bits(o[1]),
        share_c: f32::from_bits(o[2]),
        plane_f: f32::from_bits(o[3]),
        share_s: o[4] as i32,
        plane_g: f32::from_bits(o[5]),
        share_t: o[6] as i32,
        share_scale: sse.then(|| f32::from_bits(o[7])),
    }
}

/// `physics::face_shares`' floating point part, the original's operations in its order.
/// `i`: the differences d738, d73C, d740, d750, d754, d758 (whole numbers) and the scale
/// `Numbers.k65536` (bits) at [6]. `o` gets f764, f75C, f760, f810, i800, f814, i804, and with
/// `sse` f768 (`rcpss` of f764) at [7]. (`rcpss` is the SSE approximation, from the tables
/// of `arch::rcpss`: the processors' own results differ.)
pub fn face_shares_bits(i: &[u32; 7], o: &mut [u32; 8], sse: bool) {
    let n = |k: usize| int(i[k] as i32);
    let scale = ld(i[6]);
    // The three cross products, each stored as a float.
    let x0 = n(1) * n(5) - n(2) * n(4);
    let x1 = n(0) * n(5) - n(2) * n(3);
    let x2 = n(1) * n(3) - n(0) * n(4);
    o[0] = st(x0);
    if !sse {
        o[1] = st(x1);
        o[2] = st(x2);
        // Divided by f764 as stored; the quotient stored, and times the scale, rounded.
        let q1 = ld(o[1]) / ld(o[0]);
        o[3] = st(q1);
        o[4] = to_int(q1 * scale) as u32;
        let q2 = ld(o[2]) / ld(o[0]);
        o[5] = st(q2);
        o[6] = to_int(q2 * scale) as u32;
    } else {
        let rcp = crate::arch::rcpss(f32::from_bits(o[0]));
        o[7] = rcp.to_bits();
        // The cross products times 1 over f764, unrounded until stored.
        o[1] = st(x1);
        let q1 = x1 * ld(o[7]);
        o[3] = st(q1);
        o[4] = to_int(q1 * scale) as u32;
        o[2] = st(x2);
        let q2 = x2 * ld(o[7]);
        o[5] = st(q2);
        o[6] = to_int(q2 * scale) as u32;
    }
}

/// `(a - b [+ c]) - d e`, left unrounded (80 bits).
pub fn less_plus_less_product_80(
    a: impl Flt,
    b: impl Flt,
    c: Option<impl Flt>,
    d: impl Flt,
    e: impl Flt,
) -> [u8; 10] {
    let a = a.value();
    let b = b.value();
    let d = d.value();
    let e = e.value();
    let mut x = ld(a) - ld(b);
    if let Some(c) = c {
        x += ld(c.value());
    }
    f80(x - ld(d) * ld(e))
}

/// `((i a) b) / c` as a whole number.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn int_times_times_over_to_int(i: i32, a: u32, b: u32, c: u32) -> i32 {
    to_int(int(i) * ld(a) * ld(b) / ld(c))
}

/// `(a a k + c) g` and its negative, each rounded to a float.
pub fn square_times_plus_times(a: impl Flt, k: impl Flt, c: impl Flt, g: impl Flt) -> (f32, f32) {
    let a = a.value();
    let k = k.value();
    let c = c.value();
    let g = g.value();
    let x = (ld(a) * ld(a) * ld(k) + ld(c)) * ld(g);
    (out(x), out(-x))
}

/// The compare of `a` with `b`: the status word's C0 .. C3 (0x4700).
pub fn compare_full(a: impl Flt, b: impl Flt) -> u16 {
    let a = a.value();
    let b = b.value();
    compare(ld(a), ld(b)) & 0x4700
}

/// 1 less the products of the terms, `acc = 1; acc -= (a b) c` for each (a, b, c),
/// unrounded (80 bits; every step has 53 bits, so the carry between the terms is exact).
pub fn one_less_products<T: Flt>(terms: &[[T; 3]]) -> [u8; 10] {
    let mut acc = ONE;
    for t in terms {
        acc -= ld(t[0].value()) * ld(t[1].value()) * ld(t[2].value());
    }
    f80(acc)
}

/// The 80-bit `e` times `k`, unrounded.
pub fn ext_times(e: [u8; 10], k: impl Flt) -> [u8; 10] {
    let k = k.value();
    f80(from_f80(e) * ld(k))
}

/// A ramp through (t0, t1) .. (t2, t3) as the original reads it: below t0 (or no number)
/// t1; not below t2, t3; t2 - t0 equal to `zero` (or no number), t1; else
/// `(x - t0) / (t2 - t0) (t3 - t1) + t1`. Rounded to a float (the `fstp`).
pub fn ramp<T: Flt>(x: impl Flt, t: [T; 4], zero: impl Flt) -> f32 {
    let t = t.map(|v| v.value());
    let x = x.value();
    let zero = zero.value();
    if compare_full(x, t[0]) & 0x100 != 0 {
        return crate::math::x87::loaded(t[1]);
    }
    if compare_full(x, t[2]) & 0x100 == 0 {
        return crate::math::x87::loaded(t[3]);
    }
    let span = ld(t[2]) - ld(t[0]);
    if compare(span, ld(zero)) & 0x4000 != 0 {
        return crate::math::x87::loaded(t[1]);
    }
    out((ld(x) - ld(t[0])) / span * (ld(t[3]) - ld(t[1])) + ld(t[1]))
}

/// `((a b) c) d`, rounded once.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn mul3(a: u32, b: u32, c: u32, d: u32) -> u32 {
    st(ld(a) * ld(b) * ld(c) * ld(d))
}

/// `lo + (i / d) (hi - lo)`, the products in that order.
pub fn int_lerp(i: i32, d: impl Flt, hi: impl Flt, lo: impl Flt) -> f32 {
    let d = d.value();
    let hi = hi.value();
    let lo = lo.value();
    out(int(i) / ld(d) * (ld(hi) - ld(lo)) + ld(lo))
}

/// `(a - b) / c` as a whole number.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn sub_div_to_int(a: u32, b: u32, c: u32) -> i32 {
    to_int((ld(a) - ld(b)) / ld(c))
}

/// A whole number as an 80-bit value; with `negate`, its negative.
pub fn int_80(a: i32, negate: bool) -> [u8; 10] {
    let x = int(a);
    f80(if negate { -x } else { x })
}

/// The 80-bit `e` as a whole number.
pub fn ext_to_int(e: [u8; 10]) -> i32 {
    to_int(from_f80(e))
}

/// `a / b + c`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn int_over_int_plus(a: i32, b: i32, c: u32) -> u32 {
    st(int(a) / int(b) + ld(c))
}

/// (sine, cosine) of the float `a`, rounded to floats (`fsincos`: libm's `sin` / `cos`
/// within an ulp of the instruction's).
pub fn sine_cosine_of(a: impl Flt) -> (u32, u32) {
    let a = a.value();
    let (s, c) = ld(a).sin_cos();
    (st(s), st(c))
}

/// `1 - a`.
pub fn one_less(a: impl Flt) -> f32 {
    let a = a.value();
    out(ONE - ld(a))
}

/// `sqrt(|x|) t + c`.
pub fn abs_sqrt_times_plus(x: impl Flt, t: impl Flt, c: impl Flt) -> f32 {
    let x = x.value();
    let t = t.value();
    let c = c.value();
    out(ld(x).abs().sqrt() * ld(t) + ld(c))
}

/// `(a + b) + c`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn add3(a: u32, b: u32, c: u32) -> u32 {
    st(ld(a) + ld(b) + ld(c))
}

/// `(a b) c` as a whole number.
pub fn mul2_to_int(a: impl Flt, b: impl Flt, c: impl Flt) -> i32 {
    let a = a.value();
    let b = b.value();
    let c = c.value();
    to_int(ld(a) * ld(b) * ld(c))
}

/// `c` less `a b`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn neg_product_plus(a: u32, b: u32, c: u32) -> u32 {
    st(-(ld(a) * ld(b)) + ld(c))
}

/// `-(a b)`.
#[cfg(all(test, target_arch = "x86"))] // replaced by `negated` of the plain step; the assembly comparison keeps it
pub fn neg_mul(a: u32, b: u32) -> u32 {
    st(-(ld(a) * ld(b)))
}

/// The compare of `|a|` with `b`: C0 .. C3 (0x4700).
pub fn abs_compare(a: impl Flt, b: impl Flt) -> u16 {
    let a = a.value();
    let b = b.value();
    compare(ld(a).abs(), ld(b)) & 0x4700
}

/// `|a|`, as a float.
pub fn abs_of(a: impl Flt) -> f32 {
    let a = a.value();
    out(ld(a).abs())
}

/// The length of two floats at right angles, as a whole number.
pub fn float_hypot_to_int(a: impl Flt, b: impl Flt) -> i32 {
    let a = a.value();
    let b = b.value();
    to_int((ld(a) * ld(a) + ld(b) * ld(b)).sqrt())
}

/// `a + (i - a) k` as a whole number.
pub fn int_toward_to_int(i: i32, a: impl Flt, k: impl Flt) -> i32 {
    let a = a.value();
    let k = k.value();
    to_int((int(i) - ld(a)) * ld(k) + ld(a))
}

/// The product `a k` as a whole number (`fist`), and the compare's C0 .. C3 (0x4700) of
/// the product (unrounded) against z.
pub fn times_fist_compare(a: impl Flt, k: impl Flt, z: impl Flt) -> (i32, u16) {
    let a = a.value();
    let k = k.value();
    let z = z.value();
    let p = ld(a) * ld(k);
    (to_int(p), compare(p, ld(z)) & 0x4700)
}

/// `((((a / b) c) d) e)` as a whole number.
pub fn div_mul3_to_int(a: impl Flt, b: impl Flt, c: impl Flt, d: impl Flt, e: impl Flt) -> i32 {
    let a = a.value();
    let b = b.value();
    let c = c.value();
    let d = d.value();
    let e = e.value();
    to_int(ld(a) / ld(b) * ld(c) * ld(d) * ld(e))
}

/// The 80-bit `e` over `a` over `b`, as a float.
pub fn ext_div2(e: [u8; 10], a: impl Flt, b: impl Flt) -> f32 {
    let a = a.value();
    let b = b.value();
    out(from_f80(e) / ld(a) / ld(b))
}

/// The terms' sum over the divisors: `t0 + t1 + ..`, then `/ d0 / d1 ..`, as a float.
pub fn sum_over<T: Flt, D: Flt>(terms: &[T], divisors: &[D]) -> f32 {
    let mut acc = ld(terms[0].value());
    for t in &terms[1..] {
        acc += ld(t.value());
    }
    for d in divisors {
        acc /= ld(d.value());
    }
    out(acc)
}

/// `(x x k) / d`.
pub fn square_times_over(x: impl Flt, k: impl Flt, d: impl Flt) -> f32 {
    let x = x.value();
    let k = k.value();
    let d = d.value();
    out(ld(x) * ld(x) * ld(k) / ld(d))
}

/// `a + (1 - a) b`.
pub fn toward_one(a: impl Flt, b: impl Flt) -> f32 {
    let a = a.value();
    let b = b.value();
    out((ONE - ld(a)) * ld(b) + ld(a))
}

/// `((((a - b) k) i) + b) / d / w`.
pub fn lerp_int_over2(
    a: impl Flt,
    b: impl Flt,
    k: impl Flt,
    i: i32,
    d: impl Flt,
    w: impl Flt,
) -> f32 {
    let a = a.value();
    let b = b.value();
    let k = k.value();
    let d = d.value();
    let w = w.value();
    out(((ld(a) - ld(b)) * ld(k) * int(i) + ld(b)) / ld(d) / ld(w))
}

/// The step between gears, `(top - low) / (n - one)`, unrounded.
pub fn gear_step_80(top: impl Flt, low: impl Flt, n: i32, one: impl Flt) -> [u8; 10] {
    let top = top.value();
    let low = low.value();
    let one = one.value();
    f80((ld(top) - ld(low)) / (int(n) - ld(one)))
}

/// The sum of two 80-bit values, unrounded.
pub fn ext_add(a: [u8; 10], b: [u8; 10]) -> [u8; 10] {
    f80(from_f80(a) + from_f80(b))
}

/// `-((((v - one) a) + one) g m) a`... in the original's order: `(v - one) a + one`, times
/// `g`, times `m`, negated, times `a`.
pub fn weight_force(v: impl Flt, one: impl Flt, a: impl Flt, g: impl Flt, m: impl Flt) -> f32 {
    let v = v.value();
    let one = one.value();
    let a = a.value();
    let g = g.value();
    let m = m.value();
    let x = ((ld(v) - ld(one)) * ld(a) + ld(one)) * ld(g) * ld(m);
    out(-x * ld(a))
}

/// `d` less `a b c`.
pub fn neg_product2_plus(a: impl Flt, b: impl Flt, c: impl Flt, d: impl Flt) -> f32 {
    let a = a.value();
    let b = b.value();
    let c = c.value();
    let d = d.value();
    out(-(ld(a) * ld(b) * ld(c)) + ld(d))
}

/// `physics::clamped_asin`'s body (0x42120B) on the 80-bit `x`, its scratch float at `s`
/// and its limit at `l` (negated and put back on the way, as the original does): the
/// angle whose sine is `x`, by `atan2` of `x` over the root of 1 less the square of `x`
/// stored as a float and held to -limit ..= limit (`fpatan`: libm's `atan2` within 2 ulp
/// of the instruction's). A compare that finds the values unordered counts as "below".
pub fn clamped_asin_80(x: [u8; 10], s: &mut u32, l: &mut u32) -> [u8; 10] {
    let x = from_f80(x);
    let limit = ld(*l);
    let mut value = st(x);
    // Not below the limit (and a number): the limit.
    if compare(ld(value), limit) & 0x100 == 0 {
        value = st(limit);
    }
    // The limit negated (stored as a float), the value below it or unordered: it.
    let minus = st(-limit);
    if compare(ld(value), ld(minus)) & 0x100 != 0 {
        value = st(ld(minus));
    }
    *s = value;
    *l = st(-ld(minus));
    let v = ld(value);
    f80(x.atan2((ONE - v * v).sqrt()))
}

/// The angle of (x, y), `atan2(y, x)`, as a float (`fpatan`: within an ulp of the
/// instruction's).
pub fn patan(y: impl Flt, x: impl Flt) -> f32 {
    let y = y.value();
    let x = x.value();
    out(ld(y).atan2(ld(x)))
}

/// `a / cos b`, unrounded (`fcos`: libm's `cos`, the quotient within 2 ulp).
pub fn over_cos_80(a: impl Flt, b: impl Flt) -> [u8; 10] {
    let a = a.value();
    let b = b.value();
    f80(ld(a) / ld(b).cos())
}

/// `((a b) c) d` as a whole number.
pub fn mul3_to_int(a: impl Flt, b: impl Flt, c: impl Flt, d: impl Flt) -> i32 {
    let a = a.value();
    let b = b.value();
    let c = c.value();
    let d = d.value();
    to_int(ld(a) * ld(b) * ld(c) * ld(d))
}

/// The negative of the 80-bit `x`, rounded to a float.
pub fn neg_rounded(x: [u8; 10]) -> f32 {
    out(-from_f80(x))
}

/// `physics::lever_pushes`' share (0x43DE23 .. 0x43DEAC): A = |a| + |d|, B = |b| + |c|,
/// their sum B + A compared with `zero`; equal or unordered: returns A rounded and true;
/// else A / (B + A) rounded and false.
pub fn lever_share(
    a: impl Flt,
    d: impl Flt,
    b: impl Flt,
    c: impl Flt,
    zero: impl Flt,
) -> (f32, bool) {
    let a = a.value();
    let d = d.value();
    let b = b.value();
    let c = c.value();
    let zero = zero.value();
    let big_a = ld(a).abs() + ld(d).abs();
    let big_b = ld(b).abs() + ld(c).abs();
    let sum = big_b + big_a;
    if compare(sum, ld(zero)) & 0x4000 != 0 {
        (out(big_a), true)
    } else {
        (out(big_a / sum), false)
    }
}

/// `-((a b) c)`.
#[cfg(all(test, target_arch = "x86"))] // replaced by `negated` of the plain step; the assembly comparison keeps it
pub fn neg_mul2(a: u32, b: u32, c: u32) -> u32 {
    st(-(ld(a) * ld(b) * ld(c)))
}

/// `(((a - b) + c) + (x - d) e) f` as a whole number (`v` = a, b, c, x, d, e, f).
pub fn wheel_int<T: Flt>(v: [T; 7]) -> i32 {
    let v = v.map(|x| x.value());
    let p = ld(v[0]) - ld(v[1]) + ld(v[2]);
    let q = (ld(v[3]) - ld(v[4])) * ld(v[5]);
    to_int((p + q) * ld(v[6]))
}

/// `(((|x| / a) + b) c)`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn abs_div_add_mul(x: u32, a: u32, b: u32, c: u32) -> u32 {
    st((ld(x).abs() / ld(a) + ld(b)) * ld(c))
}

/// `(a b) c + d`.
pub fn mul2_plus(a: impl Flt, b: impl Flt, c: impl Flt, d: impl Flt) -> f32 {
    let a = a.value();
    let b = b.value();
    let c = c.value();
    let d = d.value();
    out(ld(a) * ld(b) * ld(c) + ld(d))
}

/// `a / (|s| + b) - c`.
pub fn over_abs_plus_minus(a: impl Flt, s: impl Flt, b: impl Flt, c: impl Flt) -> f32 {
    let a = a.value();
    let s = s.value();
    let b = b.value();
    let c = c.value();
    out(ld(a) / (ld(s).abs() + ld(b)) - ld(c))
}

/// `a - b` of two whole numbers as an 80-bit value: exact.
pub fn ints_diff_80(a: i32, b: i32) -> [u8; 10] {
    f80(int(a) - int(b))
}

/// `|x| / n`, as a float.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn abs_fidiv(x: u32, n: i32) -> u32 {
    st(ld(x).abs() / int(n))
}

/// `(a - b) c + d`.
pub fn sub_mul_add(a: impl Flt, b: impl Flt, c: impl Flt, d: impl Flt) -> f32 {
    let a = a.value();
    let b = b.value();
    let c = c.value();
    let d = d.value();
    out((ld(a) - ld(b)) * ld(c) + ld(d))
}

/// The compare of `(a b) / c` with `k`: C0 .. C3 (0x4700).
pub fn mul_div_status(a: impl Flt, b: impl Flt, c: impl Flt, k: impl Flt) -> u16 {
    let a = a.value();
    let b = b.value();
    let c = c.value();
    let k = k.value();
    compare(ld(a) * ld(b) / ld(c), ld(k)) & 0x4700
}

/// The compare of `a b` with `c`: C0 .. C3 (0x4700).
pub fn mul_status(a: impl Flt, b: impl Flt, c: impl Flt) -> u16 {
    let a = a.value();
    let b = b.value();
    let c = c.value();
    compare(ld(a) * ld(b), ld(c)) & 0x4700
}

/// `a b` as a whole number with the rounding set to chop (the control word put back
/// after), the precision as it was.
pub fn trunc_mul_to_int(a: impl Flt, b: impl Flt) -> i32 {
    let a = a.value();
    let b = b.value();
    let old = set_chop();
    let out = to_int(ld(a) * ld(b));
    set_control(old);
    out
}

/// The table line between (a0, c0) and (a1, c1) at s: `(s - a0) / (a1 - a0) (c1 - c0) +
/// c0`.
pub fn lerp_table(s: impl Flt, a0: impl Flt, a1: impl Flt, c0: impl Flt, c1: impl Flt) -> f32 {
    let s = s.value();
    let a0 = a0.value();
    let a1 = a1.value();
    let c0 = c0.value();
    let c1 = c1.value();
    out((ld(s) - ld(a0)) / (ld(a1) - ld(a0)) * (ld(c1) - ld(c0)) + ld(c0))
}

/// `(a / b) c`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn div_then_mul(a: u32, b: u32, c: u32) -> u32 {
    st(ld(a) / ld(b) * ld(c))
}

/// `((a + b) n) / k - b`.
pub fn sum_times_int_over_minus(a: impl Flt, b: impl Flt, n: i32, k: impl Flt) -> f32 {
    let a = a.value();
    let b = b.value();
    let k = k.value();
    out((ld(a) + ld(b)) * int(n) / ld(k) - ld(b))
}

/// `((a b) / c) / d`.
pub fn mul_div2(a: impl Flt, b: impl Flt, c: impl Flt, d: impl Flt) -> f32 {
    let a = a.value();
    let b = b.value();
    let c = c.value();
    let d = d.value();
    out(ld(a) * ld(b) / ld(c) / ld(d))
}

/// `i k + a [+ b]`.
pub fn int_mul_add(i: i32, k: impl Flt, a: impl Flt, b: Option<u32>) -> f32 {
    let k = k.value();
    let a = a.value();
    let mut x = int(i) * ld(k) + ld(a);
    if let Some(b) = b {
        x += ld(b);
    }
    out(x)
}

/// `a / b` as an 80-bit value.
pub fn div_80(a: impl Flt, b: impl Flt) -> [u8; 10] {
    let a = a.value();
    let b = b.value();
    f80(ld(a) / ld(b))
}

/// The compare of the 80-bit `e` with a float: C0 .. C3 (0x4700).
pub fn compare_80(e: [u8; 10], b: impl Flt) -> u16 {
    let b = b.value();
    compare(from_f80(e), ld(b)) & 0x4700
}
