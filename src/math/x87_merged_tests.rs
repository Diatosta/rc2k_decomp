//! The helpers that were merged into one named function or replaced by a plain
//! expression, each against its old body (kept here), over seeded random inputs: float
//! bits of every kind (random bits, zeros, infinities, NaNs of both signs, denormals,
//! the largest and smallest finite values, whole numbers) and whole numbers up to the
//! ends of `i32`. Results compare bit for bit.

use super::x87;
use super::x87_float::{ONE, int, ld, st, to_int};

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn float(&mut self) -> u32 {
        const SPECIAL: [u32; 16] = [
            0x0000_0000,
            0x8000_0000,
            0x7F80_0000,
            0xFF80_0000,
            0x7FC0_0000,
            0xFFC0_0000,
            0x7F80_0001,
            0xFF80_0001,
            0x0000_0001,
            0x8000_0001,
            0x007F_FFFF,
            0x7F7F_FFFF,
            0xFF7F_FFFF,
            0x3F80_0000,
            0xBF80_0000,
            0x0080_0000,
        ];
        match self.next() % 4 {
            0 => SPECIAL[(self.next() % 16) as usize],
            1 => (self.next() as i32 % 100_000) as f32 as i32 as f32 as u32,
            _ => self.next() as u32,
        }
    }

    fn whole(&mut self) -> i32 {
        const EDGE: [i32; 6] = [0, 1, -1, i32::MIN, i32::MAX, 0x100];
        match self.next() % 3 {
            0 => EDGE[(self.next() % 6) as usize],
            1 => (self.next() % 2001) as i32 - 1000,
            _ => self.next() as i32,
        }
    }
}

const CASES: usize = 2_000_000;

fn each(seed: u64, mut check: impl FnMut(&mut Rng)) {
    let mut rng = Rng(seed);
    for _ in 0..CASES {
        check(&mut rng);
    }
}

#[test]
fn from_int_and_int_to_float() {
    each(1, |r| {
        let a = r.whole();
        assert_eq!(x87::from_int(a), st(int(a)));
    });
}

#[test]
fn int_ratio_for_the_three_old_names() {
    each(2, |r| {
        let (a, b) = (r.whole(), r.whole());
        let old = st(int(a) / int(b));
        assert_eq!(x87::int_ratio(a, b), old, "{a} {b}");
    });
}

#[test]
fn int_over_for_the_old_int_div_and_int_over_float() {
    each(3, |r| {
        let (a, b) = (r.whole(), r.float());
        assert_eq!(x87::int_over(a, b), st(int(a) / ld(b)), "{a} {b:x}");
    });
}

#[test]
fn ext_less_for_the_old_ext_sub() {
    each(4, |r| {
        let (v, k) = (r.float(), r.float());
        let e = crate::math::x87_float::f80(ld(v));
        assert_eq!(
            x87::ext_less(e, k),
            st(crate::math::x87_float::from_f80(e) - ld(k))
        );
    });
}

#[test]
fn toward_one_for_the_old_one_less_times_plus() {
    each(5, |r| {
        let (d, s) = (r.float(), r.float());
        let old = st((ONE - ld(d)) * ld(s) + ld(d));
        assert_eq!(x87::toward_one(d, s), old, "{d:x} {s:x}");
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
        assert_eq!(x87::sub_mul(a, b, c), old);
    });
}

#[test]
fn mul_for_the_old_square() {
    each(8, |r| {
        let a = r.float();
        assert_eq!(x87::mul(a, a), st(ld(a) * ld(a)));
    });
}

#[test]
fn mul2_for_the_old_product3() {
    each(9, |r| {
        let (a, b, c) = (r.float(), r.float(), r.float());
        assert_eq!(x87::mul2(a, b, c), st(ld(a) * ld(b) * ld(c)));
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
    assert_eq!((-1.0f32).to_bits(), st(crate::math::x87_float::X(-1.0)));
}

#[test]
fn negated_steps_for_the_old_negated_helpers() {
    each(11, |r| {
        let (a, b, c, d) = (r.float(), r.float(), r.float(), r.float());
        assert_eq!(x87::negated(x87::sub(a, b)), st(-(ld(a) - ld(b))));
        assert_eq!(x87::negated(x87::mul(a, b)), st(-(ld(a) * ld(b))));
        assert_eq!(
            x87::negated(x87::mul2(a, b, c)),
            st(-(ld(a) * ld(b) * ld(c)))
        );
        assert_eq!(
            x87::negated(x87::product_difference(a, b, c, d)),
            st(-(ld(a) * ld(b) - ld(c) * ld(d)))
        );
    });
}

/// The typed operands: a float, a cell and bits give the same bits.
#[test]
fn operands_as_bits_floats_and_cells_agree() {
    use crate::session::Float;
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
        assert_eq!(x87::mul(a, b), x87::mul(fa, fb));
        assert_eq!(x87::mul(a, b), x87::mul(&ca, &cb));
        assert_eq!(x87::add(a, b), x87::add(&ca, fb));
        assert_eq!(x87::sub(a, b), x87::sub(fa, &cb));
        assert_eq!(x87::div(a, b), x87::div(&ca, &cb));
        assert_eq!(x87::loaded(a), x87::loaded(&ca));
        assert_eq!(x87::negated(a), x87::negated(fa));
        assert_eq!(x87::to_int(a), x87::to_int(&ca));
        assert_eq!(x87::mul_to_int(a, b), x87::mul_to_int(&ca, fb));
        assert_eq!(x87::mul2(a, b, c), x87::mul2(&ca, &cb, &cc));
        assert_eq!(x87::mul_add(a, b, c), x87::mul_add(fa, &cb, fc));
        assert_eq!(
            x87::product_difference(a, b, c, d),
            x87::product_difference(&ca, fb, &cc, fd)
        );
        let _ = (cd, fd);
    });
}
