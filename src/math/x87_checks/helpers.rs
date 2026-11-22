//! The plain versions of the x87 helpers that production code no longer calls (inlined at
//! their callers, merged into another helper, or reached only through the `_with` forms),
//! kept for the recorded results and the oracle comparisons.
//! Each is the helper as it ran, rounding as the control word says.

use crate::math::x87::{
    Flt, mirrored_with, scaled_abs_less_below_with, turned_place_f32, turned_place_less_f32,
};
use crate::math::x87_float::{self as bh, *};

/// `fstp dword`: a result stored as a float, as the control word says.
fn out(x: bh::X) -> f32 {
    f32::from_bits(bh::st(x))
}

/// A float read from game memory (possibly unaligned), as the x87 holds it once loaded.
///
/// # Safety
/// `p` is valid for reads of the dword `index` dwords on (any alignment).
unsafe fn load(p: *const u32, index: usize) -> bh::X {
    // SAFETY: the caller's promise.
    bh::ld(unsafe { p.add(index).read_unaligned() })
}

/// `x` mirrored about a + b: `-(x - a - b) + b + a`.
pub fn mirrored(x: impl Flt, a: impl Flt, b: impl Flt) -> f32 {
    bh::at_rounding!(|r| mirrored_with(r, x, a, b))
}

/// `1 - a / b`.
pub(super) fn one_less_int_ratio(a: i32, b: i32) -> u32 {
    bh::st(bh::ONE - bh::int(a) / bh::int(b))
}

/// A row of one matrix times a column of another: `a[0] b[0] + (a[1] b[4] + a[2] b[8])`,
/// the last two products added first, stored to `out`.
///
/// # Safety
/// `a` is valid for reads of three dwords, `b` of nine, `out` for writes of one (any alignment).
pub unsafe fn row_times_column(a: *const u32, b: *const u32, out: *mut u32) {
    // SAFETY: dwords 0 .. 2 of `a` and 0, 4, 8 of `b`, the caller's promise.
    let (p1, p2, p3) = unsafe {
        (
            load(a, 0) * load(b, 0),
            load(a, 1) * load(b, 4),
            load(a, 2) * load(b, 8),
        )
    };
    // SAFETY: the caller's promise.
    unsafe { out.write_unaligned(bh::st(p1 + (p2 + p3))) };
}

/// `out = a + out`.
///
/// # Safety
/// `a` is valid for reads of one dword, `out` for reads and writes of one (any alignment).
pub unsafe fn add_into(a: *const u32, out: *mut u32) {
    // SAFETY: the caller's promise.
    let sum = unsafe { load(a, 0) + load(out, 0) };
    // SAFETY: the caller's promise.
    unsafe { out.write_unaligned(bh::st(sum)) };
}

/// The same negated.
pub(super) fn negated_product_difference(a: u32, b: u32, c: u32, d: u32) -> u32 {
    bh::st(-(bh::ld(a) * bh::ld(b) - bh::ld(c) * bh::ld(d)))
}

/// The length of three floats: `sqrt(v0 v0 + (v1 v1 + v2 v2))`.
///
/// # Safety
/// `v` is valid for reads of three dwords (any alignment).
pub unsafe fn length(v: *const u32) -> u32 {
    // SAFETY: the caller's promise.
    let (x, y, z) = unsafe { (load(v, 0), load(v, 1), load(v, 2)) };
    bh::st((x * x + (y * y + z * z)).sqrt())
}

/// The root of a whole number over a float.
pub(super) fn int_over_root(a: i32, d: u32) -> u32 {
    bh::st((bh::int(a) / bh::ld(d)).sqrt())
}

/// The sine of n times pi over d: `sin(n * pi / d)`.
pub(super) fn sine_of_pi_fraction(n: i32, d: i32) -> u32 {
    bh::st((bh::int(n).times_pi() / bh::int(d)).sin())
}

/// `a / b * m * n`, as a whole number.
pub(super) fn ratio_scaled_to_int(a: u32, b: u32, m: i32, n: i32) -> i32 {
    bh::to_int(bh::ld(a) / bh::ld(b) * bh::int(m) * bh::int(n))
}

/// Two whole numbers and two floats multiplied: `a * b * c * d`.
pub(super) fn int_float_int_float(a: i32, b: u32, c: i32, d: u32) -> u32 {
    bh::st(bh::int(a) * bh::ld(b) * bh::int(c) * bh::ld(d))
}

/// `race::add_surface_share`'s sum: `a / b * c / d * e + f`.
pub fn surface_share(a: i32, b: u32, c: u32, d: i32, e: u32, f: u32) -> u32 {
    bh::st(bh::int(a) / bh::ld(b) * bh::ld(c) / bh::int(d) * bh::ld(e) + bh::ld(f))
}

/// A whole number over a float, less a float, rounded once.
pub(super) fn int_div_less(a: i32, b: u32, c: u32) -> u32 {
    bh::st(bh::int(a) / bh::ld(b) - bh::ld(c))
}

/// `(a - b) / c * d`.
pub(super) fn sub_div_mul(a: u32, b: u32, c: u32, d: u32) -> u32 {
    bh::st((bh::ld(a) - bh::ld(b)) / bh::ld(c) * bh::ld(d))
}

/// `(a - b) / (c - d) * e`.
pub(super) fn diff_ratio_mul(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 {
    bh::st((bh::ld(a) - bh::ld(b)) / (bh::ld(c) - bh::ld(d)) * bh::ld(e))
}

/// `-(a - b)`.
pub(super) fn negated_sub(a: u32, b: u32) -> u32 {
    bh::st(-(bh::ld(a) - bh::ld(b)))
}

/// The difference of two whole numbers, rounded once to a float.
pub(super) fn int_less_int(a: i32, b: i32) -> u32 {
    bh::st(bh::int(a) - bh::int(b))
}

/// `(a - b) / c`, a and b whole numbers.
pub(super) fn int_less_int_over(a: i32, b: i32, c: u32) -> u32 {
    bh::st((bh::int(a) - bh::int(b)) / bh::ld(c))
}

/// `sin(a) b + c`, as a whole number.
pub(super) fn sine_times_plus_to_int(a: u32, b: u32, c: u32) -> i32 {
    bh::to_int(bh::ld(a).sin() * bh::ld(b) + bh::ld(c))
}

/// `a - n`, as a whole number.
pub(super) fn float_less_int_to_int(a: u32, n: i32) -> i32 {
    bh::to_int(bh::ld(a) - bh::int(n))
}

/// `pi / n`.
pub fn pi_over(n: i32) -> u32 {
    bh::st(bh::int(n).pi_over())
}

/// The sum, or 0x80000000 when it does not fit.
pub(super) fn int_sum(a: i32, b: i32) -> i32 {
    bh::to_int(bh::int(a) + bh::int(b))
}

/// `a + b`, a whole number and a float.
pub(super) fn int_plus(a: i32, b: u32) -> u32 {
    bh::st(bh::int(a) + bh::ld(b))
}

/// `a + b` as a whole number.
pub(super) fn int_plus_to_int(a: i32, b: u32) -> i32 {
    bh::to_int(bh::int(a) + bh::ld(b))
}

/// `f - a + b` as a whole number.
pub(super) fn less_int_plus_int(f: u32, a: i32, b: i32) -> i32 {
    bh::to_int(bh::ld(f) - bh::int(a) + bh::int(b))
}

/// `((a m) + c) k`.
pub(super) fn int_times_plus_times(a: i32, m: u32, c: u32, k: u32) -> u32 {
    bh::st((bh::int(a) * bh::ld(m) + bh::ld(c)) * bh::ld(k))
}

/// `a k n` as a whole number.
pub(super) fn times_int_to_int(a: u32, k: u32, n: i32) -> i32 {
    bh::to_int(bh::ld(a) * bh::ld(k) * bh::int(n))
}

/// `a b - c - d`.
pub(super) fn int_times_less_less(a: i32, b: u32, c: u32, d: u32) -> u32 {
    bh::st(bh::int(a) * bh::ld(b) - bh::ld(c) - bh::ld(d))
}

/// `(|a| + |b|) k` as a whole number.
pub(super) fn abs_sum_times_to_int(a: u32, b: u32, k: u32) -> i32 {
    bh::to_int((bh::ld(a).abs() + bh::ld(b).abs()) * bh::ld(k))
}

/// `turned_place_less_f32` on bits, for the frozen comparison.
pub(super) fn turned_place_less(
    t: [u32; 3],
    k: impl Flt,
    d: impl Flt,
    lift: impl Flt,
    m: &(impl crate::math::cells::Words + ?Sized),
) -> [u32; 4] {
    turned_place_less_f32(t.map(f32::from_bits), k, d, lift, m).map(f32::to_bits)
}

/// `a k / n`.
pub(super) fn times_over_int(a: u32, k: u32, n: i32) -> u32 {
    bh::st(bh::ld(a) * bh::ld(k) / bh::int(n))
}

/// `turned_place_f32` on bits, for the frozen comparison.
pub fn turned_place(
    p: [u32; 3],
    lift: impl Flt,
    m: &(impl crate::math::cells::Words + ?Sized),
) -> [u32; 4] {
    turned_place_f32(p.map(f32::from_bits), lift, m).map(f32::to_bits)
}

/// `a b` as a whole number.
pub(super) fn int_times_int_to_int(a: i32, b: i32) -> i32 {
    bh::to_int(bh::int(a) * bh::int(b))
}

/// `-a - b + c + d`.
pub(super) fn negated_less_plus_plus(a: u32, b: u32, c: u32, d: u32) -> u32 {
    bh::st(-bh::ld(a) - bh::ld(b) + bh::ld(c) + bh::ld(d))
}

/// `i a b`.
pub(super) fn int_times_times(i: i32, a: u32, b: u32) -> u32 {
    bh::st(bh::int(i) * bh::ld(a) * bh::ld(b))
}

/// `(b - a) k + c m`.
pub(super) fn diff_times_plus_times(b: u32, a: u32, k: u32, c: u32, m: u32) -> u32 {
    bh::st((bh::ld(b) - bh::ld(a)) * bh::ld(k) + bh::ld(c) * bh::ld(m))
}

/// `a + (v + w) t`.
pub(super) fn plus_sum_times_int(a: u32, v: u32, w: u32, t: i32) -> u32 {
    bh::st((bh::ld(v) + bh::ld(w)) * bh::int(t) + bh::ld(a))
}

/// `(r k1 + a) k2 + b`.
pub(super) fn int_times_plus_times_plus(r: i32, k1: u32, a: u32, k2: u32, b: u32) -> u32 {
    bh::st((bh::int(r) * bh::ld(k1) + bh::ld(a)) * bh::ld(k2) + bh::ld(b))
}

/// `c - v k` as a whole number.
pub(super) fn times_subr_to_int(v: u32, k: u32, c: u32) -> i32 {
    bh::to_int(bh::ld(c) - bh::ld(v) * bh::ld(k))
}

/// `k t + c` as a whole number.
pub(super) fn times_int_plus_int_to_int(k: u32, t: i32, c: i32) -> i32 {
    bh::to_int(bh::ld(k) * bh::int(t) + bh::int(c))
}

/// `v - v k t`.
pub fn damp(v: u32, k: u32, t: i32) -> u32 {
    bh::st(bh::ld(v) - bh::ld(v) * bh::ld(k) * bh::int(t))
}

/// `a - v t`.
pub(super) fn less_times_int(a: u32, v: u32, t: i32) -> u32 {
    bh::st(bh::ld(a) - bh::ld(v) * bh::int(t))
}

/// `a + b k` as a whole number.
pub(super) fn plus_product_to_int(a: u32, b: u32, k: u32) -> i32 {
    bh::to_int(bh::ld(a) + bh::ld(b) * bh::ld(k))
}

/// `i a b` as a whole number.
pub(super) fn int_times_times_to_int(i: i32, a: u32, b: u32) -> i32 {
    bh::to_int(bh::int(i) * bh::ld(a) * bh::ld(b))
}

/// `(a + b + c) k`.
pub(super) fn sum3_times(a: u32, b: u32, c: u32, k: u32) -> u32 {
    bh::st((bh::ld(a) + bh::ld(b) + bh::ld(c)) * bh::ld(k))
}

/// `(a * b) / c`, rounded once at the end as the x87 is set to round: a whole number times
/// a whole number over a float.
pub fn int_times_int_over_to_int(a: i32, b: i32, c: u32) -> i32 {
    to_int(int(a) * int(b) / ld(c))
}

/// `|a|` as a whole number: the size of a float.
pub fn abs_to_int(a: u32) -> i32 {
    to_int(ld(a).abs())
}

/// C0 of the compare of `|a h| - k` (kept at the x87's precision) with `b`: below, or one
/// is no number.
pub fn scaled_abs_less_below(a: impl Flt, h: impl Flt, k: impl Flt, b: impl Flt) -> bool {
    at_rounding!(|r| scaled_abs_less_below_with(r, a, h, k, b))
}

/// `a a + b b` of two whole numbers, as a float.
pub fn int_square_sum(a: i32, b: i32) -> f32 {
    let (a, b) = (int(a), int(b));
    out(a * a + b * b)
}

/// `a a + b b` of two floats.
pub fn float_square_sum(a: impl Flt, b: impl Flt) -> f32 {
    let a = a.value();
    let b = b.value();
    let a2 = ld(a) * ld(a);
    let b2 = ld(b) * ld(b);
    out(a2 + b2)
}

/// `(a + b) c`, a whole number plus a float, times a float.
pub fn int_plus_times(a: i32, b: impl Flt, c: impl Flt) -> f32 {
    let b = b.value();
    let c = c.value();
    out((int(a) + ld(b)) * ld(c))
}

/// C0 of the compare of `|a|` with `k`: |a| below k, or one is no number.
pub fn abs_below_or_unordered(a: impl Flt, k: impl Flt) -> bool {
    let a = a.value();
    let k = k.value();
    compare(ld(a).abs(), ld(k)) & 0x0100 != 0
}

/// `((a + b) h) / d`.
pub fn sum_times_over(a: u32, b: u32, h: u32, d: u32) -> u32 {
    st((ld(a) + ld(b)) * ld(h) / ld(d))
}

/// `((i a) b) / c` as a whole number.
pub fn int_times_times_over_to_int(i: i32, a: u32, b: u32, c: u32) -> i32 {
    to_int(int(i) * ld(a) * ld(b) / ld(c))
}

/// `((a b) c) d`, rounded once.
pub fn mul3(a: u32, b: u32, c: u32, d: u32) -> u32 {
    st(ld(a) * ld(b) * ld(c) * ld(d))
}

/// `(a - b) / c` as a whole number.
pub fn sub_div_to_int(a: u32, b: u32, c: u32) -> i32 {
    to_int((ld(a) - ld(b)) / ld(c))
}

/// `a / b + c`.
pub fn int_over_int_plus(a: i32, b: i32, c: u32) -> u32 {
    st(int(a) / int(b) + ld(c))
}

/// `(a + b) + c`.
pub fn add3(a: u32, b: u32, c: u32) -> u32 {
    st(ld(a) + ld(b) + ld(c))
}

/// `c` less `a b`.
pub fn neg_product_plus(a: u32, b: u32, c: u32) -> u32 {
    st(-(ld(a) * ld(b)) + ld(c))
}

/// `-(a b)`.
pub fn neg_mul(a: u32, b: u32) -> u32 {
    st(-(ld(a) * ld(b)))
}

/// `-((a b) c)`.
pub fn neg_mul2(a: u32, b: u32, c: u32) -> u32 {
    st(-(ld(a) * ld(b) * ld(c)))
}

/// `(((|x| / a) + b) c)`.
pub fn abs_div_add_mul(x: u32, a: u32, b: u32, c: u32) -> u32 {
    st((ld(x).abs() / ld(a) + ld(b)) * ld(c))
}

/// `|x| / n`, as a float.
pub fn abs_fidiv(x: u32, n: i32) -> u32 {
    st(ld(x).abs() / int(n))
}

/// `(a / b) c`.
pub fn div_then_mul(a: u32, b: u32, c: u32) -> u32 {
    st(ld(a) / ld(b) * ld(c))
}
