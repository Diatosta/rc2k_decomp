//! The frozen comparisons of the helpers in `math/x87.rs` from its "part B" line: until
//! 2026-11-13 each plain version ran against the original's x87 instructions on seeded
//! inputs at the four rounding modes (bit for bit, any NaN equal to any NaN, a status word's
//! C3 / C2 / C0; a few within an ulp or two). The last run passed; the plain versions'
//! results on the same inputs are now kept as one hash per function and control word
//! (`test_support::frozen`, table `FROZEN` at the end).

mod diff {
    use crate::test_support::frozen::Digests;
    use std::cell::Cell;

    /// A test buffer's dwords as cells.
    fn cells_of(words: &[u32]) -> Vec<Cell<u32>> {
        words.iter().map(|&w| Cell::new(w)).collect()
    }
    use crate::math::x87 as plain;
    use crate::math::x87::{control_word, set_control_word};

    /// Cases per function and control word.
    const CASES: u32 = 20_000;

    #[derive(Clone)]
    struct Rng(u64);

    const EDGES: [f32; 40] = [
        0.0,
        -0.0,
        1.0,
        -1.0,
        0.5,
        -0.5,
        1.5,
        2.5,
        -1.5,
        -2.5,
        0.25,
        3.5,
        1e-45,
        1.1754942e-38,
        -1e-45,
        3.4028235e38,
        -3.4028235e38,
        1e30,
        1e-30,
        2147483520.0,
        2147483648.0,
        -2147483648.0,
        -2147483520.0,
        4294967296.0,
        16777216.0,
        16777217.0,
        -16777216.0,
        8388607.5,
        std::f32::consts::PI,
        std::f32::consts::FRAC_PI_2,
        2.0 * std::f32::consts::PI,
        -std::f32::consts::PI,
        0.1,
        100.0,
        1000.0,
        65536.0,
        0.001,
        255.0,
        -0.001,
        7.0,
    ];

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }
        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
        /// A float's bits: edge values, random finite patterns, integers, halves,
        /// moderate values, values near the i32 range, denormals.
        fn fl(&mut self) -> u32 {
            match self.below(14) {
                0 | 1 => EDGES[self.below(EDGES.len() as u64) as usize].to_bits(),
                2 => {
                    let b = self.next() as u32;
                    if (b >> 23) & 0xFF == 0xFF {
                        b & !(1 << 30)
                    } else {
                        b
                    }
                }
                3 => (self.int_small() as f32).to_bits(),
                4 => (self.int_small() as f32 + 0.5).to_bits(),
                5 => (self.int_small() as f32 * 0.5 + 0.25).to_bits(),
                6..=9 => self.fm(),
                10 => {
                    let base = if self.below(2) == 0 {
                        2147483648.0f32
                    } else {
                        -2147483648.0
                    };
                    (base + (self.int_small() % 300) as f32 * 128.0).to_bits()
                }
                11 => (self.next() as u32) & 0x807F_FFFF,
                12 => (self.int_small() as f32 / 1024.0).to_bits(),
                _ => {
                    (self.next() as u32 & 0x8000_0000)
                        | (0x2000_0000 + (self.next() as u32 & 0x1FFF_FFFF) % 0x2000_0000)
                }
            }
        }
        /// A moderate float (|x| below a few thousand), often a whole or half number.
        fn fm(&mut self) -> u32 {
            let v = (self.below(4_000_000) as f64 / 1000.0 - 2000.0) as f32;
            match self.below(5) {
                0 => v.round().to_bits(),
                1 => (v.round() + 0.5).to_bits(),
                2 => (v / 64.0).to_bits(),
                _ => v.to_bits(),
            }
        }
        /// An angle for `sin` / `cos`: up to a few thousand radians, often near a multiple
        /// of a quarter turn.
        fn ft(&mut self) -> u32 {
            match self.below(4) {
                0 => {
                    let k = (self.below(2001) as i32 - 1000) as f32;
                    let near = (k * std::f32::consts::FRAC_PI_2).to_bits();
                    near.wrapping_add(self.below(7) as u32).wrapping_sub(3)
                }
                1 => ((self.below(20_000_000) as f64 / 1000.0 - 10_000.0) as f32).to_bits(),
                2 => ((self.below(2_000_000) as f64 / 100_000.0 - 10.0) as f32).to_bits(),
                _ => {
                    EDGES[self.below(EDGES.len() as u64) as usize].to_bits() & !0x4000_0000
                        | 0x3000_0000
                }
            }
        }
        fn int_small(&mut self) -> i32 {
            match self.below(3) {
                0 => self.below(201) as i32 - 100,
                1 => self.below(200_001) as i32 - 100_000,
                _ => self.below(33_554_433) as i32 - 16_777_216,
            }
        }
        /// An integer: random, small, or at an edge.
        fn ii(&mut self) -> i32 {
            match self.below(8) {
                0 => [
                    0,
                    1,
                    -1,
                    i32::MAX,
                    i32::MIN,
                    1 << 24,
                    (1 << 24) + 1,
                    -(1 << 30),
                    1 << 30,
                    46341,
                ][self.below(10) as usize],
                1 | 2 => self.next() as i32,
                _ => self.int_small(),
            }
        }
        /// A small integer (a tick count, a size).
        fn is(&mut self) -> i32 {
            self.int_small() % 5000
        }
        fn bl(&mut self) -> bool {
            self.below(2) == 0
        }
        fn ofl(&mut self) -> Option<u32> {
            if self.bl() { Some(self.fl()) } else { None }
        }
        /// A double's bits (a constant of the original).
        fn d64(&mut self) -> u64 {
            match self.below(3) {
                0 => (self.below(2_000_000) as f64 / 1000.0 - 1000.0).to_bits(),
                1 => (self.below(2_000_000) as f64 / 1e7 - 0.1).to_bits(),
                _ => {
                    // a random double, its exponent kept within 2^-100 .. 2^100 (beyond
                    // that the products leave the double range, which the x87's stack does not)
                    let b = self.next();
                    let e = 1023 - 100 + (b >> 52) % 201;
                    (b & !(0x7FF << 52)) | (e << 52)
                }
            }
        }
        /// A divisor for `fprem` whose exponent is less than 64 below an integer's (the
        /// x87 reduces only partly beyond that).
        fn fd(&mut self) -> u32 {
            loop {
                let b = self.fl();
                let x = f32::from_bits(b).abs();
                if x == 0.0 || x >= 2f32.powi(-33) {
                    return b;
                }
            }
        }
        /// A small float, as the multiplier of a wave's argument.
        fn fk(&mut self) -> u32 {
            ((self.below(2000) as f64 / 100.0 - 10.0) as f32).to_bits()
        }
        fn k3(&mut self) -> [u32; 3] {
            [self.fk(), self.fk(), self.fk()]
        }
        fn a3(&mut self) -> [u32; 3] {
            [self.fl(), self.fl(), self.fl()]
        }
        fn m3(&mut self) -> [u32; 3] {
            [self.fm(), self.fm(), self.fm()]
        }
        fn a7(&mut self) -> [u32; 7] {
            [
                self.fl(),
                self.fl(),
                self.fl(),
                self.fl(),
                self.fl(),
                self.fl(),
                self.fl(),
            ]
        }
        fn a33(&mut self) -> [[u32; 3]; 3] {
            [self.a3(), self.a3(), self.a3()]
        }
        fn a32(&mut self) -> [[u32; 2]; 3] {
            [
                [self.fl(), self.fl()],
                [self.fl(), self.fl()],
                [self.fl(), self.fl()],
            ]
        }
        fn k4(&mut self) -> [u32; 4] {
            [self.fl(), self.fl(), self.fl(), self.fl()]
        }
        /// 80-bit values as the x87 holds them at 53-bit precision (what a `fstp tbyte`
        /// of this module leaves).
        fn e80(&mut self) -> [u8; 10] {
            let f = f64::from_bits(match self.below(3) {
                0 => f64::from(f32::from_bits(self.fl())).to_bits(),
                1 => (f64::from(f32::from_bits(self.fm())) * f64::from(f32::from_bits(self.fm())))
                    .to_bits(),
                _ => (f64::from(f32::from_bits(self.fl()))
                    + f64::from(f32::from_bits(self.fm())) * 1e-9)
                    .to_bits(),
            });
            to_f80(f)
        }
        fn p80(&mut self) -> [[u8; 10]; 3] {
            [self.e80(), self.e80(), self.e80()]
        }
    }

    /// `fld qword; fstp tbyte`: a double as the 80-bit format (`x87_float::f80`, itself
    /// frozen against the instructions in `x87_frozen_a`).
    fn to_f80(x: f64) -> [u8; 10] {
        crate::math::x87_float::f80(crate::math::x87_float::X(x))
    }

    /// The generator and the hashes of the cases run so far.
    struct T {
        rng: Rng,
        sets: Digests,
    }

    impl T {
        fn new(seed: u64) -> T {
            T {
                rng: Rng(seed),
                sets: Digests::new(),
            }
        }
    }

    /// Runs the plain version on `CASES` generated arguments and hashes its results.
    macro_rules! diff {
        ($t:expr, $name:ident ( $($g:ident),* )) => { diff!($t, $name($($g),*) tol 0) };
        ($t:expr, $name:ident ( $($g:ident),* ) tol $ulps:expr) => {{
            for _ in 0..CASES {
                let got = crate::math::x87_frozen_a::AsBits::into_bits(plain::$name($($t.rng.$g()),*));
                $t.sets.feed(stringify!($name), &got);
            }
        }};
    }

    /// The control word at `w` for the closure, 0x027F after.
    fn with_control(w: u16, f: impl FnOnce()) {
        struct Restore;
        impl Drop for Restore {
            fn drop(&mut self) {
                set_control_word(0x027F);
            }
        }
        let _restore = Restore;
        set_control_word(w);
        assert_eq!(control_word(), w);
        f();
    }

    fn matrix(r: &mut Rng) -> [u32; 16] {
        let mut m = [0u32; 16];
        for x in &mut m {
            *x = if r.below(4) == 0 { r.fl() } else { r.fm() };
        }
        m
    }

    fn pointer_functions(t: &mut T) {
        for _ in 0..CASES {
            let mut r = [0u32; 12];
            for x in &mut r {
                *x = t.rng.fm();
            }
            let (a, b) = (t.rng.fl(), t.rng.fl());
            t.sets
                .feed("scale_record", &plain::scale_record_bits(r, a, b));

            let m = matrix(&mut t.rng);
            let mc = cells_of(&m);
            let (p, k, d, lift) = (t.rng.a3(), t.rng.fl(), t.rng.fl(), t.rng.fl());
            let got = plain::turned_place_less(p, k, d, lift, &mc[..]);
            t.sets.feed(
                "turned_place_less",
                &crate::math::x87_frozen_a::AsBits::into_bits(got),
            );

            let got = plain::turned_place(p, lift, &mc[..]);
            t.sets.feed(
                "turned_place",
                &crate::math::x87_frozen_a::AsBits::into_bits(got),
            );

            let w = t.rng.a3();
            let got = plain::crossed_turned(p, &cells_of(&w)[..], &mc[..]);
            t.sets.feed(
                "crossed_turned",
                &crate::math::x87_frozen_a::AsBits::into_bits(got),
            );

            let p80 = t.rng.p80();
            let got = plain::column_turn(&p80, &mc[..]).map(f32::to_bits);
            t.sets.feed("column_turn", &got);
        }
    }

    fn all_functions(t: &mut T) {
        diff!(t, sub_compare(fl, fl, bl, fl));
        diff!(t, random_step(ii, a7));
        diff!(t, int_plus_less_squared(ii, fl, fl));
        diff!(t, int_times_plus_on(ii, fl, fl, fl, fl));
        diff!(t, sub_thrice(fl, fl));
        diff!(t, int_plus(ii, fl));
        diff!(t, ease(fl, fl, fl));
        diff!(t, int_plus_to_int(ii, fl));
        diff!(t, zero_less_int_plus_int(ii, ii));
        diff!(t, less_int_plus_int(fl, ii, ii));
        diff!(t, two_pi_less(fl));
        diff!(t, two_pi_less(ft));
        diff!(t, int_times_plus_times(ii, fl, fl, fl));
        diff!(t, over_int(fl, ii));
        diff!(t, zero_times_plus_times(fl, fl, fl, fl, bl));
        diff!(t, prem_plus(ii, fd, fl));
        diff!(t, grown(ii, d64, fl, fl));
        diff!(t, rate_to_int(fl, fl, fl, fl, fl, fl, fl));
        diff!(t, pair_sum_80(ii, fl, ii, fl, fl));
        diff!(t, ext_less(e80, fl));
        diff!(t, three_products(a33));
        diff!(t, times_kept_abs_to_int(fl, fl, fl));
        diff!(t, blend_kept_abs_to_int(fl, fl, fl, fl, fl));
        diff!(t, times_less_abs_times(fl, fl, ofl, fl));
        diff!(t, less_plus_int_times(fl, fl, fl, ii));
        diff!(t, times_int_plus_times_80(fl, fl, ii, ofl, fl));
        diff!(t, pair_sum_less(ii, fl, ii, fl, fl, fl));
        diff!(t, product_plus(fl, fl, fl, fl));
        diff!(t, float_times_int_to_int(fl, ii));
        diff!(t, speed_rate_to_int(fl, fl, fl, fl, fl, fl));
        diff!(t, abs_sum_to_int_kept(fl, fl, fl, fl));
        diff!(t, abs_blend_compare(fl, fl, fl, fl, fl, fl));
        diff!(t, times_plus_to_int(fl, fl, fl));
        diff!(t, plus_int_times_80(fl, fl, ii));
        diff!(t, three_products_80(a33));
        diff!(t, times_compare(fl, fl, fl));
        diff!(t, times_int_to_int(fl, fl, ii));
        diff!(t, int_times_plus_and_negated(ii, fl, fl));
        diff!(t, int_times_less_less(ii, fl, fl, fl));
        diff!(t, abs_sum_times_to_int(fl, fl, fl));
        diff!(t, drifted(ii, fl, a3, ii, a3));
        diff!(t, one_less_inverse(fl, fl, fl));
        diff!(t, times_over_int(fl, fl, ii));
        diff!(t, mul_80(fl, fl));
        diff!(t, int_times_int_to_int(ii, ii));
        diff!(t, sum4_times(fl, fl, fl, fl, fl));
        diff!(t, length_and_rate(a3, fl, fl, fl, fl));
        diff!(t, length_and_rate(m3, fl, fl, fl, fl));
        diff!(t, normalised3(a3, fl));
        diff!(t, normalised3(m3, fl));
        diff!(t, unit_and_length(a3));
        diff!(t, unit_and_length(m3));
        diff!(t, negated_less_plus_plus(fl, fl, fl, fl));
        diff!(t, sine_and_cosine(ft) tol 1);
        diff!(t, turn_yz(fl, fl, fl, fl));
        diff!(t, int_times_plus_int_to_int(ii, fl, ii));
        diff!(t, blend3_80(ii, ii, ii, a3));
        diff!(t, angle_and_wave(fm, is, fm, fm, fm) tol 1);
        diff!(t, angle_and_wave(ft, is, fm, fm, fm) tol 1);
        diff!(t, int_lerp_to_int(ii, ii, fl));
        diff!(t, int_over_int_times(ii, ii, fl));
        diff!(t, clip_lerp3(fl, fl, fl, a32));
        diff!(t, slot_project(fl, fl, fl, fl, fl, fl, fl));
        diff!(t, int_times_times(ii, fl, fl));
        diff!(t, toward(fl, fl, fl));
        diff!(t, diff_times_plus_times(fl, fl, fl, fl, fl));
        diff!(t, particle_square(fl, fl, fl, k4, is));
        diff!(t, plus_times_int(fl, fl, ii));
        diff!(t, plus_sum_times_int(fl, fl, fl, ii));
        diff!(t, less_own_times(fl, fl));
        diff!(t, int_times_int_plus_int(ii, ii, ii));
        diff!(t, int_times3_wave_plus(is, k3, fm, fm, bl) tol 1);
        diff!(t, int_times3_wave_plus(is, k3, fl, fm, bl) tol 1);
        diff!(t, int_times_plus_times_plus(ii, fl, fl, fl, fl));
        diff!(t, turned_sprite(fl, fl, fl, k4, is, fl, fl));
        diff!(t, add_80(fl, fl));
        diff!(t, turn_about(fl, fl, fl, fl, fl, fl));
        diff!(t, times_subr_to_int(fl, fl, fl));
        diff!(t, times_plus_80(fl, fl, fl, bl));
        diff!(t, times_int_plus_int_to_int(fl, ii, ii));
        diff!(t, damp(fl, fl, ii));
        diff!(t, approach(ii, fl, fl, ii));
        diff!(t, less_times_int(fl, fl, ii));
        diff!(t, plus_product_to_int(fl, fl, fl));
        diff!(t, int_times_times_to_int(ii, fl, fl));
        diff!(t, times_plus_negated_80(fl, fl, fl));
        diff!(t, less_times_plus_negated_80(fl, fl, fl, fl));
        diff!(t, fraction_times_to_int(fl, fl));
        diff!(t, int_over_int_to_int(ii, ii));
        diff!(t, sum3_times(fl, fl, fl, fl));
        // moderate values too, where nothing saturates
        diff!(t, ease(fm, fm, fm));
        diff!(t, toward(fm, fm, fm));
        diff!(t, rate_to_int(fm, fm, fm, fm, fm, fm, fm));
        diff!(t, speed_rate_to_int(fm, fm, fm, fm, fm, fm));
        diff!(t, three_products(a33));
        diff!(t, turn_about(fm, fm, fm, fm, fm, fm));
        diff!(t, clip_lerp3(fm, fm, fm, a32));
        diff!(t, particle_square(fm, fm, fm, k4, is));
        diff!(t, turned_sprite(fm, fm, fm, k4, is, fm, fm));
        diff!(t, fraction_times_to_int(fm, fm));
        diff!(t, times_plus_to_int(fm, fm, fm));
        diff!(t, int_lerp_to_int(is, is, fm));
        diff!(t, prem_plus(ii, fm, fm));
        diff!(t, prem_plus(is, fd, fm));
        diff!(t, slot_project(fm, fm, fm, fm, fm, fm, fm));
        diff!(t, drifted(is, fm, m3, is, m3));
        diff!(t, approach(is, fm, fm, is));
        diff!(t, grown(is, d64, fm, fm));
    }

    fn run(w: u16, seed: u64) {
        let mut t = T::new(seed);
        with_control(w, || {
            all_functions(&mut t);
            pointer_functions(&mut t);
        });
        t.sets.check(&format!("cw_{w:04x}"), FROZEN);
    }

    #[test]
    fn frozen_at_double_precision() {
        run(0x027F, 0x5EED_0B01);
    }

    #[test]
    fn frozen_at_chop() {
        run(0x0E7F, 0x5EED_0B02);
    }

    #[test]
    fn frozen_rounding_down() {
        run(0x067F, 0x5EED_0B03);
    }

    #[test]
    fn frozen_rounding_up() {
        run(0x0A7F, 0x5EED_0B04);
    }

    /// The results' hashes, frozen on 2026-11-13 (`RC2K_FREEZE_OUT` to make them again).
    const FROZEN: &[(&str, u64)] = &[
        ("cw_027f::abs_blend_compare", 0x2bca21cfb6a3adcf),
        ("cw_027f::abs_sum_times_to_int", 0x4f304e81048bc274),
        ("cw_027f::abs_sum_to_int_kept", 0x2ce6b8fe9db89362),
        ("cw_027f::add_80", 0x60ef0d56094edfb5),
        ("cw_027f::angle_and_wave", 0xa9b8b093cb91b4a1),
        ("cw_027f::approach", 0x7d721de0c1eae1f8),
        ("cw_027f::blend3_80", 0x93677af18d96f7d9),
        ("cw_027f::blend_kept_abs_to_int", 0x71a83898a1966fac),
        ("cw_027f::clip_lerp3", 0x3d4e5d3f91117342),
        ("cw_027f::column_turn", 0x2a94eeed0f63ac5e),
        ("cw_027f::crossed_turned", 0xb7504f0566c46afd),
        ("cw_027f::damp", 0x4767eacaed4cbb03),
        ("cw_027f::diff_times_plus_times", 0xcff134a1a16593e3),
        ("cw_027f::drifted", 0xa447897ecd14a08b),
        ("cw_027f::ease", 0x743dfa08508c5db5),
        ("cw_027f::ext_less", 0xd725716bad0f5c32),
        ("cw_027f::float_times_int_to_int", 0xd9b7723914caf2b2),
        ("cw_027f::fraction_times_to_int", 0x421d0ced74b4f4f0),
        ("cw_027f::grown", 0xa105d1c356dd98d3),
        ("cw_027f::int_lerp_to_int", 0x3eda4c4e99f8069e),
        ("cw_027f::int_over_int_times", 0xea4d878dab0c4635),
        ("cw_027f::int_over_int_to_int", 0x05569e6ef6deaba6),
        ("cw_027f::int_plus", 0x24fc70d689b446e3),
        ("cw_027f::int_plus_less_squared", 0x823134a07b32ab14),
        ("cw_027f::int_plus_to_int", 0x4b5a076ed4d1991a),
        ("cw_027f::int_times3_wave_plus", 0x18ce90ba0f5234f1),
        ("cw_027f::int_times_int_plus_int", 0x4ae27f43d2cb413a),
        ("cw_027f::int_times_int_to_int", 0x9963062bf2038315),
        ("cw_027f::int_times_less_less", 0xe322c2904df42aac),
        ("cw_027f::int_times_plus_and_negated", 0x9e061399df443a71),
        ("cw_027f::int_times_plus_int_to_int", 0xef3fd652daf69ba4),
        ("cw_027f::int_times_plus_on", 0xdfda5af33be67aef),
        ("cw_027f::int_times_plus_times", 0x6965d2b8c1c549f8),
        ("cw_027f::int_times_plus_times_plus", 0x168d9cc6033afb61),
        ("cw_027f::int_times_times", 0x4cba96a394382321),
        ("cw_027f::int_times_times_to_int", 0x0bdd00f0f485a4dd),
        ("cw_027f::length_and_rate", 0x7d60b16f83ca05e4),
        ("cw_027f::less_int_plus_int", 0xef2f7290271784a0),
        ("cw_027f::less_own_times", 0xf0cef73aa651b9af),
        ("cw_027f::less_plus_int_times", 0x9179a82d3b022486),
        ("cw_027f::less_times_int", 0x70e7c0793db9cb8c),
        ("cw_027f::less_times_plus_negated_80", 0x71424e21fab87099),
        ("cw_027f::mul_80", 0xcb50bf506eb994f6),
        ("cw_027f::negated_less_plus_plus", 0x2b669616b442e245),
        ("cw_027f::normalised3", 0xe414959564e11950),
        ("cw_027f::one_less_inverse", 0x503cf812d7222f0d),
        ("cw_027f::over_int", 0x2d9fa50b5d18dc0d),
        ("cw_027f::pair_sum_80", 0xf9dd4515bd37c631),
        ("cw_027f::pair_sum_less", 0xdaeee107ff0d2b65),
        ("cw_027f::particle_square", 0x76f86463a2a75cc7),
        ("cw_027f::plus_int_times_80", 0xabd30244c5fb8373),
        ("cw_027f::plus_product_to_int", 0xa291991971e5ef49),
        ("cw_027f::plus_sum_times_int", 0x44e8bfcc511f898a),
        ("cw_027f::plus_times_int", 0xe00137b9fb3bf11f),
        ("cw_027f::prem_plus", 0xd7d1fa0958b259be),
        ("cw_027f::product_plus", 0xa08078dad5db106b),
        ("cw_027f::random_step", 0xe3706fbf848ff7a6),
        ("cw_027f::rate_to_int", 0x3be25c2c7233c221),
        ("cw_027f::scale_record", 0x7b77cf4d2ee990c9),
        ("cw_027f::sine_and_cosine", 0xe3b2036229830dae),
        ("cw_027f::slot_project", 0x3edbdef67bcb3dc8),
        ("cw_027f::speed_rate_to_int", 0xd23df7110815857b),
        ("cw_027f::sub_compare", 0x6caf06ab78accbc5),
        ("cw_027f::sub_thrice", 0x7805010387249cf8),
        ("cw_027f::sum3_times", 0x3de28207cb0ee83b),
        ("cw_027f::sum4_times", 0x0650ca2db4a74f8d),
        ("cw_027f::three_products", 0xe4035620ea89bd9f),
        ("cw_027f::three_products_80", 0x64190dfdfcea9118),
        ("cw_027f::times_compare", 0xf7a8dc9181752cf5),
        ("cw_027f::times_int_plus_int_to_int", 0x10e9516820d9b56e),
        ("cw_027f::times_int_plus_times_80", 0x7b396b35976d49b2),
        ("cw_027f::times_int_to_int", 0x135dcfc93eae1b09),
        ("cw_027f::times_kept_abs_to_int", 0x0161c7393c773367),
        ("cw_027f::times_less_abs_times", 0x87e5f458bd765d29),
        ("cw_027f::times_over_int", 0xa9b388d52b9a085e),
        ("cw_027f::times_plus_80", 0xfd1e3491e34ba776),
        ("cw_027f::times_plus_negated_80", 0x715cad0c4d001fb1),
        ("cw_027f::times_plus_to_int", 0xf3ea4adc83310f62),
        ("cw_027f::times_subr_to_int", 0x9ed3e023ddbc5a1e),
        ("cw_027f::toward", 0xcca5ad6ffdec47e7),
        ("cw_027f::turn_about", 0x20c8248571bd75fd),
        ("cw_027f::turn_yz", 0xe56f59daa97c2490),
        ("cw_027f::turned_place", 0xe4501885a53dec72),
        ("cw_027f::turned_place_less", 0xd3c0d97ae7556d3d),
        ("cw_027f::turned_sprite", 0x6080e371f07da5af),
        ("cw_027f::two_pi_less", 0x139ac6bd9804e05f),
        ("cw_027f::unit_and_length", 0xe60daaffde533162),
        ("cw_027f::zero_less_int_plus_int", 0x52234d154195180f),
        ("cw_027f::zero_times_plus_times", 0xc026e58b0a36691a),
        ("cw_067f::abs_blend_compare", 0xe813a6dd87ce949d),
        ("cw_067f::abs_sum_times_to_int", 0xb4acaa87494d6cfc),
        ("cw_067f::abs_sum_to_int_kept", 0x05fda6821cf1e29c),
        ("cw_067f::add_80", 0xfbdfdd57031c46f9),
        ("cw_067f::angle_and_wave", 0x253718ba1a529037),
        ("cw_067f::approach", 0xbef9e83c3773472c),
        ("cw_067f::blend3_80", 0x00b8f481c207e7c9),
        ("cw_067f::blend_kept_abs_to_int", 0x3755ade814c4b06d),
        ("cw_067f::clip_lerp3", 0x8736cf098aee33a3),
        ("cw_067f::column_turn", 0x0618dfd38188579f),
        ("cw_067f::crossed_turned", 0xbbd31338112cfc59),
        ("cw_067f::damp", 0x6d4146b163067346),
        ("cw_067f::diff_times_plus_times", 0xe55a94c400956112),
        ("cw_067f::drifted", 0x89aca4fe11fcbd75),
        ("cw_067f::ease", 0x06107ce759dff217),
        ("cw_067f::ext_less", 0x2fe570f95665b9be),
        ("cw_067f::float_times_int_to_int", 0xa6d21a959e460122),
        ("cw_067f::fraction_times_to_int", 0xd7fe9c5256e33dd2),
        ("cw_067f::grown", 0xd83c64e9cfef14af),
        ("cw_067f::int_lerp_to_int", 0x49c3987d0c1a5ff9),
        ("cw_067f::int_over_int_times", 0x40e1fb4d374101d7),
        ("cw_067f::int_over_int_to_int", 0x1e8b261ba2f9a8e9),
        ("cw_067f::int_plus", 0xf0d9fbe648421ba2),
        ("cw_067f::int_plus_less_squared", 0x38f71db848bf6051),
        ("cw_067f::int_plus_to_int", 0x20bcbb7bf672c7d8),
        ("cw_067f::int_times3_wave_plus", 0x28418bce8ea12f39),
        ("cw_067f::int_times_int_plus_int", 0xfe15f864e63012fa),
        ("cw_067f::int_times_int_to_int", 0xdaceac3f3a8f7f13),
        ("cw_067f::int_times_less_less", 0x0c972065dfd72a8d),
        ("cw_067f::int_times_plus_and_negated", 0x254f67833e1fce8d),
        ("cw_067f::int_times_plus_int_to_int", 0xc9ad6b52e30a58c0),
        ("cw_067f::int_times_plus_on", 0x9faac72d1940cf90),
        ("cw_067f::int_times_plus_times", 0xc897c6ddd665cf68),
        ("cw_067f::int_times_plus_times_plus", 0x29db15b8e7ba9e08),
        ("cw_067f::int_times_times", 0xcbc90a42f29e931f),
        ("cw_067f::int_times_times_to_int", 0xab983329a67500c9),
        ("cw_067f::length_and_rate", 0xedb0171869b8334b),
        ("cw_067f::less_int_plus_int", 0x4543cef29f3591d6),
        ("cw_067f::less_own_times", 0x64451128e61d225d),
        ("cw_067f::less_plus_int_times", 0xddf696758bbb42df),
        ("cw_067f::less_times_int", 0x4513f203d678e782),
        ("cw_067f::less_times_plus_negated_80", 0x551150704a5c05eb),
        ("cw_067f::mul_80", 0x9069569473e5d9e2),
        ("cw_067f::negated_less_plus_plus", 0xc48b2bf7004d3bab),
        ("cw_067f::normalised3", 0x93d124e60e3d6f34),
        ("cw_067f::one_less_inverse", 0xd430143965ec85b9),
        ("cw_067f::over_int", 0x63c2f2ce84f601ca),
        ("cw_067f::pair_sum_80", 0x6654eaf35d497fd0),
        ("cw_067f::pair_sum_less", 0xb9f80f70b58a54b9),
        ("cw_067f::particle_square", 0xc51283dc7a4fe1b5),
        ("cw_067f::plus_int_times_80", 0xc4dca471f5a1ea4d),
        ("cw_067f::plus_product_to_int", 0x4033f121a0e5f0e8),
        ("cw_067f::plus_sum_times_int", 0x413b22f029976f31),
        ("cw_067f::plus_times_int", 0xb1f5a3bb76c51a15),
        ("cw_067f::prem_plus", 0xf4c8a6c540341250),
        ("cw_067f::product_plus", 0x7b767305bb8e6a75),
        ("cw_067f::random_step", 0xb5a25bbb9ca4111d),
        ("cw_067f::rate_to_int", 0x39748b76f1f9173c),
        ("cw_067f::scale_record", 0xa8af5a46d8a3840b),
        ("cw_067f::sine_and_cosine", 0xcbb238e97f89709e),
        ("cw_067f::slot_project", 0x1e6ff95502b091f9),
        ("cw_067f::speed_rate_to_int", 0x72800a8e7368b38b),
        ("cw_067f::sub_compare", 0x3a621a68bc6a8a52),
        ("cw_067f::sub_thrice", 0x3c56a37c1b000e61),
        ("cw_067f::sum3_times", 0x9c13d06bd09a51b7),
        ("cw_067f::sum4_times", 0xdb779fbc7b0ddb49),
        ("cw_067f::three_products", 0x2b8c934818bf9198),
        ("cw_067f::three_products_80", 0x4d6b83d54cca7772),
        ("cw_067f::times_compare", 0x864dd57eba40920a),
        ("cw_067f::times_int_plus_int_to_int", 0xc3119bc4f80c2354),
        ("cw_067f::times_int_plus_times_80", 0x8f0e97e0dd599e9c),
        ("cw_067f::times_int_to_int", 0x60f6ddffd2bcb673),
        ("cw_067f::times_kept_abs_to_int", 0x1ce6ca69b63d406a),
        ("cw_067f::times_less_abs_times", 0x798f08da86022086),
        ("cw_067f::times_over_int", 0x3b846c6810791801),
        ("cw_067f::times_plus_80", 0x727e0e99af16065e),
        ("cw_067f::times_plus_negated_80", 0xf16c05ecc0116553),
        ("cw_067f::times_plus_to_int", 0xc244942e5f02a432),
        ("cw_067f::times_subr_to_int", 0x36bc78d9e474d6c8),
        ("cw_067f::toward", 0x78502ef92a16e4e2),
        ("cw_067f::turn_about", 0x364638c4cc325249),
        ("cw_067f::turn_yz", 0x5e37bf68d9afc99d),
        ("cw_067f::turned_place", 0x19064d1267415094),
        ("cw_067f::turned_place_less", 0xd1cd1334bb18352e),
        ("cw_067f::turned_sprite", 0xd3538611aa1cff55),
        ("cw_067f::two_pi_less", 0xfafc2388058c42d7),
        ("cw_067f::unit_and_length", 0x3d696bd363e02dbd),
        ("cw_067f::zero_less_int_plus_int", 0x65ddde5ebe4581df),
        ("cw_067f::zero_times_plus_times", 0xe5024f27ddc4f6e7),
        ("cw_0a7f::abs_blend_compare", 0x24c6c1f89676fbd3),
        ("cw_0a7f::abs_sum_times_to_int", 0x910c61935b51731b),
        ("cw_0a7f::abs_sum_to_int_kept", 0x22d43f5c41a73536),
        ("cw_0a7f::add_80", 0x67ed3d7a2bd40618),
        ("cw_0a7f::angle_and_wave", 0x88c545e4a037faa7),
        ("cw_0a7f::approach", 0xe66fb4c264f870ca),
        ("cw_0a7f::blend3_80", 0xb536ff50d4953f6f),
        ("cw_0a7f::blend_kept_abs_to_int", 0xf8df67f606fed65a),
        ("cw_0a7f::clip_lerp3", 0xd4f8707121c4a7ff),
        ("cw_0a7f::column_turn", 0xe5daa777d48d1c7c),
        ("cw_0a7f::crossed_turned", 0xfaa24ba23228a123),
        ("cw_0a7f::damp", 0xc1cf127456f3e94c),
        ("cw_0a7f::diff_times_plus_times", 0x24ab6a946add4baf),
        ("cw_0a7f::drifted", 0x961209e34c8a20dd),
        ("cw_0a7f::ease", 0x3bfbe13963b031d2),
        ("cw_0a7f::ext_less", 0x5039d097cb4f42da),
        ("cw_0a7f::float_times_int_to_int", 0x4c81f1914d4208ee),
        ("cw_0a7f::fraction_times_to_int", 0x608b205201d297bb),
        ("cw_0a7f::grown", 0x252bcda2fb86a2ae),
        ("cw_0a7f::int_lerp_to_int", 0x67b8515f7f70f6e4),
        ("cw_0a7f::int_over_int_times", 0xcc36d6a6fac60d47),
        ("cw_0a7f::int_over_int_to_int", 0xc340ce33d71d8436),
        ("cw_0a7f::int_plus", 0x6d6dba2765620504),
        ("cw_0a7f::int_plus_less_squared", 0x9f4162e82151058c),
        ("cw_0a7f::int_plus_to_int", 0x50772881adacab89),
        ("cw_0a7f::int_times3_wave_plus", 0x07fab91ad908e0f0),
        ("cw_0a7f::int_times_int_plus_int", 0xf4a10293fa9d7eee),
        ("cw_0a7f::int_times_int_to_int", 0x4b0feffaa615cb96),
        ("cw_0a7f::int_times_less_less", 0xe40ad18c3242477f),
        ("cw_0a7f::int_times_plus_and_negated", 0x2165837b747360d4),
        ("cw_0a7f::int_times_plus_int_to_int", 0xc3624481a9c67926),
        ("cw_0a7f::int_times_plus_on", 0x2da356c7c2627ea0),
        ("cw_0a7f::int_times_plus_times", 0xc281c31ab24763de),
        ("cw_0a7f::int_times_plus_times_plus", 0xec5f86a5e8fdd31b),
        ("cw_0a7f::int_times_times", 0x1ceb3dc957052f44),
        ("cw_0a7f::int_times_times_to_int", 0xcaf7280cfcbc7c91),
        ("cw_0a7f::length_and_rate", 0x40a78ff7781ec720),
        ("cw_0a7f::less_int_plus_int", 0xf41a4f15688f31fd),
        ("cw_0a7f::less_own_times", 0x44078e2b9ec7026d),
        ("cw_0a7f::less_plus_int_times", 0x1b51eccc9ecd489d),
        ("cw_0a7f::less_times_int", 0x07c46d076cdcb29b),
        ("cw_0a7f::less_times_plus_negated_80", 0xc8253e9327365213),
        ("cw_0a7f::mul_80", 0xcec7aa45fa1fcb06),
        ("cw_0a7f::negated_less_plus_plus", 0x36fa7e61d2dcd5e4),
        ("cw_0a7f::normalised3", 0xd89eb90b70a52b8a),
        ("cw_0a7f::one_less_inverse", 0x48db3ce7649c95e5),
        ("cw_0a7f::over_int", 0x8ed5d7966771558c),
        ("cw_0a7f::pair_sum_80", 0x86a229a68fc550f4),
        ("cw_0a7f::pair_sum_less", 0x9f71614a55d6951f),
        ("cw_0a7f::particle_square", 0x6dea4e00ddd1dca3),
        ("cw_0a7f::plus_int_times_80", 0x0f3da8a4ebd34bc1),
        ("cw_0a7f::plus_product_to_int", 0xaa4e2641eb8f7fc8),
        ("cw_0a7f::plus_sum_times_int", 0x3f69b8c610c80099),
        ("cw_0a7f::plus_times_int", 0xb4cd5b985a1bdf62),
        ("cw_0a7f::prem_plus", 0xb529a11865a14cd0),
        ("cw_0a7f::product_plus", 0x4f0a763a708913f2),
        ("cw_0a7f::random_step", 0x2a15c453231192bf),
        ("cw_0a7f::rate_to_int", 0xd3d9c8b5226ad9dc),
        ("cw_0a7f::scale_record", 0x71c7cd42755d608d),
        ("cw_0a7f::sine_and_cosine", 0x5c59ce9222271931),
        ("cw_0a7f::slot_project", 0xca9e3d5b6a014984),
        ("cw_0a7f::speed_rate_to_int", 0x5ba629725d3845a4),
        ("cw_0a7f::sub_compare", 0x3ec53bcd782370ea),
        ("cw_0a7f::sub_thrice", 0x1ec2e1d85d71c113),
        ("cw_0a7f::sum3_times", 0x59b9d6127bdbd9e9),
        ("cw_0a7f::sum4_times", 0xa7984070e7d27f18),
        ("cw_0a7f::three_products", 0xde61834fc2cdfda4),
        ("cw_0a7f::three_products_80", 0x994997fc626b051a),
        ("cw_0a7f::times_compare", 0xffe0eab3a4675412),
        ("cw_0a7f::times_int_plus_int_to_int", 0xbf751729d8bd85b9),
        ("cw_0a7f::times_int_plus_times_80", 0x84f6d070a4bc05c7),
        ("cw_0a7f::times_int_to_int", 0x395a51c97404a013),
        ("cw_0a7f::times_kept_abs_to_int", 0xef389f8fbad667c5),
        ("cw_0a7f::times_less_abs_times", 0xf3f17897e81c2020),
        ("cw_0a7f::times_over_int", 0x2d28db5d399803dc),
        ("cw_0a7f::times_plus_80", 0xaf4b81b725d7c4fe),
        ("cw_0a7f::times_plus_negated_80", 0x3c902872ea908440),
        ("cw_0a7f::times_plus_to_int", 0x8855d6817724838e),
        ("cw_0a7f::times_subr_to_int", 0x326b7d635f8ae704),
        ("cw_0a7f::toward", 0xe6c08e1fd2fcc06f),
        ("cw_0a7f::turn_about", 0x3821a78caf7b3104),
        ("cw_0a7f::turn_yz", 0x205fa8c17849571e),
        ("cw_0a7f::turned_place", 0x939ad2d658054303),
        ("cw_0a7f::turned_place_less", 0x0aeca9dd1847c935),
        ("cw_0a7f::turned_sprite", 0x5541a3eb5f8f3e7a),
        ("cw_0a7f::two_pi_less", 0xfbc0e60b85456b66),
        ("cw_0a7f::unit_and_length", 0x08d1b211880cbc8f),
        ("cw_0a7f::zero_less_int_plus_int", 0xdccd43f4f06d6610),
        ("cw_0a7f::zero_times_plus_times", 0x1cd6ccac97f7f125),
        ("cw_0e7f::abs_blend_compare", 0x5ebc13803facf06a),
        ("cw_0e7f::abs_sum_times_to_int", 0x9545fd38d5094b54),
        ("cw_0e7f::abs_sum_to_int_kept", 0xd0d934715637ba67),
        ("cw_0e7f::add_80", 0x5b2ccf0965a33814),
        ("cw_0e7f::angle_and_wave", 0x493aed6a3d4ab5e3),
        ("cw_0e7f::approach", 0x34d9af8900879d52),
        ("cw_0e7f::blend3_80", 0xd31d97f90abe28fa),
        ("cw_0e7f::blend_kept_abs_to_int", 0x3f4310857e6fdc8f),
        ("cw_0e7f::clip_lerp3", 0xb595614b1218bac7),
        ("cw_0e7f::column_turn", 0x4dec1a5d70aabe6f),
        ("cw_0e7f::crossed_turned", 0xf5b720dbde0b36b7),
        ("cw_0e7f::damp", 0x5183423bfd82e639),
        ("cw_0e7f::diff_times_plus_times", 0x6f3fe6b48a8b16d1),
        ("cw_0e7f::drifted", 0xd53c95f1b0384b51),
        ("cw_0e7f::ease", 0x0cf62d04f7689311),
        ("cw_0e7f::ext_less", 0x867830854ea5cab0),
        ("cw_0e7f::float_times_int_to_int", 0x8e01b2524ea531df),
        ("cw_0e7f::fraction_times_to_int", 0x9ed08f8d22d78148),
        ("cw_0e7f::grown", 0x7c3059ae81c2906e),
        ("cw_0e7f::int_lerp_to_int", 0x4ad22555f8877c37),
        ("cw_0e7f::int_over_int_times", 0x08119fed58eca720),
        ("cw_0e7f::int_over_int_to_int", 0x855987de42276d31),
        ("cw_0e7f::int_plus", 0x28c9d331ec56bc71),
        ("cw_0e7f::int_plus_less_squared", 0xaa1e11af091e434c),
        ("cw_0e7f::int_plus_to_int", 0x74f577ffca1b29df),
        ("cw_0e7f::int_times3_wave_plus", 0xcecf6fbbc132cbf4),
        ("cw_0e7f::int_times_int_plus_int", 0xd38e70f533fc4567),
        ("cw_0e7f::int_times_int_to_int", 0xf525d4b09594eb72),
        ("cw_0e7f::int_times_less_less", 0x403c3d467e272114),
        ("cw_0e7f::int_times_plus_and_negated", 0x8237f8700edc91f5),
        ("cw_0e7f::int_times_plus_int_to_int", 0x180e5e504d8c9111),
        ("cw_0e7f::int_times_plus_on", 0x95daf4419aefe1db),
        ("cw_0e7f::int_times_plus_times", 0xae9b344154a27721),
        ("cw_0e7f::int_times_plus_times_plus", 0xefdb40a4dc741637),
        ("cw_0e7f::int_times_times", 0x6a7d79290cc1fd75),
        ("cw_0e7f::int_times_times_to_int", 0x24a6c13217a51977),
        ("cw_0e7f::length_and_rate", 0xc821a541e3f619ea),
        ("cw_0e7f::less_int_plus_int", 0x4fa62eba35f4b601),
        ("cw_0e7f::less_own_times", 0x79cd685ef979dbd9),
        ("cw_0e7f::less_plus_int_times", 0x642a48853cf0b924),
        ("cw_0e7f::less_times_int", 0x332bd1e54620d00a),
        ("cw_0e7f::less_times_plus_negated_80", 0x3dc4149fea2c1613),
        ("cw_0e7f::mul_80", 0xac05c318d36fa33b),
        ("cw_0e7f::negated_less_plus_plus", 0x58ada5b72ed3ada0),
        ("cw_0e7f::normalised3", 0x4a21e15fb50f9bf9),
        ("cw_0e7f::one_less_inverse", 0x41c89d35edf87857),
        ("cw_0e7f::over_int", 0xd499664d0244964c),
        ("cw_0e7f::pair_sum_80", 0x3c63f86650a95c99),
        ("cw_0e7f::pair_sum_less", 0xa3728cba704f3142),
        ("cw_0e7f::particle_square", 0xeb3a689f98c3cdf8),
        ("cw_0e7f::plus_int_times_80", 0x5326b8ac8ac51a27),
        ("cw_0e7f::plus_product_to_int", 0x96bcec23ec51aa0d),
        ("cw_0e7f::plus_sum_times_int", 0xfe6a5111132006b7),
        ("cw_0e7f::plus_times_int", 0xe7d71464eb501c07),
        ("cw_0e7f::prem_plus", 0xf6bff864f900ce87),
        ("cw_0e7f::product_plus", 0x45f0b01f7087823f),
        ("cw_0e7f::random_step", 0x2e3938afd1197467),
        ("cw_0e7f::rate_to_int", 0xd12fbc487f3369a1),
        ("cw_0e7f::scale_record", 0x567023c97549191c),
        ("cw_0e7f::sine_and_cosine", 0xfaed5c8720cc8a4e),
        ("cw_0e7f::slot_project", 0x6d62c51dbb770cf7),
        ("cw_0e7f::speed_rate_to_int", 0x0d16878da105f57e),
        ("cw_0e7f::sub_compare", 0xe263a0640ee734cd),
        ("cw_0e7f::sub_thrice", 0xd668362f4584b1d7),
        ("cw_0e7f::sum3_times", 0xf0b11b2a6a75469e),
        ("cw_0e7f::sum4_times", 0xc21baca6d8826797),
        ("cw_0e7f::three_products", 0x76f155425e875b23),
        ("cw_0e7f::three_products_80", 0xff6e4cd3a8c13f9e),
        ("cw_0e7f::times_compare", 0x6191b215e54f07cd),
        ("cw_0e7f::times_int_plus_int_to_int", 0xa85896e23c16d016),
        ("cw_0e7f::times_int_plus_times_80", 0xd33b9fe8295bc014),
        ("cw_0e7f::times_int_to_int", 0xb9143fc6c5d5f4f4),
        ("cw_0e7f::times_kept_abs_to_int", 0x79b842dd63e82521),
        ("cw_0e7f::times_less_abs_times", 0xa2e432678342893c),
        ("cw_0e7f::times_over_int", 0x306a21c3a3bb9a80),
        ("cw_0e7f::times_plus_80", 0x0568162b444654f3),
        ("cw_0e7f::times_plus_negated_80", 0x5b3eac3949e82cfc),
        ("cw_0e7f::times_plus_to_int", 0x02b71f6d1a0db4f5),
        ("cw_0e7f::times_subr_to_int", 0xab3983e5913f7c05),
        ("cw_0e7f::toward", 0xfe0e3b25f1245c0e),
        ("cw_0e7f::turn_about", 0x1cd59de8e9cf83cb),
        ("cw_0e7f::turn_yz", 0x4dc8bc0d90d14641),
        ("cw_0e7f::turned_place", 0x1db95ef0ae0f2746),
        ("cw_0e7f::turned_place_less", 0x6b8a8939784dfb17),
        ("cw_0e7f::turned_sprite", 0x651b92bc3ecdd042),
        ("cw_0e7f::two_pi_less", 0x2e62efe978c0835c),
        ("cw_0e7f::unit_and_length", 0x2521b72809ba3fc9),
        ("cw_0e7f::zero_less_int_plus_int", 0xd55ec724b78f8dd6),
        ("cw_0e7f::zero_times_plus_times", 0xd4b9453e8bc730c3),
    ];
}
