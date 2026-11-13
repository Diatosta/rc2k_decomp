//! The frozen comparisons of the helpers in `math/x87.rs` up to its "part B" line: until
//! 2026-11-13 each plain version ran against the original's x87 instructions at the four
//! rounding modes (bit for bit, any NaN equal to any NaN; the sines and cosines within an
//! ulp or the error of `fsin` / `fcos`'s 66-bit pi). The last run passed; the plain
//! versions' results on the same inputs are now kept as one hash per function
//! (`test_support::frozen`, table `FROZEN` at the end).

use crate::math::x87 as plain;
use crate::math::x87::set_control_word;
use crate::math::x87_float::{X, f80, from_f80};
use crate::test_support::frozen::{Canon, Digests};

/// Seeded generator (xorshift64*).
struct G(u64);

impl G {
    fn new(seed: u64) -> G {
        G(seed | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    /// A float's bits: random patterns, edge values, whole numbers, halves, values around
    /// the range of `i32`, denormals; now and then an infinity or a NaN.
    fn f(&mut self) -> u32 {
        const EDGE: [f32; 18] = [
            0.0,
            -0.0,
            1.0,
            -1.0,
            0.5,
            -0.5,
            1.5,
            2.5,
            -2.5,
            0.1,
            f32::MIN_POSITIVE,
            f32::MAX,
            -f32::MAX,
            2147483648.0,
            2147483520.0,
            -2147483648.0,
            1e-45,
            3.15,
        ];
        match self.below(10) {
            9 => [
                0x7F80_0000u32,
                0xFF80_0000,
                0x7FC0_0000,
                0xFFC0_0000,
                0x7F80_0001,
                0xFFA0_0000,
            ][self.below(6) as usize],
            0 => loop {
                let bits = self.next() as u32;
                if (bits >> 23) & 0xFF != 0xFF {
                    return bits;
                }
            },
            1 => EDGE[self.below(EDGE.len() as u64) as usize].to_bits(),
            2 => ((self.below(401) as i32 - 200) as f32).to_bits(),
            3 => (((self.below(200_001) as i32 - 100_000) as f32) + 0.5).to_bits(),
            4 => (((self.next() >> 11) as f64 / (1u64 << 53) as f64 * 2e4 - 1e4) as f32).to_bits(),
            5 => (((self.next() >> 11) as f64 / (1u64 << 53) as f64 * 6e9 - 3e9) as f32).to_bits(),
            6 => self.next() as u32 & 0x807F_FFFF,
            7 => (((self.next() >> 11) as f64 / (1u64 << 53) as f64 * 4.0 - 2.0) as f32).to_bits(),
            _ => (self.next() as i32 as f32).to_bits(),
        }
    }

    /// A float for the sine and cosine: modest angles, some larger, some edge values.
    fn fa(&mut self) -> u32 {
        match self.below(6) {
            0 => ((self.below(2001) as f32 - 1000.0) / 8.0).to_bits(),
            1 => {
                (((self.next() >> 11) as f64 / (1u64 << 53) as f64 * 20.0 - 10.0) as f32).to_bits()
            }
            2 => (((self.next() >> 11) as f64 / (1u64 << 53) as f64 * 2e6 - 1e6) as f32).to_bits(),
            3 => [0.0f32, -0.0, 1e-30, 3.15, 1.55, -3.15, 6.3][self.below(7) as usize].to_bits(),
            _ => self.f() & 0x8000_FFFF | 0x3F00_0000,
        }
    }

    /// A scale for an angle: between 0.5 and 1000.5.
    fn fs(&mut self) -> u32 {
        (((self.next() >> 11) as f64 / (1u64 << 53) as f64 * 1000.0 + 0.5) as f32).to_bits()
    }

    /// A whole number: random, small, or an extreme.
    fn i(&mut self) -> i32 {
        match self.below(6) {
            0 => self.next() as i32,
            1 => self.below(4001) as i32 - 2000,
            2 => {
                [0, 1, -1, i32::MIN, i32::MAX, 46340, 46341, -46341, 65536][self.below(9) as usize]
            }
            3 => self.below(2_000_001) as i32 - 1_000_000,
            4 => (self.next() as i32) >> (self.below(31) as u32),
            _ => self.below(33) as i32 - 16,
        }
    }

    /// A non-zero whole number up to 1000 in size (an angle's divisor: a tiny angle would
    /// leave only the x87's extended cosine to tell `1 - cos` from 0).
    fn c1000(&mut self) -> i32 {
        loop {
            let v = self.below(2001) as i32 - 1000;
            if v != 0 {
                return v;
            }
        }
    }

    /// A non-zero whole number.
    fn nz(&mut self) -> i32 {
        loop {
            let v = self.i();
            if v != 0 {
                return v;
            }
        }
    }

    /// A small count for loops.
    fn count(&mut self, max: u64) -> u32 {
        self.below(max + 1) as u32
    }
}

/// A plain helper's result as the reference's: the bits of a float result, a whole number or
/// a status word as it is.
pub(super) trait AsBits {
    type Out;
    fn into_bits(self) -> Self::Out;
}

impl AsBits for f32 {
    type Out = u32;
    fn into_bits(self) -> u32 {
        self.to_bits()
    }
}

macro_rules! as_they_are {
    ($($t:ty),*) => {$(
        impl AsBits for $t {
            type Out = $t;
            fn into_bits(self) -> $t {
                self
            }
        }
    )*};
}
as_they_are!(u8, u16, u32, i32, bool, f64);

impl<A: AsBits, B: AsBits> AsBits for (A, B) {
    type Out = (A::Out, B::Out);
    fn into_bits(self) -> Self::Out {
        (self.0.into_bits(), self.1.into_bits())
    }
}

impl<A: AsBits, B: AsBits, C: AsBits> AsBits for (A, B, C) {
    type Out = (A::Out, B::Out, C::Out);
    fn into_bits(self) -> Self::Out {
        (self.0.into_bits(), self.1.into_bits(), self.2.into_bits())
    }
}

impl<A: AsBits, const N: usize> AsBits for [A; N] {
    type Out = [A::Out; N];
    fn into_bits(self) -> Self::Out {
        self.map(AsBits::into_bits)
    }
}

/// Restores the control word at the end of a test, panicking or not.
struct Restore;

impl Drop for Restore {
    fn drop(&mut self) {
        set_control_word(0x027F);
    }
}

const CASES: usize = 30_000;

/// The control words the game's floats run under: nearest (double precision), chop, down
/// and up.
const MODES: [u16; 4] = [0x027F, 0x0E7F, 0x067F, 0x0A7F];

thread_local! {
    /// The running test's case sets.
    static SETS: core::cell::RefCell<Digests> = core::cell::RefCell::new(Digests::new());
}

/// Runs `body` `CASES` times at each of the control words, then checks the hashes of the
/// results `body` fed (`feed`) against the ones frozen for `test`.
fn each_mode(test: &str, seed: u64, modes: &[u16], mut body: impl FnMut(&mut G)) {
    let _restore = Restore;
    SETS.with(|s| *s.borrow_mut() = Digests::new());
    for &cw in modes {
        set_control_word(cw);
        let mut g = G::new(seed ^ u64::from(cw));
        for _ in 0..CASES {
            body(&mut g);
        }
    }
    SETS.with(std::cell::RefCell::take).check(test, FROZEN);
}

/// A case's result, into the hash of the set `name`.
fn feed<T: Canon>(name: &str, result: &T) {
    SETS.with(|s| s.borrow_mut().feed(name, result));
}

/// `diff!(g, "name", a = f, b = i; plain expression)`: draws each named input from the
/// generator (`G::f`, `G::i`, ..) and feeds the plain version's result.
macro_rules! diff {
    ($g:ident, $name:literal, $($v:ident = $gen:ident),*; $p:expr) => {{
        $(let $v = $g.$gen();)*
        feed($name, &AsBits::into_bits($p));
    }};
}

#[test]
fn float_arithmetic() {
    each_mode("float_arithmetic", 1, &MODES, |g| {
        diff!(g, "add", a = f, b = f; plain::add(a, b));
        diff!(g, "sub", a = f, b = f; plain::sub(a, b));
        diff!(g, "from_int", a = i; plain::from_int(a));
        diff!(g, "mul", a = f, b = f; plain::mul(a, b));
        diff!(g, "reciprocal_times", a = f, b = f; plain::reciprocal_times(a, b));
        diff!(g, "mirrored", x = f, a = f, b = f; plain::mirrored(x, a, b));
        diff!(g, "loaded", a = f; plain::loaded(a));
        diff!(g, "int_ratio", a = i, b = i; plain::int_ratio(a, b));
        diff!(g, "one_less_int_ratio", a = i, b = i; plain::one_less_int_ratio(a, b));
        diff!(g, "int_over_16", a = i; plain::int_over_16(a));
        diff!(g, "div", a = f, b = f; plain::div(a, b));
        diff!(g, "product_difference", a = f, b = f, c = f, d = f; plain::product_difference(a, b, c, d));
        diff!(g, "negated_product_difference", a = f, b = f, c = f, d = f; plain::negated_product_difference(a, b, c, d));
        diff!(g, "equal_or_unordered", a = f, b = f; plain::equal_or_unordered(a, b));
        diff!(g, "int_over_root", a = i, d = f; plain::int_over_root(a, d));
        diff!(g, "sqrt", a = f; plain::sqrt(a));
        diff!(g, "int_times", a = i, b = f; plain::int_times(a, b));
        diff!(g, "int_over", a = i, b = f; plain::int_over(a, b));
        diff!(g, "int_less", a = i, b = f; plain::int_less(a, b));
        diff!(g, "reciprocal", a = f; plain::reciprocal(a));
        diff!(g, "int_float_int_float", a = i, b = f, c = i, d = f; plain::int_float_int_float(a, b, c, d));
        diff!(g, "int_product_difference", a = i, b = i, c = i, d = i; plain::int_product_difference(a, b, c, d));
        diff!(g, "int_div", a = i, b = f; plain::int_over(a, b));
        diff!(g, "int_div_div_doubled", a = i, b = f, c = f; plain::int_div_div_doubled(a, b, c));
        diff!(g, "surface_share", a = i, b = f, c = f, d = i, e = f, f = f; plain::surface_share(a, b, c, d, e, f));
        diff!(g, "hypot_ratio", a = i, b = i, c = i, d = i; plain::hypot_ratio(a, b, c, d));
        diff!(g, "int_div_less", a = i, b = f, c = f; plain::int_div_less(a, b, c));
        diff!(g, "reciprocal_int_and_times", a = i, b = f; plain::reciprocal_int_and_times(a, b));
        diff!(g, "mul_and_div", a = f, b = f, c = f; plain::mul_and_div(a, b, c));
        diff!(g, "sub_div_mul", a = f, b = f, c = f, d = f; plain::sub_div_mul(a, b, c, d));
        diff!(g, "diff_ratio_mul", a = f, b = f, c = f, d = f, e = f; plain::diff_ratio_mul(a, b, c, d, e));
        diff!(g, "lerp_scaled_less", a = f, b = f, m = f, n = f, l = f; plain::lerp_scaled_less(a, b, m, n, Some(l)));
        diff!(g, "lerp_scaled_less (none)", a = f, b = f, m = f, n = f; plain::lerp_scaled_less(a, b, m, n, None::<u32>));
        diff!(g, "negated_sub", a = f, b = f; plain::negated_sub(a, b));
        diff!(g, "int_times_plus", a = i, b = f, c = f; plain::int_times_plus(a, b, c));
        diff!(g, "int_less_int", a = i, b = i; plain::int_less_int(a, b));
        diff!(g, "int_less_int_over", a = i, b = i, c = f; plain::int_less_int_over(a, b, c));
        diff!(g, "sub_div", a = f, b = f, c = f; plain::sub_div(a, b, c));
    });
}

#[test]
fn float_arithmetic_b() {
    each_mode("float_arithmetic_b", 2, &MODES, |g| {
        diff!(g, "mul2", a = f, b = f, c = f; plain::mul2(a, b, c));
        diff!(g, "circle_curve", i = i, a = f, b = f; plain::circle_curve(i, a, b, false));
        diff!(g, "circle_curve (negate)", i = i, a = f, b = f; plain::circle_curve(i, a, b, true));
        diff!(g, "negated", a = f; plain::negated(a));
        diff!(g, "sub_mul", b = f, a = f, k = f; plain::sub_mul(b, a, k));
        diff!(g, "frac_step", y = f, k = f, s = f, x = f; plain::frac_step(y, k, s, x));
        diff!(g, "compare_status", a = f, b = f; plain::compare_status(a, b));
        diff!(g, "split_x", a = f, b = f, c = f, x0 = f, x2 = f; plain::split_x(a, b, c, x0, x2));
        diff!(g, "half_and_middle", b = f, a = f, k = f; plain::half_and_middle(b, a, k));
        diff!(g, "pi_over", n = i; plain::pi_over(n));
        diff!(g, "mul_add", a = f, b = f, c = f; plain::mul_add(a, b, c));
        diff!(g, "int_div_int", a = i, n = i; plain::int_ratio(a, n));
        diff!(g, "sub_80", a = f, b = f; plain::sub_80(a, b));
        diff!(g, "int_sub_80", a = i, b = f; plain::int_sub_80(a, b));
    });
}

#[test]
fn whole_numbers_after_division_or_transcendental() {
    each_mode(
        "whole_numbers_after_division_or_transcendental",
        9,
        &MODES,
        |g| {
            diff!(g, "ratio_scaled_to_int", a = f, b = f, m = i, n = i; plain::ratio_scaled_to_int(a, b, m, n));
            diff!(g, "mul_div_div_to_int", a = f, b = f, c = f, d = f; plain::mul_div_div_to_int(a, b, c, d));
            diff!(g, "share_left_to_int", a = i, b = i, c = f; plain::share_left_to_int(a, b, c));
            diff!(g, "angle_scaled_to_int", y = i, x = i, a = f, b = f; plain::angle_scaled_to_int(y, x, a, b));
            diff!(g, "angle_scaled3_to_int", y = i, x = i, a = f, b = f, c = f; plain::angle_scaled3_to_int(y, x, a, b, c));
            diff!(g, "angle_scaled5_to_int", y = i, x = i, a = f, b = f; plain::angle_scaled5_to_int(y, x, a, b));
            diff!(g, "sine_times_plus_to_int", a = fa, b = f, c = f; plain::sine_times_plus_to_int(a, b, c));
        },
    );
}

#[test]
fn results_as_whole_numbers() {
    each_mode("results_as_whole_numbers", 7, &MODES, |g| {
        diff!(g, "to_int", a = f; plain::to_int(a));
        diff!(g, "mul_to_int", a = f, b = f; plain::mul_to_int(a, b));
        diff!(g, "int_hypot", a = i, b = i; plain::int_hypot(a, b));
        diff!(g, "int_times_to_int", a = i, b = f; plain::int_times_to_int(a, b));
        diff!(g, "weighted3_to_int", a = i, b = i, c = i, x = f, y = f, z = f; plain::weighted3_to_int(a, b, c, x, y, z));
        diff!(g, "negated_square_times_plus_to_int", a = f, b = f, c = f; plain::negated_square_times_plus_to_int(a, b, c));
        diff!(g, "length_plus_int", x = i, y = i, z = i, s = i; plain::length_plus_int(x, y, z, s));
        diff!(g, "round_float", a = f; plain::round_float(a));
        diff!(g, "float_less_int_to_int", a = f, n = i; plain::float_less_int_to_int(a, n));
        diff!(g, "int_length3", a = i, b = i, c = i; plain::int_length3(a, b, c));
        diff!(g, "int_sum", a = i, b = i; plain::int_sum(a, b));
        diff!(g, "compare_squares", a = i, b = i, c = i, d = i; plain::compare_squares(a, b, c, d));
        diff!(g, "add_to_int", a = f, b = f; plain::add_to_int(a, b));
    });
}

/// The comparisons see no-number inputs too.
#[test]
fn comparisons_with_nan() {
    let nans = [
        0x7FC0_0000u32,
        0xFFC0_0000,
        0x7F80_0001,
        0x7F80_0000,
        0xFF80_0000,
    ];
    each_mode("comparisons_with_nan", 3, &MODES, |g| {
        let n = nans[g.below(5) as usize];
        let a = g.f();
        for (x, y) in [(n, a), (a, n), (n, n)] {
            feed("equal_or_unordered", &plain::equal_or_unordered(x, y));
            feed("compare_status", &plain::compare_status(x, y));
        }
        let mut p = [g.f(), g.f(), g.f()];
        let z = n;
        let two = plain::normalise_in_place((&mut p[..3]).try_into().unwrap(), z);
        let two = (two.0.to_bits(), two.1);
        feed("normalise_in_place (nan zero)", &(two, p.to_vec()));
    });
}

#[test]
fn trigonometry() {
    each_mode("trigonometry", 4, &MODES, |g| {
        // the sines and cosines: within one ulp of a float (2 ulps for the two-step ones)
        diff!(g, "sine_cosine", angle = i, a = fs, b = fs; plain::sine_cosine(angle % 100_000, a, b));
        diff!(g, "sine_of_pi_fraction", n = c1000, d = c1000; plain::sine_of_pi_fraction(n, d));
        diff!(g, "cosine_step", c = c1000, m = i, t = f, a = f; plain::cosine_step(c, m, t, a));
        diff!(g, "less_cosine", a = fa, s = fa, h = f, m = f; plain::less_cosine(a, Some(s), h, m));
        diff!(g, "less_cosine (none)", a = fa, h = f, m = f; plain::less_cosine(a, None::<u32>, h, m));
        diff!(g, "sine", a = fa; plain::sine(a));
        diff!(g, "cosine", a = fa; plain::cosine(a, false));
        diff!(g, "cosine (negate)", a = fa; plain::cosine(a, true));
        let step = g.fa();
        let count = g.count(40);
        let mut p = vec![0xAAAA_AAAAu32; 41];
        plain::sine_table(step, &mut p[..count as usize]);
        feed("sine_table", &p);
        let radius = g.f();
        let (c, d) = (g.nz() % 1000, g.fa());
        let count = 1 + g.count(11);
        let mut p = vec![0xAAAA_AAAAu32; 40];
        plain::circle_points(radius, &mut p, count, c, d);
        feed("circle_points", &p);
    });
}

#[test]
fn pointer_functions() {
    each_mode("pointer_functions", 5, &MODES, |g| {
        let a = [g.f(), g.f(), g.f()];
        // a matrix's worth of floats (9) for the strided ones
        let m: Vec<u32> = (0..9).map(|_| g.f()).collect();
        // `row_times_column` reads b + 0x10 and b + 0x20: a 12-dword matrix
        let b: Vec<u32> = (0..12).map(|_| g.f()).collect();
        let mut p = vec![0xAAAA_AAAAu32];
        // SAFETY: three dwords of `a`, twelve of `b`, one of `p`.
        unsafe { plain::row_times_column(a.as_ptr(), b.as_ptr(), p.as_mut_ptr()) };
        feed("row_times_column", &p);
        let mut p = vec![0xAAAA_AAAAu32];
        let cells = |v: &[u32]| {
            v.iter()
                .map(|&x| crate::session::Dword::new(x))
                .collect::<Vec<_>>()
        };
        p[0] = crate::scene::matrix3::row_times_column_3(&cells(&a)[..], &cells(&m)[..]);
        feed("row_times_column_3", &p);

        let (x, start) = (g.f(), g.f());
        let mut p = vec![start];
        // SAFETY: the local `x` and one dword of `p`.
        unsafe { plain::add_into(&raw const x, p.as_mut_ptr()) };
        feed("add_into", &p);

        // SAFETY: three dwords of `a`.
        feed("length", &unsafe { plain::length(a.as_ptr()) });

        let zero = [0u32, 0x8000_0000, g.f(), 0x3F80_0000][g.below(4) as usize];
        let mut v = [g.f(), g.f(), g.f()];
        if g.below(8) == 0 {
            v = [0, 0x8000_0000, 0];
        }
        let mut p = v;
        let two = plain::normalise_in_place((&mut p[..3]).try_into().unwrap(), zero);
        let two = (two.0.to_bits(), two.1);
        feed("normalise_in_place", &(two, p.to_vec()));

        let (s, c) = (g.fa(), g.fa());
        let (x, z) = (g.f(), g.f());
        let (mut px, mut pz) = (x, z);
        plain::turn_in_place(s, c, &mut px, &mut pz);
        feed("turn_in_place", &vec![px, pz]);

        let row = [g.f(), g.f(), g.f()];
        let k = g.f();
        feed(
            "scale_row",
            &plain::scale_row(row, k).map(f32::to_bits).to_vec(),
        );

        let count = g.count(16);
        let values: Vec<i32> = (0..16).map(|_| g.i()).collect();
        let (last, first, by) = (g.i(), g.i(), g.nz());
        let mut p = values.clone();
        plain::ramp_down(&mut p[..count as usize], last, first, by);
        feed("ramp_down", &p);
    });
}

#[test]
fn pointer_functions_to_int() {
    each_mode("pointer_functions_to_int", 8, &MODES, |g| {
        let pt = [g.i(), g.i(), g.i()];
        let mat: Vec<u32> = (0..9).map(|_| g.f()).collect();
        let mut p = vec![0x5555_5555i32; 3];
        let cells = |v: &[u32]| {
            v.iter()
                .map(|&x| crate::session::Dword::new(x))
                .collect::<Vec<_>>()
        };
        let turn = |from: &[crate::session::Dword], to: &[crate::session::Dword]| {
            crate::scene::matrix3::turn_int_point(from, &cells(&mat)[..], to, 0);
        };
        let (from, to) = (cells(&pt.map(|x| x as u32)), cells(&[0x5555_5555; 3]));
        turn(&from, &to);
        for (o, c) in p.iter_mut().zip(&to) {
            *o = c.get() as i32;
        }
        feed("turn_int_point", &p);
        // out over p
        let mut p = pt;
        let in_place = cells(&pt.map(|x| x as u32));
        turn(&in_place, &in_place);
        for (o, c) in p.iter_mut().zip(&in_place) {
            *o = c.get() as i32;
        }
        feed("turn_int_point (in place)", &p.to_vec());
    });
}

/// The 80-bit conversions: a double there and back is the same double, and both ways give
/// what `fld qword; fstp tbyte` and `fld tbyte; fstp qword` (rounding to nearest even) gave.
#[test]
fn eighty_bit_conversions() {
    let mut g = G::new(6);
    let mut sets = Digests::new();
    for _ in 0..200_000 {
        let bits = match g.below(4) {
            0 => g.next(),
            1 => g.next() & 0x800F_FFFF_FFFF_FFFF,
            2 => g.next() & 0x80FF_FFFF_FFFF_FFFF,
            _ => (u64::from(g.f()) << 32) | (g.next() & 0xFFFF_FFFF),
        };
        let x = f64::from_bits(bits);
        if x.is_nan() {
            continue;
        }
        let v = f80(X(x));
        assert_eq!(from_f80(v).0.to_bits(), bits, "round trip of {bits:#x}");
        sets.feed("f64_to_f80", &v);
        let mut raw = [0u8; 10];
        raw[..8].copy_from_slice(&(g.next() | (1 << 63)).to_le_bytes());
        let exp = (0x3C00 + g.below(0x0800) as u16) | ((g.below(2) as u16) << 15);
        raw[8..].copy_from_slice(&exp.to_le_bytes());
        sets.feed("f80_to_f64", &from_f80(raw).0);
    }
    sets.check("eighty_bit_conversions", FROZEN);
}

/// The results' hashes, frozen on 2026-11-13 (`RC2K_FREEZE_OUT` to make them again).
const FROZEN: &[(&str, u64)] = &[
    ("comparisons_with_nan::compare_status", 0x9b16fe6ad7bcbf5a),
    (
        "comparisons_with_nan::equal_or_unordered",
        0x96487a7ee563a67f,
    ),
    (
        "comparisons_with_nan::normalise_in_place (nan zero)",
        0x1f4bc0b94e085265,
    ),
    ("eighty_bit_conversions::f64_to_f80", 0x20ac673fe8802d5e),
    ("eighty_bit_conversions::f80_to_f64", 0x7d38d5ccb7e087a7),
    ("float_arithmetic::add", 0x3cd1a8611fd29fbe),
    ("float_arithmetic::diff_ratio_mul", 0x87e6a4055ff96e82),
    ("float_arithmetic::div", 0x700662239c767b4e),
    ("float_arithmetic::equal_or_unordered", 0x456614102cebce85),
    ("float_arithmetic::from_int", 0x44a2528343ef2d77),
    ("float_arithmetic::hypot_ratio", 0x29bb34b2c419220d),
    ("float_arithmetic::int_div", 0x3439771940f358a9),
    ("float_arithmetic::int_div_div_doubled", 0xe4f661315aa5dd48),
    ("float_arithmetic::int_div_less", 0x8db61ebaa4e3598d),
    ("float_arithmetic::int_float_int_float", 0x5e01fbc72618b07c),
    ("float_arithmetic::int_less", 0xe5b67691ee614672),
    ("float_arithmetic::int_less_int", 0x90f59bff99269dfe),
    ("float_arithmetic::int_less_int_over", 0xa87532ddff546602),
    ("float_arithmetic::int_over", 0x4b799346708c910c),
    ("float_arithmetic::int_over_16", 0x746f738d752512e2),
    ("float_arithmetic::int_over_root", 0x9b12cc73eac1059e),
    (
        "float_arithmetic::int_product_difference",
        0x427b7879ac812ad4,
    ),
    ("float_arithmetic::int_ratio", 0x24c437b36052393d),
    ("float_arithmetic::int_times", 0xffd3701126b79056),
    ("float_arithmetic::int_times_plus", 0x73d202b11822e63a),
    (
        "float_arithmetic::lerp_scaled_less (none)",
        0xbf0f5442c96048ee,
    ),
    ("float_arithmetic::lerp_scaled_less", 0x68efe6991f8cce28),
    ("float_arithmetic::loaded", 0xa481aebe0fb6d489),
    ("float_arithmetic::mirrored", 0x70e63a4ee465d221),
    ("float_arithmetic::mul", 0xa5cbfab8ddf07acd),
    ("float_arithmetic::mul_and_div", 0x28146f8dde251f45),
    (
        "float_arithmetic::negated_product_difference",
        0x9e89c026559d64a3,
    ),
    ("float_arithmetic::negated_sub", 0x94be06cd18132ce2),
    ("float_arithmetic::one_less_int_ratio", 0xb83980444c7534ef),
    ("float_arithmetic::product_difference", 0x14560bb5c6062804),
    ("float_arithmetic::reciprocal", 0xf4f9f8c7bbfa7649),
    (
        "float_arithmetic::reciprocal_int_and_times",
        0x7e283a391ee649b4,
    ),
    ("float_arithmetic::reciprocal_times", 0x1fef69a2eef4ef0a),
    ("float_arithmetic::sqrt", 0xa2cd58b035d015c2),
    ("float_arithmetic::sub", 0xbfcf66d9f73758f6),
    ("float_arithmetic::sub_div", 0xd873f715395e62e8),
    ("float_arithmetic::sub_div_mul", 0x04502c9c0c773a25),
    ("float_arithmetic::surface_share", 0xbb625d2545125395),
    (
        "float_arithmetic_b::circle_curve (negate)",
        0x1cddbf648b8e4f3a,
    ),
    ("float_arithmetic_b::circle_curve", 0xcda8b208eb72790c),
    ("float_arithmetic_b::compare_status", 0x065fe53ab38040c9),
    ("float_arithmetic_b::frac_step", 0xfaf99518357f269e),
    ("float_arithmetic_b::half_and_middle", 0x6abb345c39313a57),
    ("float_arithmetic_b::int_div_int", 0x7dbd241312a67284),
    ("float_arithmetic_b::int_sub_80", 0xada7ae86c33c843b),
    ("float_arithmetic_b::mul2", 0x0a1a1c8cb1b91d8d),
    ("float_arithmetic_b::mul_add", 0xb3dd56b840f788e1),
    ("float_arithmetic_b::negated", 0x6a3aee3bb636cf63),
    ("float_arithmetic_b::pi_over", 0xf94997bcb9e8ffab),
    ("float_arithmetic_b::split_x", 0x91c0d8a16d7dbdc2),
    ("float_arithmetic_b::sub_80", 0x44a0dd939ca25d38),
    ("float_arithmetic_b::sub_mul", 0x5c8bb78a582b8f17),
    ("pointer_functions::add_into", 0x67428855e627221b),
    ("pointer_functions::length", 0x456708023bc8a010),
    ("pointer_functions::normalise_in_place", 0x6d66dc2c13accffc),
    ("pointer_functions::ramp_down", 0x362aa0a575bfaf4e),
    ("pointer_functions::row_times_column", 0xe532e9ddb920e4c8),
    ("pointer_functions::row_times_column_3", 0x1d79f84679f9c9d2),
    ("pointer_functions::scale_row", 0xff3857505a674a76),
    ("pointer_functions::turn_in_place", 0x587b41e331cef2ba),
    (
        "pointer_functions_to_int::turn_int_point (in place)",
        0x390e6240c1331cb1,
    ),
    (
        "pointer_functions_to_int::turn_int_point",
        0x390e6240c1331cb1,
    ),
    ("results_as_whole_numbers::add_to_int", 0xda85f964d76f2ce6),
    (
        "results_as_whole_numbers::compare_squares",
        0xa3b2b3823aef6872,
    ),
    (
        "results_as_whole_numbers::float_less_int_to_int",
        0x55f31fae73827369,
    ),
    ("results_as_whole_numbers::int_hypot", 0x7003c7171f051ae9),
    ("results_as_whole_numbers::int_length3", 0xd1492932d7802c43),
    ("results_as_whole_numbers::int_sum", 0xe21e8da9b6417a4f),
    (
        "results_as_whole_numbers::int_times_to_int",
        0x5eb2f0444ce6d3af,
    ),
    (
        "results_as_whole_numbers::length_plus_int",
        0x7b1b2cb2a8fbd8f3,
    ),
    ("results_as_whole_numbers::mul_to_int", 0xf4970ee8a985cadf),
    (
        "results_as_whole_numbers::negated_square_times_plus_to_int",
        0x8b62ecf2938155f4,
    ),
    ("results_as_whole_numbers::round_float", 0xa414b9a969d79e59),
    ("results_as_whole_numbers::to_int", 0x93eb2ce1540623fe),
    (
        "results_as_whole_numbers::weighted3_to_int",
        0x84fbfb233e0c97cb,
    ),
    ("trigonometry::circle_points", 0xc6f6daba686784e4),
    ("trigonometry::cosine (negate)", 0x28a0fa118f69d235),
    ("trigonometry::cosine", 0xd39e75afdc6e601b),
    ("trigonometry::cosine_step", 0x50a2b1f4490d9032),
    ("trigonometry::less_cosine (none)", 0xbf18a66a830995fa),
    ("trigonometry::less_cosine", 0xd1eb8a65b9a0a035),
    ("trigonometry::sine", 0x4bf10853dbbaf78c),
    ("trigonometry::sine_cosine", 0x2564f29408750244),
    ("trigonometry::sine_of_pi_fraction", 0x3aea4837c5865c06),
    ("trigonometry::sine_table", 0xf7fb521fbefba90c),
    (
        "whole_numbers_after_division_or_transcendental::angle_scaled3_to_int",
        0xb60204bf075cf842,
    ),
    (
        "whole_numbers_after_division_or_transcendental::angle_scaled5_to_int",
        0xb787ee530be5c03f,
    ),
    (
        "whole_numbers_after_division_or_transcendental::angle_scaled_to_int",
        0xba04639dfeff86c3,
    ),
    (
        "whole_numbers_after_division_or_transcendental::mul_div_div_to_int",
        0xe1a8b50ee2bc8a1a,
    ),
    (
        "whole_numbers_after_division_or_transcendental::ratio_scaled_to_int",
        0x93b1f2f94a4e19d7,
    ),
    (
        "whole_numbers_after_division_or_transcendental::share_left_to_int",
        0x086b8cf73290baf2,
    ),
    (
        "whole_numbers_after_division_or_transcendental::sine_times_plus_to_int",
        0x6c75331abe53c4ae,
    ),
];
