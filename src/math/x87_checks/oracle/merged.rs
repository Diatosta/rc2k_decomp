//! The helpers that were merged into one named function or replaced by a plain
//! expression, each against its old body (kept here), over seeded random inputs: float
//! bits of every kind (random bits, zeros, infinities, NaNs of both signs, denormals,
//! the largest and smallest finite values, whole numbers) and whole numbers up to the
//! ends of `i32`. Results compare bit for bit.

use super::super::inputs::{Rng, assert_same};
use crate::math::x87;
use crate::math::x87_float::{ONE, int, ld, st, to_int};

const CASES: usize = 2_000_000;

fn each(seed: u64, mut check: impl FnMut(&mut Rng)) {
    let mut rng = Rng::new(seed);
    for _ in 0..CASES {
        check(&mut rng);
    }
}

#[test]
fn from_int_and_int_to_float() {
    each(1, |r| {
        let a = r.whole();
        assert_same!(x87::from_int(a).to_bits(), st(int(a)));
    });
}

#[test]
fn int_ratio_for_the_three_old_names() {
    each(2, |r| {
        let (a, b) = (r.whole(), r.whole());
        let old = st(int(a) / int(b));
        assert_same!(x87::int_ratio(a, b).to_bits(), old, "{a} {b}");
    });
}

#[test]
fn int_over_for_the_old_int_div_and_int_over_float() {
    each(3, |r| {
        let (a, b) = (r.whole(), r.float());
        assert_same!(
            x87::int_over(a, b).to_bits(),
            st(int(a) / ld(b)),
            "{a} {b:x}"
        );
    });
}

#[test]
fn ext_less_for_the_old_ext_sub() {
    each(4, |r| {
        let (v, k) = (r.float(), r.float());
        let e = crate::math::x87_float::f80(ld(v));
        assert_same!(
            x87::ext_less(e, k).to_bits(),
            st(crate::math::x87_float::from_f80(e) - ld(k))
        );
    });
}

#[test]
fn toward_one_for_the_old_one_less_times_plus() {
    each(5, |r| {
        let (d, s) = (r.float(), r.float());
        let old = st((ONE - ld(d)) * ld(s) + ld(d));
        assert_same!(x87::toward_one(d, s).to_bits(), old, "{d:x} {s:x}");
    });
}

#[test]
fn add_to_int_of_the_two_old_modules() {
    each(6, |r| {
        let (a, b) = (r.float(), r.float());
        assert_eq!(x87::add_to_int(a, b), to_int(ld(a) + ld(b)));
    });
}

#[test]
fn sub_mul_of_the_two_old_modules() {
    each(7, |r| {
        let (a, b, c) = (r.float(), r.float(), r.float());
        let old = st((ld(a) - ld(b)) * ld(c));
        assert_same!(x87::sub_mul(a, b, c).to_bits(), old);
    });
}

#[test]
fn mul_for_the_old_square() {
    each(8, |r| {
        let a = r.float();
        assert_same!(x87::mul(a, a).to_bits(), st(ld(a) * ld(a)));
    });
}

#[test]
fn mul2_for_the_old_product3() {
    each(9, |r| {
        let (a, b, c) = (r.float(), r.float(), r.float());
        assert_same!(x87::mul2(a, b, c).to_bits(), st(ld(a) * ld(b) * ld(c)));
    });
}

#[test]
fn mul3_to_int_for_the_old_product4_to_int() {
    each(10, |r| {
        let (a, b, c, d) = (r.float(), r.float(), r.float(), r.float());
        let old = to_int(ld(a) * ld(b) * ld(c) * ld(d)) as u32;
        assert_eq!(x87::mul3_to_int(a, b, c, d) as u32, old);
    });
}

#[test]
fn minus_one_literal() {
    assert_same!((-1.0f32).to_bits(), st(crate::math::x87_float::X(-1.0)));
}

#[test]
fn negated_steps_for_the_old_negated_helpers() {
    each(11, |r| {
        let (a, b, c, d) = (r.float(), r.float(), r.float(), r.float());
        assert_same!(x87::negated(x87::sub(a, b)).to_bits(), st(-(ld(a) - ld(b))));
        assert_same!(x87::negated(x87::mul(a, b)).to_bits(), st(-(ld(a) * ld(b))));
        assert_same!(
            x87::negated(x87::mul2(a, b, c)).to_bits(),
            st(-(ld(a) * ld(b) * ld(c)))
        );
        assert_same!(
            x87::negated(x87::product_difference(a, b, c, d)).to_bits(),
            st(-(ld(a) * ld(b) - ld(c) * ld(d)))
        );
    });
}

/// Two float results are the same bits, or both a NaN: the random operands include NaNs,
/// and which NaN an operation gives is not fixed in Rust (from opt-level 1 the optimiser
/// folds the two forms' operations differently, so their NaNs differ in payload). The game
/// itself makes no NaN.
fn same(a: f32, b: f32) {
    if a.is_nan() && b.is_nan() {
        return;
    }
    assert_same!(a.to_bits(), b.to_bits());
}

/// The typed operands: a float, a cell and bits give the same bits.
#[test]
fn operands_as_bits_floats_and_cells_agree() {
    use crate::memory::session::Float;
    each(12, |r| {
        let (a, b, c, d) = (r.float(), r.float(), r.float(), r.float());
        let (fa, fb, fc, fd) = (
            f32::from_bits(a),
            f32::from_bits(b),
            f32::from_bits(c),
            f32::from_bits(d),
        );
        let (ca, cb, cc, cd) = (
            Float::from_bits(a),
            Float::from_bits(b),
            Float::from_bits(c),
            Float::from_bits(d),
        );
        same(x87::mul(a, b), x87::mul(fa, fb));
        same(x87::mul(a, b), x87::mul(&ca, &cb));
        same(x87::add(a, b), x87::add(&ca, fb));
        same(x87::sub(a, b), x87::sub(fa, &cb));
        same(x87::div(a, b), x87::div(&ca, &cb));
        same(x87::loaded(a), x87::loaded(&ca));
        same(x87::negated(a), x87::negated(fa));
        assert_eq!(x87::to_int(a), x87::to_int(&ca));
        assert_eq!(x87::mul_to_int(a, b), x87::mul_to_int(&ca, fb));
        same(x87::mul2(a, b, c), x87::mul2(&ca, &cb, &cc));
        same(x87::mul_add(a, b, c), x87::mul_add(fa, &cb, fc));
        same(
            x87::product_difference(a, b, c, d),
            x87::product_difference(&ca, fb, &cc, fd),
        );
        let _ = (cd, fd);
    });
}
