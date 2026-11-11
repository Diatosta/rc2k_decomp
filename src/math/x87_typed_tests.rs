//! The helpers whose floats became `f32` values (inputs as `impl Flt`, results as `f32`),
//! each against its old body (`x87_old`, results as bits), over seeded random inputs: float
//! bits of every kind (random bits, zeros, infinities, NaNs of both signs with their payloads,
//! denormals, the largest and smallest finite values, whole numbers) and whole numbers up to
//! the ends of `i32`. Inputs go in as bits and as `f32`, results compare bit for bit, at
//! each of the four rounding modes.

use super::x87;
use super::x87_merged_tests::Rng;
use super::x87_old as old;

const CASES: usize = 20_000;

/// Runs `check` at every rounding mode, `CASES` times each, on a fixed seed.
fn each(seed: u64, mut check: impl FnMut(&mut Rng)) {
    let word = x87::control_word();
    for mode in 0..4u16 {
        x87::set_control_word(word & 0xF3FF | mode << 10);
        let mut rng = Rng::new(seed);
        for _ in 0..CASES {
            check(&mut rng);
        }
    }
    x87::set_control_word(word);
}

#[test]
fn add() {
    each(1, |rng| {
        let a = rng.float();
        let b = rng.float();
        let old = old::add(a, b);
        assert_eq!(x87::add(a, b).to_bits(), old, "add {:x?}", (a, b));
        assert_eq!(
            x87::add(f32::from_bits(a), f32::from_bits(b)).to_bits(),
            old,
            "add (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn sub() {
    each(2, |rng| {
        let a = rng.float();
        let b = rng.float();
        let old = old::sub(a, b);
        assert_eq!(x87::sub(a, b).to_bits(), old, "sub {:x?}", (a, b));
        assert_eq!(
            x87::sub(f32::from_bits(a), f32::from_bits(b)).to_bits(),
            old,
            "sub (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn from_int() {
    each(3, |rng| {
        let a = rng.whole();
        let old = old::from_int(a);
        assert_eq!(x87::from_int(a).to_bits(), old, "from_int {:x?}", (a,));
    });
}

#[test]
fn mul() {
    each(4, |rng| {
        let a = rng.float();
        let b = rng.float();
        let old = old::mul(a, b);
        assert_eq!(x87::mul(a, b).to_bits(), old, "mul {:x?}", (a, b));
        assert_eq!(
            x87::mul(f32::from_bits(a), f32::from_bits(b)).to_bits(),
            old,
            "mul (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn reciprocal_times() {
    each(5, |rng| {
        let a = rng.float();
        let b = rng.float();
        let old = old::reciprocal_times(a, b);
        assert_eq!(
            x87::reciprocal_times(a, b).to_bits(),
            old,
            "reciprocal_times {:x?}",
            (a, b)
        );
        assert_eq!(
            x87::reciprocal_times(f32::from_bits(a), f32::from_bits(b)).to_bits(),
            old,
            "reciprocal_times (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn mirrored() {
    each(6, |rng| {
        let x = rng.float();
        let a = rng.float();
        let b = rng.float();
        let old = old::mirrored(x, a, b);
        assert_eq!(
            x87::mirrored(x, a, b).to_bits(),
            old,
            "mirrored {:x?}",
            (x, a, b)
        );
        assert_eq!(
            x87::mirrored(f32::from_bits(x), f32::from_bits(a), f32::from_bits(b)).to_bits(),
            old,
            "mirrored (f32 inputs) {:x?}",
            (x, a, b)
        );
    });
}

#[test]
fn loaded() {
    each(7, |rng| {
        let a = rng.float();
        let old = old::loaded(a);
        assert_eq!(x87::loaded(a).to_bits(), old, "loaded {:x?}", (a,));
        assert_eq!(
            x87::loaded(f32::from_bits(a)).to_bits(),
            old,
            "loaded (f32 inputs) {:x?}",
            (a,)
        );
    });
}

#[test]
fn int_ratio() {
    each(8, |rng| {
        let a = rng.whole();
        let b = rng.whole();
        let old = old::int_ratio(a, b);
        assert_eq!(
            x87::int_ratio(a, b).to_bits(),
            old,
            "int_ratio {:x?}",
            (a, b)
        );
    });
}

#[test]
fn int_over_16() {
    each(9, |rng| {
        let a = rng.whole();
        let old = old::int_over_16(a);
        assert_eq!(
            x87::int_over_16(a).to_bits(),
            old,
            "int_over_16 {:x?}",
            (a,)
        );
    });
}

#[test]
fn div() {
    each(10, |rng| {
        let a = rng.float();
        let b = rng.float();
        let old = old::div(a, b);
        assert_eq!(x87::div(a, b).to_bits(), old, "div {:x?}", (a, b));
        assert_eq!(
            x87::div(f32::from_bits(a), f32::from_bits(b)).to_bits(),
            old,
            "div (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn product_difference() {
    each(11, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let d = rng.float();
        let old = old::product_difference(a, b, c, d);
        assert_eq!(
            x87::product_difference(a, b, c, d).to_bits(),
            old,
            "product_difference {:x?}",
            (a, b, c, d)
        );
        assert_eq!(
            x87::product_difference(
                f32::from_bits(a),
                f32::from_bits(b),
                f32::from_bits(c),
                f32::from_bits(d)
            )
            .to_bits(),
            old,
            "product_difference (f32 inputs) {:x?}",
            (a, b, c, d)
        );
    });
}

#[test]
fn equal_or_unordered() {
    each(12, |rng| {
        let a = rng.float();
        let b = rng.float();
        let old = old::equal_or_unordered(a, b);
        assert_eq!(
            x87::equal_or_unordered(a, b),
            old,
            "equal_or_unordered {:x?}",
            (a, b)
        );
        assert_eq!(
            x87::equal_or_unordered(f32::from_bits(a), f32::from_bits(b)),
            old,
            "equal_or_unordered (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn sqrt() {
    each(13, |rng| {
        let a = rng.float();
        let old = old::sqrt(a);
        assert_eq!(x87::sqrt(a).to_bits(), old, "sqrt {:x?}", (a,));
        assert_eq!(
            x87::sqrt(f32::from_bits(a)).to_bits(),
            old,
            "sqrt (f32 inputs) {:x?}",
            (a,)
        );
    });
}

#[test]
fn angle_scaled_to_int() {
    each(14, |rng| {
        let y = rng.whole();
        let x = rng.whole();
        let a = rng.float();
        let b = rng.float();
        let old = old::angle_scaled_to_int(y, x, a, b);
        assert_eq!(
            x87::angle_scaled_to_int(y, x, a, b),
            old,
            "angle_scaled_to_int {:x?}",
            (y, x, a, b)
        );
        assert_eq!(
            x87::angle_scaled_to_int(y, x, f32::from_bits(a), f32::from_bits(b)),
            old,
            "angle_scaled_to_int (f32 inputs) {:x?}",
            (y, x, a, b)
        );
    });
}

#[test]
fn int_times() {
    each(15, |rng| {
        let a = rng.whole();
        let b = rng.float();
        let old = old::int_times(a, b);
        assert_eq!(
            x87::int_times(a, b).to_bits(),
            old,
            "int_times {:x?}",
            (a, b)
        );
        assert_eq!(
            x87::int_times(a, f32::from_bits(b)).to_bits(),
            old,
            "int_times (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn int_over() {
    each(16, |rng| {
        let a = rng.whole();
        let b = rng.float();
        let old = old::int_over(a, b);
        assert_eq!(x87::int_over(a, b).to_bits(), old, "int_over {:x?}", (a, b));
        assert_eq!(
            x87::int_over(a, f32::from_bits(b)).to_bits(),
            old,
            "int_over (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn int_less() {
    each(17, |rng| {
        let a = rng.whole();
        let b = rng.float();
        let old = old::int_less(a, b);
        assert_eq!(x87::int_less(a, b).to_bits(), old, "int_less {:x?}", (a, b));
        assert_eq!(
            x87::int_less(a, f32::from_bits(b)).to_bits(),
            old,
            "int_less (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn reciprocal() {
    each(18, |rng| {
        let a = rng.float();
        let old = old::reciprocal(a);
        assert_eq!(x87::reciprocal(a).to_bits(), old, "reciprocal {:x?}", (a,));
        assert_eq!(
            x87::reciprocal(f32::from_bits(a)).to_bits(),
            old,
            "reciprocal (f32 inputs) {:x?}",
            (a,)
        );
    });
}

#[test]
fn mul_div_div_to_int() {
    each(19, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let d = rng.float();
        let old = old::mul_div_div_to_int(a, b, c, d);
        assert_eq!(
            x87::mul_div_div_to_int(a, b, c, d),
            old,
            "mul_div_div_to_int {:x?}",
            (a, b, c, d)
        );
        assert_eq!(
            x87::mul_div_div_to_int(
                f32::from_bits(a),
                f32::from_bits(b),
                f32::from_bits(c),
                f32::from_bits(d)
            ),
            old,
            "mul_div_div_to_int (f32 inputs) {:x?}",
            (a, b, c, d)
        );
    });
}

#[test]
fn int_product_difference() {
    each(20, |rng| {
        let a = rng.whole();
        let b = rng.whole();
        let c = rng.whole();
        let d = rng.whole();
        let old = old::int_product_difference(a, b, c, d);
        assert_eq!(
            x87::int_product_difference(a, b, c, d).to_bits(),
            old,
            "int_product_difference {:x?}",
            (a, b, c, d)
        );
    });
}

#[test]
fn int_div_div_doubled() {
    each(21, |rng| {
        let a = rng.whole();
        let b = rng.float();
        let c = rng.float();
        let old = old::int_div_div_doubled(a, b, c);
        assert_eq!(
            x87::int_div_div_doubled(a, b, c).to_bits(),
            old,
            "int_div_div_doubled {:x?}",
            (a, b, c)
        );
        assert_eq!(
            x87::int_div_div_doubled(a, f32::from_bits(b), f32::from_bits(c)).to_bits(),
            old,
            "int_div_div_doubled (f32 inputs) {:x?}",
            (a, b, c)
        );
    });
}

#[test]
fn angle_scaled3_to_int() {
    each(22, |rng| {
        let y = rng.whole();
        let x = rng.whole();
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let old = old::angle_scaled3_to_int(y, x, a, b, c);
        assert_eq!(
            x87::angle_scaled3_to_int(y, x, a, b, c),
            old,
            "angle_scaled3_to_int {:x?}",
            (y, x, a, b, c)
        );
        assert_eq!(
            x87::angle_scaled3_to_int(
                y,
                x,
                f32::from_bits(a),
                f32::from_bits(b),
                f32::from_bits(c)
            ),
            old,
            "angle_scaled3_to_int (f32 inputs) {:x?}",
            (y, x, a, b, c)
        );
    });
}

#[test]
fn angle_scaled5_to_int() {
    each(23, |rng| {
        let y = rng.whole();
        let x = rng.whole();
        let a = rng.float();
        let b = rng.float();
        let old = old::angle_scaled5_to_int(y, x, a, b);
        assert_eq!(
            x87::angle_scaled5_to_int(y, x, a, b),
            old,
            "angle_scaled5_to_int {:x?}",
            (y, x, a, b)
        );
        assert_eq!(
            x87::angle_scaled5_to_int(y, x, f32::from_bits(a), f32::from_bits(b)),
            old,
            "angle_scaled5_to_int (f32 inputs) {:x?}",
            (y, x, a, b)
        );
    });
}

#[test]
fn hypot_ratio() {
    each(24, |rng| {
        let a = rng.whole();
        let b = rng.whole();
        let c = rng.whole();
        let d = rng.whole();
        let old = old::hypot_ratio(a, b, c, d);
        assert_eq!(
            x87::hypot_ratio(a, b, c, d).to_bits(),
            old,
            "hypot_ratio {:x?}",
            (a, b, c, d)
        );
    });
}

#[test]
fn weighted3_to_int() {
    each(25, |rng| {
        let a = rng.whole();
        let b = rng.whole();
        let c = rng.whole();
        let x = rng.float();
        let y = rng.float();
        let z = rng.float();
        let old = old::weighted3_to_int(a, b, c, x, y, z);
        assert_eq!(
            x87::weighted3_to_int(a, b, c, x, y, z),
            old,
            "weighted3_to_int {:x?}",
            (a, b, c, x, y, z)
        );
        assert_eq!(
            x87::weighted3_to_int(
                a,
                b,
                c,
                f32::from_bits(x),
                f32::from_bits(y),
                f32::from_bits(z)
            ),
            old,
            "weighted3_to_int (f32 inputs) {:x?}",
            (a, b, c, x, y, z)
        );
    });
}

#[test]
fn int_times_plus() {
    each(26, |rng| {
        let a = rng.whole();
        let b = rng.float();
        let c = rng.float();
        let old = old::int_times_plus(a, b, c);
        assert_eq!(
            x87::int_times_plus(a, b, c).to_bits(),
            old,
            "int_times_plus {:x?}",
            (a, b, c)
        );
        assert_eq!(
            x87::int_times_plus(a, f32::from_bits(b), f32::from_bits(c)).to_bits(),
            old,
            "int_times_plus (f32 inputs) {:x?}",
            (a, b, c)
        );
    });
}

#[test]
fn sub_div() {
    each(27, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let old = old::sub_div(a, b, c);
        assert_eq!(
            x87::sub_div(a, b, c).to_bits(),
            old,
            "sub_div {:x?}",
            (a, b, c)
        );
        assert_eq!(
            x87::sub_div(f32::from_bits(a), f32::from_bits(b), f32::from_bits(c)).to_bits(),
            old,
            "sub_div (f32 inputs) {:x?}",
            (a, b, c)
        );
    });
}

#[test]
fn negated_square_times_plus_to_int() {
    each(28, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let old = old::negated_square_times_plus_to_int(a, b, c);
        assert_eq!(
            x87::negated_square_times_plus_to_int(a, b, c),
            old,
            "negated_square_times_plus_to_int {:x?}",
            (a, b, c)
        );
        assert_eq!(
            x87::negated_square_times_plus_to_int(
                f32::from_bits(a),
                f32::from_bits(b),
                f32::from_bits(c)
            ),
            old,
            "negated_square_times_plus_to_int (f32 inputs) {:x?}",
            (a, b, c)
        );
    });
}

#[test]
fn share_left_to_int() {
    each(29, |rng| {
        let a = rng.whole();
        let b = rng.whole();
        let c = rng.float();
        let old = old::share_left_to_int(a, b, c);
        assert_eq!(
            x87::share_left_to_int(a, b, c),
            old,
            "share_left_to_int {:x?}",
            (a, b, c)
        );
        assert_eq!(
            x87::share_left_to_int(a, b, f32::from_bits(c)),
            old,
            "share_left_to_int (f32 inputs) {:x?}",
            (a, b, c)
        );
    });
}

#[test]
fn mul2() {
    each(30, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let old = old::mul2(a, b, c);
        assert_eq!(x87::mul2(a, b, c).to_bits(), old, "mul2 {:x?}", (a, b, c));
        assert_eq!(
            x87::mul2(f32::from_bits(a), f32::from_bits(b), f32::from_bits(c)).to_bits(),
            old,
            "mul2 (f32 inputs) {:x?}",
            (a, b, c)
        );
    });
}

#[test]
fn circle_curve() {
    each(31, |rng| {
        let i = rng.whole();
        let a = rng.float();
        let b = rng.float();
        let negate = rng.flag();
        let old = old::circle_curve(i, a, b, negate);
        assert_eq!(
            x87::circle_curve(i, a, b, negate).to_bits(),
            old,
            "circle_curve {:x?}",
            (i, a, b, negate)
        );
        assert_eq!(
            x87::circle_curve(i, f32::from_bits(a), f32::from_bits(b), negate).to_bits(),
            old,
            "circle_curve (f32 inputs) {:x?}",
            (i, a, b, negate)
        );
    });
}

#[test]
fn negated() {
    each(32, |rng| {
        let a = rng.float();
        let old = old::negated(a);
        assert_eq!(x87::negated(a).to_bits(), old, "negated {:x?}", (a,));
        assert_eq!(
            x87::negated(f32::from_bits(a)).to_bits(),
            old,
            "negated (f32 inputs) {:x?}",
            (a,)
        );
    });
}

#[test]
fn sub_mul() {
    each(33, |rng| {
        let b = rng.float();
        let a = rng.float();
        let k = rng.float();
        let old = old::sub_mul(b, a, k);
        assert_eq!(
            x87::sub_mul(b, a, k).to_bits(),
            old,
            "sub_mul {:x?}",
            (b, a, k)
        );
        assert_eq!(
            x87::sub_mul(f32::from_bits(b), f32::from_bits(a), f32::from_bits(k)).to_bits(),
            old,
            "sub_mul (f32 inputs) {:x?}",
            (b, a, k)
        );
    });
}

#[test]
fn cosine_step() {
    each(34, |rng| {
        let c = rng.whole();
        let m = rng.whole();
        let t = rng.float();
        let a = rng.float();
        let old = old::cosine_step(c, m, t, a);
        assert_eq!(
            x87::cosine_step(c, m, t, a).to_bits(),
            old,
            "cosine_step {:x?}",
            (c, m, t, a)
        );
        assert_eq!(
            x87::cosine_step(c, m, f32::from_bits(t), f32::from_bits(a)).to_bits(),
            old,
            "cosine_step (f32 inputs) {:x?}",
            (c, m, t, a)
        );
    });
}

#[test]
fn round_float() {
    each(35, |rng| {
        let a = rng.float();
        let old = old::round_float(a);
        assert_eq!(
            x87::round_float(a).to_bits(),
            old,
            "round_float {:x?}",
            (a,)
        );
        assert_eq!(
            x87::round_float(f32::from_bits(a)).to_bits(),
            old,
            "round_float (f32 inputs) {:x?}",
            (a,)
        );
    });
}

#[test]
fn compare_status() {
    each(36, |rng| {
        let a = rng.float();
        let b = rng.float();
        let old = old::compare_status(a, b);
        assert_eq!(
            x87::compare_status(a, b),
            old,
            "compare_status {:x?}",
            (a, b)
        );
        assert_eq!(
            x87::compare_status(f32::from_bits(a), f32::from_bits(b)),
            old,
            "compare_status (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn split_x() {
    each(37, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let x0 = rng.float();
        let x2 = rng.float();
        let old = old::split_x(a, b, c, x0, x2);
        assert_eq!(
            x87::split_x(a, b, c, x0, x2).to_bits(),
            old,
            "split_x {:x?}",
            (a, b, c, x0, x2)
        );
        assert_eq!(
            x87::split_x(
                f32::from_bits(a),
                f32::from_bits(b),
                f32::from_bits(c),
                f32::from_bits(x0),
                f32::from_bits(x2)
            )
            .to_bits(),
            old,
            "split_x (f32 inputs) {:x?}",
            (a, b, c, x0, x2)
        );
    });
}

#[test]
fn sine() {
    each(38, |rng| {
        let a = rng.float();
        let old = old::sine(a);
        assert_eq!(x87::sine(a).to_bits(), old, "sine {:x?}", (a,));
        assert_eq!(
            x87::sine(f32::from_bits(a)).to_bits(),
            old,
            "sine (f32 inputs) {:x?}",
            (a,)
        );
    });
}

#[test]
fn cosine() {
    each(39, |rng| {
        let a = rng.float();
        let negate = rng.flag();
        let old = old::cosine(a, negate);
        assert_eq!(
            x87::cosine(a, negate).to_bits(),
            old,
            "cosine {:x?}",
            (a, negate)
        );
        assert_eq!(
            x87::cosine(f32::from_bits(a), negate).to_bits(),
            old,
            "cosine (f32 inputs) {:x?}",
            (a, negate)
        );
    });
}

#[test]
fn mul_add() {
    each(40, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let old = old::mul_add(a, b, c);
        assert_eq!(
            x87::mul_add(a, b, c).to_bits(),
            old,
            "mul_add {:x?}",
            (a, b, c)
        );
        assert_eq!(
            x87::mul_add(f32::from_bits(a), f32::from_bits(b), f32::from_bits(c)).to_bits(),
            old,
            "mul_add (f32 inputs) {:x?}",
            (a, b, c)
        );
    });
}

#[test]
fn add_to_int() {
    each(41, |rng| {
        let a = rng.float();
        let b = rng.float();
        let old = old::add_to_int(a, b);
        assert_eq!(x87::add_to_int(a, b), old, "add_to_int {:x?}", (a, b));
        assert_eq!(
            x87::add_to_int(f32::from_bits(a), f32::from_bits(b)),
            old,
            "add_to_int (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn sub_compare() {
    each(42, |rng| {
        let a = rng.float();
        let b = rng.float();
        let negate = rng.flag();
        let c = rng.float();
        let old = old::sub_compare(a, b, negate, c);
        assert_eq!(
            x87::sub_compare(a, b, negate, c),
            old,
            "sub_compare {:x?}",
            (a, b, negate, c)
        );
        assert_eq!(
            x87::sub_compare(
                f32::from_bits(a),
                f32::from_bits(b),
                negate,
                f32::from_bits(c)
            ),
            old,
            "sub_compare (f32 inputs) {:x?}",
            (a, b, negate, c)
        );
    });
}

#[test]
fn int_plus_less_squared() {
    each(43, |rng| {
        let y = rng.whole();
        let a = rng.float();
        let b = rng.float();
        let old = old::int_plus_less_squared(y, a, b);
        assert_eq!(
            x87::int_plus_less_squared(y, a, b).to_bits(),
            old,
            "int_plus_less_squared {:x?}",
            (y, a, b)
        );
        assert_eq!(
            x87::int_plus_less_squared(y, f32::from_bits(a), f32::from_bits(b)).to_bits(),
            old,
            "int_plus_less_squared (f32 inputs) {:x?}",
            (y, a, b)
        );
    });
}

#[test]
fn sub_thrice() {
    each(44, |rng| {
        let a = rng.float();
        let b = rng.float();
        let old = old::sub_thrice(a, b);
        assert_eq!(
            x87::sub_thrice(a, b).to_bits(),
            old,
            "sub_thrice {:x?}",
            (a, b)
        );
        assert_eq!(
            x87::sub_thrice(f32::from_bits(a), f32::from_bits(b)).to_bits(),
            old,
            "sub_thrice (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn ease() {
    each(45, |rng| {
        let t = rng.float();
        let c = rng.float();
        let k = rng.float();
        let old = old::ease(t, c, k);
        assert_eq!(x87::ease(t, c, k).to_bits(), old, "ease {:x?}", (t, c, k));
        assert_eq!(
            x87::ease(f32::from_bits(t), f32::from_bits(c), f32::from_bits(k)).to_bits(),
            old,
            "ease (f32 inputs) {:x?}",
            (t, c, k)
        );
    });
}

#[test]
fn two_pi_less() {
    each(46, |rng| {
        let a = rng.float();
        let old = old::two_pi_less(a);
        assert_eq!(
            x87::two_pi_less(a).to_bits(),
            old,
            "two_pi_less {:x?}",
            (a,)
        );
    });
}

#[test]
fn over_int() {
    each(47, |rng| {
        let a = rng.float();
        let n = rng.whole();
        let old = old::over_int(a, n);
        assert_eq!(x87::over_int(a, n).to_bits(), old, "over_int {:x?}", (a, n));
        assert_eq!(
            x87::over_int(f32::from_bits(a), n).to_bits(),
            old,
            "over_int (f32 inputs) {:x?}",
            (a, n)
        );
    });
}

#[test]
fn zero_times_plus_times() {
    each(48, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let k = rng.float();
        let negative = rng.flag();
        let old = old::zero_times_plus_times(a, b, c, k, negative);
        assert_eq!(
            x87::zero_times_plus_times(a, b, c, k, negative).to_bits(),
            old,
            "zero_times_plus_times {:x?}",
            (a, b, c, k, negative)
        );
        assert_eq!(
            x87::zero_times_plus_times(
                f32::from_bits(a),
                f32::from_bits(b),
                f32::from_bits(c),
                f32::from_bits(k),
                negative
            )
            .to_bits(),
            old,
            "zero_times_plus_times (f32 inputs) {:x?}",
            (a, b, c, k, negative)
        );
    });
}

#[test]
fn prem_plus() {
    each(49, |rng| {
        let r = rng.whole();
        let m = rng.float();
        let v = rng.float();
        let old = old::prem_plus(r, m, v);
        assert_eq!(
            x87::prem_plus(r, m, v).to_bits(),
            old,
            "prem_plus {:x?}",
            (r, m, v)
        );
        assert_eq!(
            x87::prem_plus(r, f32::from_bits(m), f32::from_bits(v)).to_bits(),
            old,
            "prem_plus (f32 inputs) {:x?}",
            (r, m, v)
        );
    });
}

#[test]
fn rate_to_int() {
    each(50, |rng| {
        let a = rng.float();
        let b = rng.float();
        let k1 = rng.float();
        let k2 = rng.float();
        let k3 = rng.float();
        let one = rng.float();
        let k4 = rng.float();
        let old = old::rate_to_int(a, b, k1, k2, k3, one, k4);
        assert_eq!(
            x87::rate_to_int(a, b, k1, k2, k3, one, k4),
            old,
            "rate_to_int {:x?}",
            (a, b, k1, k2, k3, one, k4)
        );
        assert_eq!(
            x87::rate_to_int(
                f32::from_bits(a),
                f32::from_bits(b),
                f32::from_bits(k1),
                f32::from_bits(k2),
                f32::from_bits(k3),
                f32::from_bits(one),
                f32::from_bits(k4)
            ),
            old,
            "rate_to_int (f32 inputs) {:x?}",
            (a, b, k1, k2, k3, one, k4)
        );
    });
}

#[test]
fn less_plus_int_times() {
    each(51, |rng| {
        let d = rng.float();
        let m = rng.float();
        let f = rng.float();
        let i = rng.whole();
        let old = old::less_plus_int_times(d, m, f, i);
        assert_eq!(
            x87::less_plus_int_times(d, m, f, i).to_bits(),
            old,
            "less_plus_int_times {:x?}",
            (d, m, f, i)
        );
        assert_eq!(
            x87::less_plus_int_times(f32::from_bits(d), f32::from_bits(m), f32::from_bits(f), i)
                .to_bits(),
            old,
            "less_plus_int_times (f32 inputs) {:x?}",
            (d, m, f, i)
        );
    });
}

#[test]
fn pair_sum_less() {
    each(52, |rng| {
        let a = rng.whole();
        let m1 = rng.float();
        let b = rng.whole();
        let m2 = rng.float();
        let d = rng.float();
        let e = rng.float();
        let old = old::pair_sum_less(a, m1, b, m2, d, e);
        assert_eq!(
            x87::pair_sum_less(a, m1, b, m2, d, e).to_bits(),
            old,
            "pair_sum_less {:x?}",
            (a, m1, b, m2, d, e)
        );
        assert_eq!(
            x87::pair_sum_less(
                a,
                f32::from_bits(m1),
                b,
                f32::from_bits(m2),
                f32::from_bits(d),
                f32::from_bits(e)
            )
            .to_bits(),
            old,
            "pair_sum_less (f32 inputs) {:x?}",
            (a, m1, b, m2, d, e)
        );
    });
}

#[test]
fn product_plus() {
    each(53, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let d = rng.float();
        let old = old::product_plus(a, b, c, d);
        assert_eq!(
            x87::product_plus(a, b, c, d).to_bits(),
            old,
            "product_plus {:x?}",
            (a, b, c, d)
        );
        assert_eq!(
            x87::product_plus(
                f32::from_bits(a),
                f32::from_bits(b),
                f32::from_bits(c),
                f32::from_bits(d)
            )
            .to_bits(),
            old,
            "product_plus (f32 inputs) {:x?}",
            (a, b, c, d)
        );
    });
}

#[test]
fn float_times_int_to_int() {
    each(54, |rng| {
        let f = rng.float();
        let i = rng.whole();
        let old = old::float_times_int_to_int(f, i);
        assert_eq!(
            x87::float_times_int_to_int(f, i),
            old,
            "float_times_int_to_int {:x?}",
            (f, i)
        );
        assert_eq!(
            x87::float_times_int_to_int(f32::from_bits(f), i),
            old,
            "float_times_int_to_int (f32 inputs) {:x?}",
            (f, i)
        );
    });
}

#[test]
fn speed_rate_to_int() {
    each(55, |rng| {
        let a = rng.float();
        let k1 = rng.float();
        let k2 = rng.float();
        let k3 = rng.float();
        let one = rng.float();
        let k4 = rng.float();
        let old = old::speed_rate_to_int(a, k1, k2, k3, one, k4);
        assert_eq!(
            x87::speed_rate_to_int(a, k1, k2, k3, one, k4),
            old,
            "speed_rate_to_int {:x?}",
            (a, k1, k2, k3, one, k4)
        );
        assert_eq!(
            x87::speed_rate_to_int(
                f32::from_bits(a),
                f32::from_bits(k1),
                f32::from_bits(k2),
                f32::from_bits(k3),
                f32::from_bits(one),
                f32::from_bits(k4)
            ),
            old,
            "speed_rate_to_int (f32 inputs) {:x?}",
            (a, k1, k2, k3, one, k4)
        );
    });
}

#[test]
fn times_plus_to_int() {
    each(56, |rng| {
        let v = rng.float();
        let a = rng.float();
        let b = rng.float();
        let old = old::times_plus_to_int(v, a, b);
        assert_eq!(
            x87::times_plus_to_int(v, a, b),
            old,
            "times_plus_to_int {:x?}",
            (v, a, b)
        );
        assert_eq!(
            x87::times_plus_to_int(f32::from_bits(v), f32::from_bits(a), f32::from_bits(b)),
            old,
            "times_plus_to_int (f32 inputs) {:x?}",
            (v, a, b)
        );
    });
}

#[test]
fn times_compare() {
    each(57, |rng| {
        let a = rng.float();
        let k = rng.float();
        let c = rng.float();
        let old = old::times_compare(a, k, c);
        assert_eq!(
            x87::times_compare(a, k, c),
            old,
            "times_compare {:x?}",
            (a, k, c)
        );
        assert_eq!(
            x87::times_compare(f32::from_bits(a), f32::from_bits(k), f32::from_bits(c)),
            old,
            "times_compare (f32 inputs) {:x?}",
            (a, k, c)
        );
    });
}

#[test]
fn one_less_inverse() {
    each(58, |rng| {
        let a = rng.float();
        let k = rng.float();
        let one = rng.float();
        let old = old::one_less_inverse(a, k, one);
        assert_eq!(
            x87::one_less_inverse(a, k, one).to_bits(),
            old,
            "one_less_inverse {:x?}",
            (a, k, one)
        );
        assert_eq!(
            x87::one_less_inverse(f32::from_bits(a), f32::from_bits(k), f32::from_bits(one))
                .to_bits(),
            old,
            "one_less_inverse (f32 inputs) {:x?}",
            (a, k, one)
        );
    });
}

#[test]
fn sum4_times() {
    each(59, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let d = rng.float();
        let k = rng.float();
        let old = old::sum4_times(a, b, c, d, k);
        assert_eq!(
            x87::sum4_times(a, b, c, d, k).to_bits(),
            old,
            "sum4_times {:x?}",
            (a, b, c, d, k)
        );
        assert_eq!(
            x87::sum4_times(
                f32::from_bits(a),
                f32::from_bits(b),
                f32::from_bits(c),
                f32::from_bits(d),
                f32::from_bits(k)
            )
            .to_bits(),
            old,
            "sum4_times (f32 inputs) {:x?}",
            (a, b, c, d, k)
        );
    });
}

#[test]
fn int_times_plus_int_to_int() {
    each(60, |rng| {
        let a = rng.whole();
        let k = rng.float();
        let b = rng.whole();
        let old = old::int_times_plus_int_to_int(a, k, b);
        assert_eq!(
            x87::int_times_plus_int_to_int(a, k, b),
            old,
            "int_times_plus_int_to_int {:x?}",
            (a, k, b)
        );
        assert_eq!(
            x87::int_times_plus_int_to_int(a, f32::from_bits(k), b),
            old,
            "int_times_plus_int_to_int (f32 inputs) {:x?}",
            (a, k, b)
        );
    });
}

#[test]
fn int_lerp_to_int() {
    each(61, |rng| {
        let a = rng.whole();
        let b = rng.whole();
        let f = rng.float();
        let old = old::int_lerp_to_int(a, b, f);
        assert_eq!(
            x87::int_lerp_to_int(a, b, f),
            old,
            "int_lerp_to_int {:x?}",
            (a, b, f)
        );
        assert_eq!(
            x87::int_lerp_to_int(a, b, f32::from_bits(f)),
            old,
            "int_lerp_to_int (f32 inputs) {:x?}",
            (a, b, f)
        );
    });
}

#[test]
fn int_over_int_times() {
    each(62, |rng| {
        let a = rng.whole();
        let b = rng.whole();
        let k = rng.float();
        let old = old::int_over_int_times(a, b, k);
        assert_eq!(
            x87::int_over_int_times(a, b, k).to_bits(),
            old,
            "int_over_int_times {:x?}",
            (a, b, k)
        );
        assert_eq!(
            x87::int_over_int_times(a, b, f32::from_bits(k)).to_bits(),
            old,
            "int_over_int_times (f32 inputs) {:x?}",
            (a, b, k)
        );
    });
}

#[test]
fn toward() {
    each(63, |rng| {
        let a = rng.float();
        let b = rng.float();
        let r = rng.float();
        let old = old::toward(a, b, r);
        assert_eq!(
            x87::toward(a, b, r).to_bits(),
            old,
            "toward {:x?}",
            (a, b, r)
        );
        assert_eq!(
            x87::toward(f32::from_bits(a), f32::from_bits(b), f32::from_bits(r)).to_bits(),
            old,
            "toward (f32 inputs) {:x?}",
            (a, b, r)
        );
    });
}

#[test]
fn plus_times_int() {
    each(64, |rng| {
        let a = rng.float();
        let v = rng.float();
        let t = rng.whole();
        let old = old::plus_times_int(a, v, t);
        assert_eq!(
            x87::plus_times_int(a, v, t).to_bits(),
            old,
            "plus_times_int {:x?}",
            (a, v, t)
        );
        assert_eq!(
            x87::plus_times_int(f32::from_bits(a), f32::from_bits(v), t).to_bits(),
            old,
            "plus_times_int (f32 inputs) {:x?}",
            (a, v, t)
        );
    });
}

#[test]
fn less_own_times() {
    each(65, |rng| {
        let a = rng.float();
        let k = rng.float();
        let old = old::less_own_times(a, k);
        assert_eq!(
            x87::less_own_times(a, k).to_bits(),
            old,
            "less_own_times {:x?}",
            (a, k)
        );
        assert_eq!(
            x87::less_own_times(f32::from_bits(a), f32::from_bits(k)).to_bits(),
            old,
            "less_own_times (f32 inputs) {:x?}",
            (a, k)
        );
    });
}

#[test]
fn fraction_times_to_int() {
    each(66, |rng| {
        let v = rng.float();
        let k = rng.float();
        let old = old::fraction_times_to_int(v, k);
        assert_eq!(
            x87::fraction_times_to_int(v, k),
            old,
            "fraction_times_to_int {:x?}",
            (v, k)
        );
        assert_eq!(
            x87::fraction_times_to_int(f32::from_bits(v), f32::from_bits(k)),
            old,
            "fraction_times_to_int (f32 inputs) {:x?}",
            (v, k)
        );
    });
}

#[test]
fn float_hypot() {
    each(67, |rng| {
        let a = rng.float();
        let b = rng.float();
        let old = old::float_hypot(a, b);
        assert_eq!(
            x87::float_hypot(a, b).to_bits(),
            old,
            "float_hypot {:x?}",
            (a, b)
        );
        assert_eq!(
            x87::float_hypot(f32::from_bits(a), f32::from_bits(b)).to_bits(),
            old,
            "float_hypot (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn below_or_unordered() {
    each(68, |rng| {
        let a = rng.float();
        let b = rng.float();
        let old = old::below_or_unordered(a, b);
        assert_eq!(
            x87::below_or_unordered(a, b),
            old,
            "below_or_unordered {:x?}",
            (a, b)
        );
        assert_eq!(
            x87::below_or_unordered(f32::from_bits(a), f32::from_bits(b)),
            old,
            "below_or_unordered (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn scaled_abs_less_below() {
    each(69, |rng| {
        let a = rng.float();
        let h = rng.float();
        let k = rng.float();
        let b = rng.float();
        let old = old::scaled_abs_less_below(a, h, k, b);
        assert_eq!(
            x87::scaled_abs_less_below(a, h, k, b),
            old,
            "scaled_abs_less_below {:x?}",
            (a, h, k, b)
        );
        assert_eq!(
            x87::scaled_abs_less_below(
                f32::from_bits(a),
                f32::from_bits(h),
                f32::from_bits(k),
                f32::from_bits(b)
            ),
            old,
            "scaled_abs_less_below (f32 inputs) {:x?}",
            (a, h, k, b)
        );
    });
}

#[test]
fn int_square_sum() {
    each(70, |rng| {
        let a = rng.whole();
        let b = rng.whole();
        let old = old::int_square_sum(a, b);
        assert_eq!(
            x87::int_square_sum(a, b).to_bits(),
            old,
            "int_square_sum {:x?}",
            (a, b)
        );
    });
}

#[test]
fn float_square_sum() {
    each(71, |rng| {
        let a = rng.float();
        let b = rng.float();
        let old = old::float_square_sum(a, b);
        assert_eq!(
            x87::float_square_sum(a, b).to_bits(),
            old,
            "float_square_sum {:x?}",
            (a, b)
        );
        assert_eq!(
            x87::float_square_sum(f32::from_bits(a), f32::from_bits(b)).to_bits(),
            old,
            "float_square_sum (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn int_plus_times() {
    each(72, |rng| {
        let a = rng.whole();
        let b = rng.float();
        let c = rng.float();
        let old = old::int_plus_times(a, b, c);
        assert_eq!(
            x87::int_plus_times(a, b, c).to_bits(),
            old,
            "int_plus_times {:x?}",
            (a, b, c)
        );
        assert_eq!(
            x87::int_plus_times(a, f32::from_bits(b), f32::from_bits(c)).to_bits(),
            old,
            "int_plus_times (f32 inputs) {:x?}",
            (a, b, c)
        );
    });
}

#[test]
fn abs_below_or_unordered() {
    each(73, |rng| {
        let a = rng.float();
        let k = rng.float();
        let old = old::abs_below_or_unordered(a, k);
        assert_eq!(
            x87::abs_below_or_unordered(a, k),
            old,
            "abs_below_or_unordered {:x?}",
            (a, k)
        );
        assert_eq!(
            x87::abs_below_or_unordered(f32::from_bits(a), f32::from_bits(k)),
            old,
            "abs_below_or_unordered (f32 inputs) {:x?}",
            (a, k)
        );
    });
}

#[test]
fn int_square_sum_scaled_to_int() {
    each(74, |rng| {
        let a = rng.whole();
        let b = rng.whole();
        let k = rng.float();
        let times = rng.float();
        let old = old::int_square_sum_scaled_to_int(a, b, k, times);
        assert_eq!(
            x87::int_square_sum_scaled_to_int(a, b, k, times),
            old,
            "int_square_sum_scaled_to_int {:x?}",
            (a, b, k, times)
        );
        assert_eq!(
            x87::int_square_sum_scaled_to_int(a, b, f32::from_bits(k), times),
            old,
            "int_square_sum_scaled_to_int (f32 inputs) {:x?}",
            (a, b, k, times)
        );
    });
}

#[test]
fn eased_roll() {
    each(75, |rng| {
        let i = rng.whole();
        let a = rng.float();
        let b = rng.float();
        let v = rng.float();
        let c = rng.float();
        let m = rng.float();
        let h = rng.float();
        let old = old::eased_roll(i, a, b, v, c, m, h);
        assert_eq!(
            x87::eased_roll(i, a, b, v, c, m, h).to_bits(),
            old,
            "eased_roll {:x?}",
            (i, a, b, v, c, m, h)
        );
        assert_eq!(
            x87::eased_roll(
                i,
                f32::from_bits(a),
                f32::from_bits(b),
                f32::from_bits(v),
                f32::from_bits(c),
                f32::from_bits(m),
                f32::from_bits(h)
            )
            .to_bits(),
            old,
            "eased_roll (f32 inputs) {:x?}",
            (i, a, b, v, c, m, h)
        );
    });
}

#[test]
fn sun_size() {
    each(76, |rng| {
        let x = rng.float();
        let cx = rng.float();
        let y = rng.float();
        let cy = rng.float();
        let h = rng.float();
        let k = rng.float();
        let old = old::sun_size(x, cx, y, cy, h, k);
        assert_eq!(
            x87::sun_size(x, cx, y, cy, h, k),
            old,
            "sun_size {:x?}",
            (x, cx, y, cy, h, k)
        );
        assert_eq!(
            x87::sun_size(
                f32::from_bits(x),
                f32::from_bits(cx),
                f32::from_bits(y),
                f32::from_bits(cy),
                f32::from_bits(h),
                f32::from_bits(k)
            ),
            old,
            "sun_size (f32 inputs) {:x?}",
            (x, cx, y, cy, h, k)
        );
    });
}

#[test]
fn plane_height() {
    each(77, |rng| {
        let a = rng.whole();
        let b = rng.whole();
        let c = rng.whole();
        let f = rng.float();
        let g = rng.float();
        let old = old::plane_height(a, b, c, f, g);
        assert_eq!(
            x87::plane_height(a, b, c, f, g),
            old,
            "plane_height {:x?}",
            (a, b, c, f, g)
        );
        assert_eq!(
            x87::plane_height(a, b, c, f32::from_bits(f), f32::from_bits(g)),
            old,
            "plane_height (f32 inputs) {:x?}",
            (a, b, c, f, g)
        );
    });
}

#[test]
fn compare_full() {
    each(78, |rng| {
        let a = rng.float();
        let b = rng.float();
        let old = old::compare_full(a, b);
        assert_eq!(x87::compare_full(a, b), old, "compare_full {:x?}", (a, b));
        assert_eq!(
            x87::compare_full(f32::from_bits(a), f32::from_bits(b)),
            old,
            "compare_full (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn int_lerp() {
    each(79, |rng| {
        let i = rng.whole();
        let d = rng.float();
        let hi = rng.float();
        let lo = rng.float();
        let old = old::int_lerp(i, d, hi, lo);
        assert_eq!(
            x87::int_lerp(i, d, hi, lo).to_bits(),
            old,
            "int_lerp {:x?}",
            (i, d, hi, lo)
        );
        assert_eq!(
            x87::int_lerp(i, f32::from_bits(d), f32::from_bits(hi), f32::from_bits(lo)).to_bits(),
            old,
            "int_lerp (f32 inputs) {:x?}",
            (i, d, hi, lo)
        );
    });
}

#[test]
fn one_less() {
    each(80, |rng| {
        let a = rng.float();
        let old = old::one_less(a);
        assert_eq!(x87::one_less(a).to_bits(), old, "one_less {:x?}", (a,));
        assert_eq!(
            x87::one_less(f32::from_bits(a)).to_bits(),
            old,
            "one_less (f32 inputs) {:x?}",
            (a,)
        );
    });
}

#[test]
fn abs_sqrt_times_plus() {
    each(81, |rng| {
        let x = rng.float();
        let t = rng.float();
        let c = rng.float();
        let old = old::abs_sqrt_times_plus(x, t, c);
        assert_eq!(
            x87::abs_sqrt_times_plus(x, t, c).to_bits(),
            old,
            "abs_sqrt_times_plus {:x?}",
            (x, t, c)
        );
        assert_eq!(
            x87::abs_sqrt_times_plus(f32::from_bits(x), f32::from_bits(t), f32::from_bits(c))
                .to_bits(),
            old,
            "abs_sqrt_times_plus (f32 inputs) {:x?}",
            (x, t, c)
        );
    });
}

#[test]
fn mul2_to_int() {
    each(82, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let old = old::mul2_to_int(a, b, c);
        assert_eq!(
            x87::mul2_to_int(a, b, c),
            old,
            "mul2_to_int {:x?}",
            (a, b, c)
        );
        assert_eq!(
            x87::mul2_to_int(f32::from_bits(a), f32::from_bits(b), f32::from_bits(c)),
            old,
            "mul2_to_int (f32 inputs) {:x?}",
            (a, b, c)
        );
    });
}

#[test]
fn abs_compare() {
    each(83, |rng| {
        let a = rng.float();
        let b = rng.float();
        let old = old::abs_compare(a, b);
        assert_eq!(x87::abs_compare(a, b), old, "abs_compare {:x?}", (a, b));
        assert_eq!(
            x87::abs_compare(f32::from_bits(a), f32::from_bits(b)),
            old,
            "abs_compare (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn abs_of() {
    each(84, |rng| {
        let a = rng.float();
        let old = old::abs_of(a);
        assert_eq!(x87::abs_of(a).to_bits(), old, "abs_of {:x?}", (a,));
        assert_eq!(
            x87::abs_of(f32::from_bits(a)).to_bits(),
            old,
            "abs_of (f32 inputs) {:x?}",
            (a,)
        );
    });
}

#[test]
fn float_hypot_to_int() {
    each(85, |rng| {
        let a = rng.float();
        let b = rng.float();
        let old = old::float_hypot_to_int(a, b);
        assert_eq!(
            x87::float_hypot_to_int(a, b),
            old,
            "float_hypot_to_int {:x?}",
            (a, b)
        );
        assert_eq!(
            x87::float_hypot_to_int(f32::from_bits(a), f32::from_bits(b)),
            old,
            "float_hypot_to_int (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn int_toward_to_int() {
    each(86, |rng| {
        let i = rng.whole();
        let a = rng.float();
        let k = rng.float();
        let old = old::int_toward_to_int(i, a, k);
        assert_eq!(
            x87::int_toward_to_int(i, a, k),
            old,
            "int_toward_to_int {:x?}",
            (i, a, k)
        );
        assert_eq!(
            x87::int_toward_to_int(i, f32::from_bits(a), f32::from_bits(k)),
            old,
            "int_toward_to_int (f32 inputs) {:x?}",
            (i, a, k)
        );
    });
}

#[test]
fn div_mul3_to_int() {
    each(87, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let d = rng.float();
        let e = rng.float();
        let old = old::div_mul3_to_int(a, b, c, d, e);
        assert_eq!(
            x87::div_mul3_to_int(a, b, c, d, e),
            old,
            "div_mul3_to_int {:x?}",
            (a, b, c, d, e)
        );
        assert_eq!(
            x87::div_mul3_to_int(
                f32::from_bits(a),
                f32::from_bits(b),
                f32::from_bits(c),
                f32::from_bits(d),
                f32::from_bits(e)
            ),
            old,
            "div_mul3_to_int (f32 inputs) {:x?}",
            (a, b, c, d, e)
        );
    });
}

#[test]
fn square_times_over() {
    each(88, |rng| {
        let x = rng.float();
        let k = rng.float();
        let d = rng.float();
        let old = old::square_times_over(x, k, d);
        assert_eq!(
            x87::square_times_over(x, k, d).to_bits(),
            old,
            "square_times_over {:x?}",
            (x, k, d)
        );
        assert_eq!(
            x87::square_times_over(f32::from_bits(x), f32::from_bits(k), f32::from_bits(d))
                .to_bits(),
            old,
            "square_times_over (f32 inputs) {:x?}",
            (x, k, d)
        );
    });
}

#[test]
fn toward_one() {
    each(89, |rng| {
        let a = rng.float();
        let b = rng.float();
        let old = old::toward_one(a, b);
        assert_eq!(
            x87::toward_one(a, b).to_bits(),
            old,
            "toward_one {:x?}",
            (a, b)
        );
        assert_eq!(
            x87::toward_one(f32::from_bits(a), f32::from_bits(b)).to_bits(),
            old,
            "toward_one (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn lerp_int_over2() {
    each(90, |rng| {
        let a = rng.float();
        let b = rng.float();
        let k = rng.float();
        let i = rng.whole();
        let d = rng.float();
        let w = rng.float();
        let old = old::lerp_int_over2(a, b, k, i, d, w);
        assert_eq!(
            x87::lerp_int_over2(a, b, k, i, d, w).to_bits(),
            old,
            "lerp_int_over2 {:x?}",
            (a, b, k, i, d, w)
        );
        assert_eq!(
            x87::lerp_int_over2(
                f32::from_bits(a),
                f32::from_bits(b),
                f32::from_bits(k),
                i,
                f32::from_bits(d),
                f32::from_bits(w)
            )
            .to_bits(),
            old,
            "lerp_int_over2 (f32 inputs) {:x?}",
            (a, b, k, i, d, w)
        );
    });
}

#[test]
fn weight_force() {
    each(91, |rng| {
        let v = rng.float();
        let one = rng.float();
        let a = rng.float();
        let g = rng.float();
        let m = rng.float();
        let old = old::weight_force(v, one, a, g, m);
        assert_eq!(
            x87::weight_force(v, one, a, g, m).to_bits(),
            old,
            "weight_force {:x?}",
            (v, one, a, g, m)
        );
        assert_eq!(
            x87::weight_force(
                f32::from_bits(v),
                f32::from_bits(one),
                f32::from_bits(a),
                f32::from_bits(g),
                f32::from_bits(m)
            )
            .to_bits(),
            old,
            "weight_force (f32 inputs) {:x?}",
            (v, one, a, g, m)
        );
    });
}

#[test]
fn neg_product2_plus() {
    each(92, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let d = rng.float();
        let old = old::neg_product2_plus(a, b, c, d);
        assert_eq!(
            x87::neg_product2_plus(a, b, c, d).to_bits(),
            old,
            "neg_product2_plus {:x?}",
            (a, b, c, d)
        );
        assert_eq!(
            x87::neg_product2_plus(
                f32::from_bits(a),
                f32::from_bits(b),
                f32::from_bits(c),
                f32::from_bits(d)
            )
            .to_bits(),
            old,
            "neg_product2_plus (f32 inputs) {:x?}",
            (a, b, c, d)
        );
    });
}

#[test]
fn patan() {
    each(93, |rng| {
        let y = rng.float();
        let x = rng.float();
        let old = old::patan(y, x);
        assert_eq!(x87::patan(y, x).to_bits(), old, "patan {:x?}", (y, x));
        assert_eq!(
            x87::patan(f32::from_bits(y), f32::from_bits(x)).to_bits(),
            old,
            "patan (f32 inputs) {:x?}",
            (y, x)
        );
    });
}

#[test]
fn mul3_to_int() {
    each(94, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let d = rng.float();
        let old = old::mul3_to_int(a, b, c, d);
        assert_eq!(
            x87::mul3_to_int(a, b, c, d),
            old,
            "mul3_to_int {:x?}",
            (a, b, c, d)
        );
        assert_eq!(
            x87::mul3_to_int(
                f32::from_bits(a),
                f32::from_bits(b),
                f32::from_bits(c),
                f32::from_bits(d)
            ),
            old,
            "mul3_to_int (f32 inputs) {:x?}",
            (a, b, c, d)
        );
    });
}

#[test]
fn mul2_plus() {
    each(95, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let d = rng.float();
        let old = old::mul2_plus(a, b, c, d);
        assert_eq!(
            x87::mul2_plus(a, b, c, d).to_bits(),
            old,
            "mul2_plus {:x?}",
            (a, b, c, d)
        );
        assert_eq!(
            x87::mul2_plus(
                f32::from_bits(a),
                f32::from_bits(b),
                f32::from_bits(c),
                f32::from_bits(d)
            )
            .to_bits(),
            old,
            "mul2_plus (f32 inputs) {:x?}",
            (a, b, c, d)
        );
    });
}

#[test]
fn over_abs_plus_minus() {
    each(96, |rng| {
        let a = rng.float();
        let s = rng.float();
        let b = rng.float();
        let c = rng.float();
        let old = old::over_abs_plus_minus(a, s, b, c);
        assert_eq!(
            x87::over_abs_plus_minus(a, s, b, c).to_bits(),
            old,
            "over_abs_plus_minus {:x?}",
            (a, s, b, c)
        );
        assert_eq!(
            x87::over_abs_plus_minus(
                f32::from_bits(a),
                f32::from_bits(s),
                f32::from_bits(b),
                f32::from_bits(c)
            )
            .to_bits(),
            old,
            "over_abs_plus_minus (f32 inputs) {:x?}",
            (a, s, b, c)
        );
    });
}

#[test]
fn sub_mul_add() {
    each(97, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let d = rng.float();
        let old = old::sub_mul_add(a, b, c, d);
        assert_eq!(
            x87::sub_mul_add(a, b, c, d).to_bits(),
            old,
            "sub_mul_add {:x?}",
            (a, b, c, d)
        );
        assert_eq!(
            x87::sub_mul_add(
                f32::from_bits(a),
                f32::from_bits(b),
                f32::from_bits(c),
                f32::from_bits(d)
            )
            .to_bits(),
            old,
            "sub_mul_add (f32 inputs) {:x?}",
            (a, b, c, d)
        );
    });
}

#[test]
fn mul_div_status() {
    each(98, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let k = rng.float();
        let old = old::mul_div_status(a, b, c, k);
        assert_eq!(
            x87::mul_div_status(a, b, c, k),
            old,
            "mul_div_status {:x?}",
            (a, b, c, k)
        );
        assert_eq!(
            x87::mul_div_status(
                f32::from_bits(a),
                f32::from_bits(b),
                f32::from_bits(c),
                f32::from_bits(k)
            ),
            old,
            "mul_div_status (f32 inputs) {:x?}",
            (a, b, c, k)
        );
    });
}

#[test]
fn mul_status() {
    each(99, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let old = old::mul_status(a, b, c);
        assert_eq!(x87::mul_status(a, b, c), old, "mul_status {:x?}", (a, b, c));
        assert_eq!(
            x87::mul_status(f32::from_bits(a), f32::from_bits(b), f32::from_bits(c)),
            old,
            "mul_status (f32 inputs) {:x?}",
            (a, b, c)
        );
    });
}

#[test]
fn trunc_mul_to_int() {
    each(100, |rng| {
        let a = rng.float();
        let b = rng.float();
        let old = old::trunc_mul_to_int(a, b);
        assert_eq!(
            x87::trunc_mul_to_int(a, b),
            old,
            "trunc_mul_to_int {:x?}",
            (a, b)
        );
        assert_eq!(
            x87::trunc_mul_to_int(f32::from_bits(a), f32::from_bits(b)),
            old,
            "trunc_mul_to_int (f32 inputs) {:x?}",
            (a, b)
        );
    });
}

#[test]
fn lerp_table() {
    each(101, |rng| {
        let s = rng.float();
        let a0 = rng.float();
        let a1 = rng.float();
        let c0 = rng.float();
        let c1 = rng.float();
        let old = old::lerp_table(s, a0, a1, c0, c1);
        assert_eq!(
            x87::lerp_table(s, a0, a1, c0, c1).to_bits(),
            old,
            "lerp_table {:x?}",
            (s, a0, a1, c0, c1)
        );
        assert_eq!(
            x87::lerp_table(
                f32::from_bits(s),
                f32::from_bits(a0),
                f32::from_bits(a1),
                f32::from_bits(c0),
                f32::from_bits(c1)
            )
            .to_bits(),
            old,
            "lerp_table (f32 inputs) {:x?}",
            (s, a0, a1, c0, c1)
        );
    });
}

#[test]
fn sum_times_int_over_minus() {
    each(102, |rng| {
        let a = rng.float();
        let b = rng.float();
        let n = rng.whole();
        let k = rng.float();
        let old = old::sum_times_int_over_minus(a, b, n, k);
        assert_eq!(
            x87::sum_times_int_over_minus(a, b, n, k).to_bits(),
            old,
            "sum_times_int_over_minus {:x?}",
            (a, b, n, k)
        );
        assert_eq!(
            x87::sum_times_int_over_minus(
                f32::from_bits(a),
                f32::from_bits(b),
                n,
                f32::from_bits(k)
            )
            .to_bits(),
            old,
            "sum_times_int_over_minus (f32 inputs) {:x?}",
            (a, b, n, k)
        );
    });
}

#[test]
fn mul_div2() {
    each(103, |rng| {
        let a = rng.float();
        let b = rng.float();
        let c = rng.float();
        let d = rng.float();
        let old = old::mul_div2(a, b, c, d);
        assert_eq!(
            x87::mul_div2(a, b, c, d).to_bits(),
            old,
            "mul_div2 {:x?}",
            (a, b, c, d)
        );
        assert_eq!(
            x87::mul_div2(
                f32::from_bits(a),
                f32::from_bits(b),
                f32::from_bits(c),
                f32::from_bits(d)
            )
            .to_bits(),
            old,
            "mul_div2 (f32 inputs) {:x?}",
            (a, b, c, d)
        );
    });
}
