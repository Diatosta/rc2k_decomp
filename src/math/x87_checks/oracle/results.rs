//! The helpers whose results are floats in tuples and arrays (`(f32, f32)`, `[f32; 3]`),
//! each against its old body (`x87_old`, results as bits), over seeded random inputs of
//! every kind of float (`x87_typed_tests`) at each of the four rounding modes. Inputs go in
//! as bits and as `f32` where the helper takes `impl Flt`; results compare bit for bit (any two NaNs count as equal).

use super::old;
use crate::math::x87_checks::inputs::{assert_same, each_rounding};
use crate::math::x87_checks::plain as x87;
use crate::memory::session::Dword;

fn f(bits: u32) -> f32 {
    f32::from_bits(bits)
}

fn pair((a, b): (f32, f32)) -> (u32, u32) {
    (a.to_bits(), b.to_bits())
}

fn triple(v: [f32; 3]) -> [u32; 3] {
    v.map(f32::to_bits)
}

#[test]
fn reciprocal_int_and_times() {
    each_rounding(201, |rng| {
        let (a, b) = (rng.whole(), rng.float());
        let old = old::reciprocal_int_and_times(a, b);
        assert_same!(
            pair(x87::reciprocal_int_and_times(a, b)),
            old,
            "{:x?}",
            (a, b)
        );
        assert_same!(
            pair(x87::reciprocal_int_and_times(a, f(b))),
            old,
            "{:x?}",
            (a, b)
        );
    });
}

#[test]
fn mul_and_div() {
    each_rounding(202, |rng| {
        let (a, b, c) = (rng.float(), rng.float(), rng.float());
        let old = old::mul_and_div(a, b, c);
        assert_same!(pair(x87::mul_and_div(a, b, c)), old, "{:x?}", (a, b, c));
        assert_same!(
            pair(x87::mul_and_div(f(a), f(b), f(c))),
            old,
            "{:x?}",
            (a, b, c)
        );
    });
}

#[test]
fn half_and_middle() {
    each_rounding(203, |rng| {
        let (b, a, k) = (rng.float(), rng.float(), rng.float());
        let old = old::half_and_middle(b, a, k);
        assert_same!(pair(x87::half_and_middle(b, a, k)), old, "{:x?}", (b, a, k));
        assert_same!(
            pair(x87::half_and_middle(f(b), f(a), f(k))),
            old,
            "{:x?}",
            (b, a, k)
        );
    });
}

#[test]
fn int_times_plus_on() {
    each_rounding(204, |rng| {
        let a = rng.whole();
        let (m, c, d, e) = (rng.float(), rng.float(), rng.float(), rng.float());
        let old = old::int_times_plus_on(a, m, c, d, e);
        let args = (a, m, c, d, e);
        assert_same!(
            pair(x87::int_times_plus_on(a, m, c, d, e)),
            old,
            "{args:x?}"
        );
        assert_same!(
            pair(x87::int_times_plus_on(a, f(m), f(c), f(d), f(e))),
            old,
            "{args:x?}"
        );
    });
}

#[test]
fn int_times_plus_and_negated() {
    each_rounding(205, |rng| {
        let (a, b, c) = (rng.whole(), rng.float(), rng.float());
        let old = old::int_times_plus_and_negated(a, b, c);
        assert_same!(
            pair(x87::int_times_plus_and_negated(a, b, c)),
            old,
            "{:x?}",
            (a, b, c)
        );
        assert_same!(
            pair(x87::int_times_plus_and_negated(a, f(b), f(c))),
            old,
            "{:x?}",
            (a, b, c)
        );
    });
}

#[test]
fn sine_and_cosine() {
    each_rounding(206, |rng| {
        let t = rng.float();
        let old = old::sine_and_cosine(t);
        assert_same!(pair(x87::sine_and_cosine(t)), old, "{t:x}");
        assert_same!(pair(x87::sine_and_cosine(f(t))), old, "{t:x}");
    });
}

#[test]
fn turn_yz() {
    each_rounding(207, |rng| {
        let (y, z, s, c) = (rng.float(), rng.float(), rng.float(), rng.float());
        let old = old::turn_yz(y, z, s, c);
        assert_same!(pair(x87::turn_yz(y, z, s, c)), old, "{:x?}", (y, z, s, c));
        assert_same!(
            pair(x87::turn_yz(f(y), f(z), f(s), f(c))),
            old,
            "{:x?}",
            (y, z, s, c)
        );
    });
}

#[test]
fn angle_and_wave() {
    each_rounding(208, |rng| {
        let (a, v) = (rng.float(), rng.whole());
        let (kd, km, ks) = (rng.float(), rng.float(), rng.float());
        let old = old::angle_and_wave(a, v, kd, km, ks);
        let args = (a, v, kd, km, ks);
        assert_same!(
            pair(x87::angle_and_wave(a, v, kd, km, ks)),
            old,
            "{args:x?}"
        );
        assert_same!(
            pair(x87::angle_and_wave(f(a), v, f(kd), f(km), f(ks))),
            old,
            "{args:x?}"
        );
    });
}

#[test]
fn normalise_in_place() {
    each_rounding(209, |rng| {
        let v = [rng.float(), rng.float(), rng.float()];
        let zero = rng.float();
        let (mut want_v, mut got_v, mut got_f) = (v, v, v);
        let (want, want_equal) = old::normalise_in_place(&mut want_v, zero);
        let (got, equal) = x87::normalise_in_place(&mut got_v, zero);
        assert_same!(
            (got.to_bits(), equal, got_v),
            (want, want_equal, want_v),
            "{v:x?}"
        );
        let (got, equal) = x87::normalise_in_place(&mut got_f, f(zero));
        assert_same!(
            (got.to_bits(), equal, got_f),
            (want, want_equal, want_v),
            "{v:x?}"
        );
    });
}

#[test]
fn times_kept_abs_to_int() {
    each_rounding(210, |rng| {
        let (a, b, c) = (rng.float(), rng.float(), rng.float());
        let old = old::times_kept_abs_to_int(a, b, c);
        let got = x87::times_kept_abs_to_int(a, b, c);
        assert_same!((got.0.to_bits(), got.1), old, "{:x?}", (a, b, c));
        let got = x87::times_kept_abs_to_int(f(a), f(b), f(c));
        assert_same!((got.0.to_bits(), got.1), old, "{:x?}", (a, b, c));
    });
}

#[test]
fn blend_kept_abs_to_int() {
    each_rounding(221, |rng| {
        let (a, b, k1, k2, c) = (
            rng.float(),
            rng.float(),
            rng.float(),
            rng.float(),
            rng.float(),
        );
        let old = old::blend_kept_abs_to_int(a, b, k1, k2, c);
        let got = x87::blend_kept_abs_to_int(a, b, k1, k2, c);
        assert_same!((got.0.to_bits(), got.1), old, "{:x?}", (a, b, k1, k2, c));
        let got = x87::blend_kept_abs_to_int(f(a), f(b), f(k1), f(k2), f(c));
        assert_same!((got.0.to_bits(), got.1), old, "{:x?}", (a, b, k1, k2, c));
    });
}

#[test]
fn frac_step() {
    each_rounding(211, |rng| {
        let (y, k, s, x) = (rng.float(), rng.float(), rng.float(), rng.float());
        let old = old::frac_step(y, k, s, x);
        let got = x87::frac_step(y, k, s, x);
        assert_same!((got.0, got.1.to_bits()), old, "{:x?}", (y, k, s, x));
        let got = x87::frac_step(f(y), f(k), f(s), f(x));
        assert_same!((got.0, got.1.to_bits()), old, "{:x?}", (y, k, s, x));
    });
}

#[test]
fn abs_sum_to_int_kept() {
    each_rounding(212, |rng| {
        let (a, b, c, d) = (rng.float(), rng.float(), rng.float(), rng.float());
        let old = old::abs_sum_to_int_kept(a, b, c, d);
        let got = x87::abs_sum_to_int_kept(a, b, c, d);
        assert_same!((got.0, got.1.to_bits()), old, "{:x?}", (a, b, c, d));
        let got = x87::abs_sum_to_int_kept(f(a), f(b), f(c), f(d));
        assert_same!((got.0, got.1.to_bits()), old, "{:x?}", (a, b, c, d));
    });
}

#[test]
fn approach() {
    each_rounding(213, |rng| {
        let (i, v, k, t) = (rng.whole(), rng.float(), rng.float(), rng.whole());
        let old = old::approach(i, v, k, t);
        let got = x87::approach(i, v, k, t);
        assert_same!((got.0, got.1.to_bits()), old, "{:x?}", (i, v, k, t));
        let got = x87::approach(i, f(v), f(k), t);
        assert_same!((got.0, got.1.to_bits()), old, "{:x?}", (i, v, k, t));
    });
}

#[test]
fn unit_and_length() {
    each_rounding(214, |rng| {
        let p = [rng.float(), rng.float(), rng.float()];
        let (unit, length) = old::unit_and_length(p);
        let got = x87::unit_and_length(p);
        assert_same!((triple(got.0), got.1.to_bits()), (unit, length), "{p:x?}");
    });
}

#[test]
fn scale_row_bits() {
    each_rounding(215, |rng| {
        let row = [rng.float(), rng.float(), rng.float()];
        let c = rng.float();
        let old = old::scale_row_bits(row, c);
        assert_same!(triple(x87::scale_row(row, c)), old, "{:x?}", (row, c));
        assert_same!(triple(x87::scale_row(row, f(c))), old, "{:x?}", (row, c));
    });
}

#[test]
fn drifted() {
    each_rounding(216, |rng| {
        let r = rng.whole();
        let k = rng.float();
        let p = [rng.float(), rng.float(), rng.float()];
        let ticks = rng.whole();
        let at = [rng.float(), rng.float(), rng.float()];
        let old = old::drifted(r, k, p, ticks, at);
        let args = (r, k, p, ticks, at);
        assert_same!(triple(x87::drifted(r, k, p, ticks, at)), old, "{args:x?}");
        assert_same!(
            triple(x87::drifted(r, f(k), p, ticks, at)),
            old,
            "{args:x?}"
        );
    });
}

#[test]
fn normalised3() {
    each_rounding(217, |rng| {
        let p = [rng.float(), rng.float(), rng.float()];
        let one = rng.float();
        let old = old::normalised3(p, one);
        assert_same!(triple(x87::normalised3(p, one)), old, "{:x?}", (p, one));
        assert_same!(triple(x87::normalised3(p, f(one))), old, "{:x?}", (p, one));
    });
}

#[test]
fn clip_lerp3() {
    each_rounding(218, |rng| {
        let (m, fc, fs) = (rng.float(), rng.float(), rng.float());
        let pairs = [0; 3].map(|_| [rng.float(), rng.float()]);
        let old = old::clip_lerp3(m, fc, fs, pairs);
        let args = (m, fc, fs, pairs);
        assert_same!(triple(x87::clip_lerp3(m, fc, fs, pairs)), old, "{args:x?}");
        assert_same!(
            triple(x87::clip_lerp3(f(m), f(fc), f(fs), pairs)),
            old,
            "{args:x?}"
        );
    });
}

#[test]
fn column_turn() {
    each_rounding(219, |rng| {
        let p = [0; 3].map(|_| x87::to_tbyte(rng.float()));
        let matrix: [u32; 16] = std::array::from_fn(|_| rng.float());
        let cells = matrix.map(Dword::new);
        let old = old::column_turn(&p, &cells);
        assert_same!(
            triple(x87::column_turn(&p, &cells)),
            old,
            "{p:x?} {matrix:x?}"
        );
    });
}

#[test]
fn particle_square() {
    each_rounding(220, |rng| {
        let (a, b, c) = (rng.float(), rng.float(), rng.float());
        let k = [rng.float(), rng.float(), rng.float(), rng.float()];
        let size = rng.whole();
        let old = old::particle_square(a, b, c, k, size);
        let args = (a, b, c, k, size);
        assert_same!(
            x87::particle_square(a, b, c, k, size).map(f32::to_bits),
            old,
            "{args:x?}"
        );
        assert_same!(
            x87::particle_square(f(a), f(b), f(c), k, size).map(f32::to_bits),
            old,
            "{args:x?}"
        );
    });
}
