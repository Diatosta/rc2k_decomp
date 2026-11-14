//! The frozen comparisons of the helpers in `math/x87/run.rs`, `math/x87/load.rs` and
//! `math/normalise_vector.rs`: the plain versions' results on seeded inputs at the four
//! control words are kept as one hash per function (`test_support::frozen`, table `FROZEN`
//! at the end). The hashes were taken from versions that matched the x87's instructions
//! (bit for bit, any NaN equal to any NaN; the transcendental ones within an ulp of a
//! float, two of a double).

mod tests {
    use crate::math::x87::set_control_word;
    use crate::math::x87_float::{X, f80, from_f80};
    use crate::math::x87_frozen_a::AsBits;
    use crate::math::{normalise_vector, x87};
    use crate::test_support::frozen::{Canon, Digest, check_one};

    /// The control words tried: double precision with nearest (the game's), chop, down and
    /// up.
    const ROUNDING: [u16; 4] = [0x027F, 0x0E7F, 0x067F, 0x0A7F];

    /// A double as the 80-bit format.
    fn f64_to_f80(x: f64) -> [u8; 10] {
        f80(X(x))
    }
    /// An 80-bit value as a double (to nearest even).
    fn f80_to_f64(b: [u8; 10]) -> f64 {
        from_f80(b).0
    }
    const CASES: usize = 20_000;

    struct Rng(u64);

    impl Rng {
        fn new(name: &str) -> Rng {
            // FNV-1a of the name: every function its own seed.
            let mut h = 0xCBF2_9CE4_8422_2325u64;
            for b in name.bytes() {
                h = (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01B3);
            }
            Rng(h | 1)
        }
        fn u64(&mut self) -> u64 {
            self.0 ^= self.0 >> 12;
            self.0 ^= self.0 << 25;
            self.0 ^= self.0 >> 27;
            self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }
        fn u32(&mut self) -> u32 {
            (self.u64() >> 32) as u32
        }
        fn below(&mut self, n: usize) -> usize {
            (self.u64() % n as u64) as usize
        }
        /// A finite float's bits: random bit patterns and the values that round oddly.
        fn f(&mut self) -> u32 {
            const EDGE: [u32; 24] = [
                0x0000_0000,
                0x8000_0000,
                0x3F80_0000,
                0xBF80_0000,
                0x3F00_0000,
                0x3FC0_0000,
                0x4020_0000,
                0xBF00_0000,
                0xBFC0_0000,
                0x0000_0001,
                0x007F_FFFF,
                0x0080_0000,
                0x7F7F_FFFF,
                0xFF7F_FFFF,
                0x4EFF_FFFF,
                0x4F00_0000,
                0xCF00_0000,
                0x4F80_0000,
                0x501C_F6C0,
                0x3DCC_CCCD,
                0x4049_0FDB,
                0x4AFF_FFFF,
                0xCAFF_FFFF,
                0x3FC9_0FDB,
            ];
            match self.below(12) {
                0 => loop {
                    let b = self.u32();
                    if f32::from_bits(b).is_finite() {
                        return b;
                    }
                },
                1 | 2 => EDGE[self.below(EDGE.len())],
                3 => ((self.below(2001) as i32 - 1000) as f32).to_bits(),
                4 => ((self.below(4001) as i32 - 2000) as f32 * 0.5).to_bits(),
                5 => ((self.below(8001) as i32 - 4000) as f32 * 0.25).to_bits(),
                6 => {
                    let (s, e, m) = (
                        self.u32() & 0x8000_0000,
                        100 + self.below(51) as u32,
                        self.u32(),
                    );
                    s | (e << 23) | (m & 0x7F_FFFF)
                }
                7 => {
                    // Near the edge of the whole numbers.
                    let base = if self.below(2) == 0 {
                        2147483648.0f32
                    } else {
                        -2147483648.0
                    };
                    (base + (self.below(1025) as f32 - 512.0) * 128.0).to_bits()
                }
                8 => (self.u32() as f32 / u32::MAX as f32 * 2.0 - 1.0).to_bits(),
                _ => {
                    let (s, e, m) = (
                        self.u32() & 0x8000_0000,
                        117 + self.below(21) as u32,
                        self.u32(),
                    );
                    s | (e << 23) | (m & 0x7F_FFFF)
                }
            }
        }
        /// As `f`, now and then no number or an infinity.
        fn fw(&mut self) -> u32 {
            match self.below(20) {
                0 => 0x7FC0_0000,
                1 => 0xFFC0_0000,
                2 => 0x7F80_0000,
                3 => 0xFF80_0000,
                _ => self.f(),
            }
        }
        /// A positive float of a plain size.
        fn pos(&mut self) -> u32 {
            self.f() & 0x7FFF_FFFF
        }
        /// A whole number: any, small, or at the edges.
        fn i(&mut self) -> i32 {
            const EDGE: [i32; 12] = [
                0,
                1,
                -1,
                i32::MAX,
                i32::MIN,
                65536,
                -65536,
                46341,
                16_777_216,
                16_777_217,
                -16_777_217,
                1 << 30,
            ];
            match self.below(8) {
                0 => self.u32() as i32,
                1 | 2 => self.below(201) as i32 - 100,
                3 => EDGE[self.below(EDGE.len())],
                4 => (self.u32() >> 8) as i32 - (1 << 23),
                5 => (self.u32() >> 12) as i32 - (1 << 19),
                _ => self.below(4001) as i32 - 2000,
            }
        }
        /// A float of a size an angle may have.
        fn angle(&mut self) -> u32 {
            match self.below(6) {
                0 => (self.u32() as f32 / u32::MAX as f32 * 2000.0 - 1000.0).to_bits(),
                1 => ((self.below(801) as f32 - 400.0) * core::f32::consts::FRAC_PI_2).to_bits(),
                2 => (self.u32() as f32 / u32::MAX as f32 * 20.0 - 10.0).to_bits(),
                3 => (self.u32() as f32 / u32::MAX as f32 * 6.0e5 - 3.0e5).to_bits(),
                _ => (self.u32() as f32 / u32::MAX as f32 * 8.0 - 4.0).to_bits(),
            }
        }
        /// An 80-bit value that came from a double.
        fn e80(&mut self) -> [u8; 10] {
            let x = match self.below(4) {
                0 => f64::from(f32::from_bits(self.f())),
                1 => f64::from(self.i()),
                2 => f64::from(f32::from_bits(self.f())) * (1.0 + self.u32() as f64 / 4.0e12),
                _ => (self.u64() as f64 / u64::MAX as f64 - 0.5) * 4.0,
            };
            f64_to_f80(x)
        }
    }

    /// Runs `plain` on `CASES` inputs per control word and hashes its results
    /// (`test_support::frozen`): the hash must be the one frozen for `name`.
    fn check<T, P: AsBits>(
        name: &str,
        cws: &[u16],
        gen_input: impl Fn(&mut Rng) -> T,
        plain: impl Fn(&T) -> P,
    ) where
        P::Out: Canon,
    {
        let mut rng = Rng::new(name);
        let mut digest = Digest::default();
        for &cw in cws {
            set_control_word(cw);
            for _ in 0..CASES {
                let input = gen_input(&mut rng);
                plain(&input).into_bits().feed(&mut digest);
            }
        }
        set_control_word(0x027F);
        check_one("c", name, digest, FROZEN);
    }

    /// `check` at all four control words.
    fn check_all<T, P: AsBits>(
        name: &str,
        gen_input: impl Fn(&mut Rng) -> T,
        plain: impl Fn(&T) -> P,
    ) where
        P::Out: Canon,
    {
        check(name, &ROUNDING, gen_input, plain);
    }

    // ---- the boundary: 80 bits and doubles ----

    /// `f64_to_f80`, frozen from its comparison with the x87's conversion (a NaN keeps its
    /// payload, its quiet bit set).
    #[test]
    fn f64_to_f80_as_fld_qword_fstp_tbyte() {
        let mut rng = Rng::new("f64_to_f80");
        set_control_word(0x027F);
        let mut digest = Digest::default();
        for k in 0..100_000 {
            let bits = match k % 5 {
                0 => rng.u64(),
                1 => rng.u64() & 0x800F_FFFF_FFFF_FFFF, // denormals and zeros
                2 => f64::from(f32::from_bits(rng.f())).to_bits(),
                3 => (rng.i() as f64).to_bits(),
                _ => 0x7FF0_0000_0000_0000 | (rng.u64() & 0x800F_FFFF_FFFF_FFFF),
            };
            f64_to_f80(f64::from_bits(bits)).feed(&mut digest);
        }
        check_one("c", "f64_to_f80", digest, FROZEN);
    }

    /// `f80_to_f64`, frozen from its comparison with the x87's conversion.
    #[test]
    fn f80_to_f64_as_fld_tbyte_fstp_qword() {
        let mut rng = Rng::new("f80_to_f64");
        set_control_word(0x027F);
        let mut digest = Digest::default();
        for k in 0..100_000 {
            let mant = (rng.u64() | (1 << 63)) & if k % 3 == 0 { !0x7FF } else { !0 };
            let e = 16383 - 1000 + rng.below(2000) as u16;
            let mut b = [0u8; 10];
            b[..8].copy_from_slice(&mant.to_le_bytes());
            let sign = if rng.below(2) == 0 { 0 } else { 0x8000 };
            b[8..].copy_from_slice(&(e | sign).to_le_bytes());
            f80_to_f64(b).feed(&mut digest);
        }
        check_one("c", "f80_to_f64", digest, FROZEN);
        // Zero, infinity, a number too large for a double.
        assert_eq!(f80_to_f64([0; 10]).to_bits(), 0);
        assert_eq!(f80_to_f64(f64_to_f80(f64::NEG_INFINITY)), f64::NEG_INFINITY);
        assert!(f80_to_f64(f64_to_f80(f64::NAN)).is_nan());
        assert_eq!(f80_to_f64(f64_to_f80(-0.0)).to_bits(), (-0.0f64).to_bits());
    }

    // ---- math/x87/load.rs ----

    #[test]
    fn load_whole_number_results() {
        check_all(
            "int_times_int_over_to_int",
            |g| (g.i(), g.i(), g.f()),
            |&(a, b, c)| x87::int_times_int_over_to_int(a, b, c),
        );
        check("abs_to_int", &ROUNDING, Rng::f, |&a| x87::abs_to_int(a));
        check_all(
            "add_to_int",
            |g| (g.f(), g.f()),
            |&(a, b)| x87::add_to_int(a, b),
        );
        check_all(
            "int_square_sum_scaled_to_int",
            |g| (g.i(), g.i(), g.f(), 1 + g.below(2) as u32),
            |&(a, b, k, t)| x87::int_square_sum_scaled_to_int(a, b, k, t),
        );
        check(
            "product4_to_int",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, b, c, d)| x87::mul3_to_int(a, b, c, d) as u32,
        );
        check("tbyte_to_int", &ROUNDING, Rng::e80, |&a| {
            x87::tbyte_to_int(a)
        });
        check_all(
            "sun_size",
            |g| (g.f(), g.f(), g.f(), g.f(), g.f(), g.f()),
            |&(x, cx, y, cy, h, k)| x87::sun_size(x, cx, y, cy, h, k),
        );
    }

    #[test]
    fn load_float_results() {
        check(
            "float_hypot",
            &ROUNDING,
            |g| (g.f(), g.f()),
            |&(a, b)| x87::float_hypot(a, b),
        );
        check(
            "int_square_sum",
            &ROUNDING,
            |g| (g.i(), g.i()),
            |&(a, b)| x87::int_square_sum(a, b),
        );
        check(
            "float_square_sum",
            &ROUNDING,
            |g| (g.f(), g.f()),
            |&(a, b)| x87::float_square_sum(a, b),
        );
        check(
            "product3",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f()),
            |&(a, b, c)| x87::mul2(a, b, c),
        );
        check(
            "int_over_float",
            &ROUNDING,
            |g| (g.i(), g.f()),
            |&(a, b)| x87::int_over(a, b),
        );
        check(
            "int_plus_times",
            &ROUNDING,
            |g| (g.i(), g.f(), g.f()),
            |&(a, b, c)| x87::int_plus_times(a, b, c),
        );
        check(
            "eased_roll",
            &ROUNDING,
            |g| (g.i(), g.f(), g.f(), g.f(), g.f(), g.f(), g.f()),
            |&(i, a, b, v, c, m, h)| x87::eased_roll(i, a, b, v, c, m, h),
        );
        check("to_tbyte", &ROUNDING, Rng::f, |&a| x87::to_tbyte(a));
    }

    /// `fsincos`: the sine and cosine of the angles within an ulp of a float.
    #[test]
    fn load_sun_direction() {
        check(
            "sun_direction",
            &ROUNDING,
            |g| {
                (
                    g.i() % 4096,
                    g.angle(),
                    (g.below(1000) as f32 * 1e-4).to_bits(),
                    g.i() % 4096,
                    g.f(),
                )
            },
            |&(a, k, s, b, m)| x87::sun_direction(a, k, s, b, m),
        );
    }

    #[test]
    fn load_comparisons() {
        check(
            "below_or_unordered",
            &ROUNDING,
            |g| (g.fw(), g.fw()),
            |&(a, b)| x87::below_or_unordered(a, b),
        );
        check(
            "scaled_abs_less_below",
            &ROUNDING,
            |g| (g.fw(), g.f(), g.f(), g.fw()),
            |&(a, h, k, b)| x87::scaled_abs_less_below(a, h, k, b),
        );
        check(
            "abs_below_or_unordered",
            &ROUNDING,
            |g| (g.fw(), g.fw()),
            |&(a, k)| x87::abs_below_or_unordered(a, k),
        );
    }

    fn matrix_input(g: &mut Rng) -> ([i32; 3], [u32; 16]) {
        let mut m = [0u32; 16];
        for x in &mut m {
            *x = g.f();
        }
        ([g.i(), g.i(), g.i()], m)
    }

    #[test]
    fn load_rotations() {
        check_all("rotate_rounding_z", matrix_input, |(p, m)| {
            x87::rotate_rounding_z(*p, m)
        });
        check_all("rotate_rounding", matrix_input, |(p, m)| {
            x87::rotate_rounding(*p, m)
        });
    }

    #[test]
    fn load_sun_on_screen() {
        check_all(
            "sun_on_screen",
            |g| {
                (
                    [
                        g.i() as u32,
                        g.i() as u32,
                        g.i() as u32,
                        g.i() as u32,
                        g.f(),
                        g.f(),
                        g.f(),
                    ],
                    g.below(2) == 0,
                )
            },
            |&(v, flat)| {
                let scratch = crate::session::Dword::new(0xAAAA_AAAA);
                let point = [v[0] as i32, v[1] as i32, v[2] as i32, v[3] as i32];
                let (place, hit) = x87::sun_on_screen(point, v[4], [v[5], v[6]], flat, &scratch);
                let o = [
                    place[0].to_bits(),
                    hit[0] as u32,
                    place[1].to_bits(),
                    hit[1] as u32,
                ];
                (o, scratch.get())
            },
        );
    }

    // ---- math/x87/run.rs ----

    #[test]
    fn run_whole_number_results() {
        check_all(
            "plane_height",
            |g| (g.i(), g.i(), g.i(), g.f(), g.f()),
            |&(a, b, c, f, h)| x87::plane_height(a, b, c, f, h),
        );
        check(
            "int_times_times_over_to_int",
            &ROUNDING,
            |g| (g.i(), g.f(), g.f(), g.f()),
            |&(i, a, b, c)| x87::int_times_times_over_to_int(i, a, b, c),
        );
        check_all(
            "sub_div_to_int",
            |g| (g.f(), g.f(), g.f()),
            |&(a, b, c)| x87::sub_div_to_int(a, b, c),
        );
        check("ext_to_int", &ROUNDING, Rng::e80, |&e| x87::ext_to_int(e));
        check(
            "mul2_to_int",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f()),
            |&(a, b, c)| x87::mul2_to_int(a, b, c),
        );
        check_all(
            "float_hypot_to_int",
            |g| (g.f(), g.f()),
            |&(a, b)| x87::float_hypot_to_int(a, b),
        );
        check_all(
            "int_toward_to_int",
            |g| (g.i(), g.f(), g.f()),
            |&(i, a, k)| x87::int_toward_to_int(i, a, k),
        );
        check(
            "times_fist_compare",
            &ROUNDING,
            |g| (g.fw(), g.f(), g.fw()),
            |&(a, k, z)| x87::times_fist_compare(a, k, z),
        );
        check_all(
            "div_mul3_to_int",
            |g| (g.f(), g.f(), g.f(), g.f(), g.f()),
            |&(a, b, c, d, e)| x87::div_mul3_to_int(a, b, c, d, e),
        );
        check(
            "mul3_to_int",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, b, c, d)| x87::mul3_to_int(a, b, c, d),
        );
        check_all(
            "wheel_int",
            |g| [g.f(), g.f(), g.f(), g.f(), g.f(), g.f(), g.f()],
            |&v| x87::wheel_int(v),
        );
        check(
            "trunc_mul_to_int",
            &ROUNDING,
            |g| (g.f(), g.f()),
            |&(a, b)| x87::trunc_mul_to_int(a, b),
        );
    }

    #[test]
    fn run_float_results() {
        check(
            "sum_times_over",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, b, h, d)| x87::sum_times_over(a, b, h, d),
        );
        check(
            "square_times_plus_times",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, k, c, h)| x87::square_times_plus_times(a, k, c, h),
        );
        check("minus_one", &ROUNDING, |_| (), |_| (-1.0f32).to_bits());
        check(
            "mul3",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, b, c, d)| x87::mul3(a, b, c, d),
        );
        check("square", &ROUNDING, Rng::f, |&a| x87::mul(a, a));
        check(
            "int_lerp",
            &ROUNDING,
            |g| (g.i(), g.f(), g.f(), g.f()),
            |&(i, d, hi, lo)| x87::int_lerp(i, d, hi, lo),
        );
        check(
            "int_over_int_plus",
            &ROUNDING,
            |g| (g.i(), g.i(), g.f()),
            |&(a, b, c)| x87::int_over_int_plus(a, b, c),
        );
        check("one_less", &ROUNDING, Rng::f, |&a| x87::one_less(a));
        check(
            "abs_sqrt_times_plus",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f()),
            |&(x, t, c)| x87::abs_sqrt_times_plus(x, t, c),
        );
        check(
            "add3",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f()),
            |&(a, b, c)| x87::add3(a, b, c),
        );
        check(
            "neg_product_plus",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f()),
            |&(a, b, c)| x87::neg_product_plus(a, b, c),
        );
        check(
            "neg_mul",
            &ROUNDING,
            |g| (g.f(), g.f()),
            |&(a, b)| x87::neg_mul(a, b),
        );
        check("abs_of", &ROUNDING, Rng::f, |&a| x87::abs_of(a));
        check(
            "square_times_over",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f()),
            |&(x, k, d)| x87::square_times_over(x, k, d),
        );
        check(
            "toward_one",
            &ROUNDING,
            |g| (g.f(), g.f()),
            |&(a, b)| x87::toward_one(a, b),
        );
        check(
            "lerp_int_over2",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.i(), g.f(), g.f()),
            |&(a, b, k, i, d, w)| x87::lerp_int_over2(a, b, k, i, d, w),
        );
        check(
            "weight_force",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f(), g.f()),
            |&(v, one, a, h, m)| x87::weight_force(v, one, a, h, m),
        );
        check(
            "neg_product2_plus",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, b, c, d)| x87::neg_product2_plus(a, b, c, d),
        );
        check(
            "neg_mul2",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f()),
            |&(a, b, c)| x87::neg_mul2(a, b, c),
        );
        check(
            "sub_mul",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f()),
            |&(a, b, c)| x87::sub_mul(a, b, c),
        );
        check(
            "abs_div_add_mul",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(x, a, b, c)| x87::abs_div_add_mul(x, a, b, c),
        );
        check(
            "mul2_plus",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, b, c, d)| x87::mul2_plus(a, b, c, d),
        );
        check(
            "over_abs_plus_minus",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, s, b, c)| x87::over_abs_plus_minus(a, s, b, c),
        );
        check(
            "abs_fidiv",
            &ROUNDING,
            |g| (g.f(), g.i()),
            |&(x, n)| x87::abs_fidiv(x, n),
        );
        check(
            "int_fidiv",
            &ROUNDING,
            |g| (g.i(), g.i()),
            |&(a, b)| x87::int_ratio(a, b),
        );
        check(
            "one_less_times_plus",
            &ROUNDING,
            |g| (g.f(), g.f()),
            |&(d, s)| x87::toward_one(d, s),
        );
        check(
            "sub_mul_add",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, b, c, d)| x87::sub_mul_add(a, b, c, d),
        );
        check(
            "lerp_table",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f(), g.f()),
            |&(s, a0, a1, c0, c1)| x87::lerp_table(s, a0, a1, c0, c1),
        );
        check(
            "div_then_mul",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f()),
            |&(a, b, c)| x87::div_then_mul(a, b, c),
        );
        check(
            "sum_times_int_over_minus",
            &ROUNDING,
            |g| (g.f(), g.f(), g.i(), g.f()),
            |&(a, b, n, k)| x87::sum_times_int_over_minus(a, b, n, k),
        );
        check(
            "mul_div2",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, b, c, d)| x87::mul_div2(a, b, c, d),
        );
        check(
            "int_mul_add",
            &ROUNDING,
            |g| {
                (
                    g.i(),
                    g.f(),
                    g.f(),
                    if g.below(2) == 0 { None } else { Some(g.f()) },
                )
            },
            |&(i, k, a, b)| x87::int_mul_add(i, k, a, b),
        );
        check(
            "ext_div2",
            &ROUNDING,
            |g| (g.e80(), g.f(), g.f()),
            |&(e, a, b)| x87::ext_div2(e, a, b),
        );
        check("neg_rounded", &ROUNDING, Rng::e80, |&x| x87::neg_rounded(x));
        check(
            "lever_share",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f(), g.fw()),
            |&(a, d, b, c, z)| {
                let (share, zero) = x87::lever_share(a, d, b, c, z);
                (share.to_bits(), zero)
            },
        );
        // Zero sums: the equal-to-zero branch needs them.
        check(
            "lever_share (zeros)",
            &ROUNDING,
            |g| {
                (
                    g.f() & 0x8000_0000,
                    g.f() & 0x8000_0000,
                    g.f() & 0x8000_0000,
                    g.f() & 0x8000_0000,
                    g.f() & 0x8000_0000,
                )
            },
            |&(a, d, b, c, z)| {
                let (share, zero) = x87::lever_share(a, d, b, c, z);
                (share.to_bits(), zero)
            },
        );
        // The sums over the divisors: a list each.
        check(
            "sum_over",
            &ROUNDING,
            |g| {
                let t: Vec<u32> = (0..1 + g.below(5)).map(|_| g.f()).collect();
                let d: Vec<u32> = (0..g.below(4)).map(|_| g.f()).collect();
                (t, d)
            },
            |(t, d)| x87::sum_over(t, d),
        );
        check(
            "ramp",
            &ROUNDING,
            |g| {
                let mut t = [g.fw(), g.f(), g.f(), g.f()];
                // Now and then the ends equal or in order, so that every branch is seen.
                match g.below(4) {
                    0 => t[2] = t[0],
                    1 => {
                        let (lo, hi) = (g.pos(), g.pos());
                        t[0] = lo.min(hi);
                        t[2] = lo.max(hi);
                    }
                    _ => {}
                }
                let x = if g.below(3) == 0 { t[0] } else { g.fw() };
                (x, t, if g.below(4) == 0 { 0 } else { g.fw() })
            },
            |&(x, t, z)| x87::ramp(x, t, z),
        );
    }

    #[test]
    fn run_comparisons() {
        check(
            "compare_full",
            &ROUNDING,
            |g| (g.fw(), g.fw()),
            |&(a, b)| x87::compare_full(a, b),
        );
        check(
            "compare_full (equal)",
            &ROUNDING,
            |g| {
                let a = g.f();
                (a, a)
            },
            |&(a, b)| x87::compare_full(a, b),
        );
        check(
            "abs_compare",
            &ROUNDING,
            |g| (g.fw(), g.fw()),
            |&(a, b)| x87::abs_compare(a, b),
        );
        check(
            "mul_div_status",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.fw()),
            |&(a, b, c, k)| x87::mul_div_status(a, b, c, k),
        );
        check(
            "mul_status",
            &ROUNDING,
            |g| (g.f(), g.f(), g.fw()),
            |&(a, b, c)| x87::mul_status(a, b, c),
        );
        check(
            "compare_80",
            &ROUNDING,
            |g| (g.e80(), g.fw()),
            |&(e, b)| x87::compare_80(e, b),
        );
    }

    #[test]
    fn run_extended_results() {
        check(
            "length_80",
            &ROUNDING,
            |g| [g.f(), g.f(), g.f()],
            |v| x87::length_80(*v),
        );
        check(
            "scale_by_80",
            &ROUNDING,
            |g| ([g.f(), g.f(), g.f()], g.e80()),
            |&(v, s)| x87::scale_by_80(v, s),
        );
        check(
            "less_plus_less_product_80",
            &ROUNDING,
            |g| {
                (
                    g.f(),
                    g.f(),
                    if g.below(2) == 0 { None } else { Some(g.f()) },
                    g.f(),
                    g.f(),
                )
            },
            |&(a, b, c, d, e)| x87::less_plus_less_product_80(a, b, c, d, e),
        );
        check(
            "one_less_products",
            &ROUNDING,
            |g| {
                (0..g.below(4))
                    .map(|_| [g.f(), g.f(), g.f()])
                    .collect::<Vec<_>>()
            },
            |t| x87::one_less_products(t),
        );
        check(
            "ext_times",
            &ROUNDING,
            |g| (g.e80(), g.f()),
            |&(e, k)| x87::ext_times(e, k),
        );
        check(
            "int_80",
            &ROUNDING,
            |g| (g.i(), g.below(2) == 0),
            |&(a, n)| x87::int_80(a, n),
        );
        check(
            "gear_step_80",
            &ROUNDING,
            |g| (g.f(), g.f(), g.i(), g.f()),
            |&(t, l, n, o)| x87::gear_step_80(t, l, n, o),
        );
        check(
            "ext_add",
            &ROUNDING,
            |g| (g.e80(), g.e80()),
            |&(a, b)| x87::ext_add(a, b),
        );
        check(
            "ints_diff_80",
            &ROUNDING,
            |g| (g.i(), g.i()),
            |&(a, b)| x87::ints_diff_80(a, b),
        );
        check(
            "div_80",
            &ROUNDING,
            |g| (g.f(), g.f()),
            |&(a, b)| x87::div_80(a, b),
        );
    }

    #[test]
    fn run_face_shares() {
        // The SSE path's `rcpss` is the tables' (`arch::rcpss`), the same on every processor.
        for sse in [false, true] {
            let name = if sse {
                "face_shares (sse)"
            } else {
                "face_shares"
            };
            check_all(
                name,
                |g| {
                    (
                        [
                            g.i() as u32,
                            g.i() as u32,
                            g.i() as u32,
                            g.i() as u32,
                            g.i() as u32,
                            g.i() as u32,
                            g.f(),
                        ],
                        sse,
                    )
                },
                |&(i, sse)| {
                    let mut o = [0xAAAA_AAAAu32; 8];
                    {
                        x87::face_shares_bits(&i, &mut o, sse);
                    };
                    o
                },
            );
        }
    }

    // ---- the transcendental ones (within an ulp of a float against the x87, two of a double)

    #[test]
    fn run_sine_cosine_of() {
        check("sine_cosine_of", &ROUNDING, Rng::angle, |&a| {
            x87::sine_cosine_of(a)
        });
    }

    #[test]
    fn run_patan() {
        check(
            "patan",
            &ROUNDING,
            |g| (g.f(), g.f()),
            |&(y, x)| x87::patan(y, x),
        );
    }

    /// `a / cos b`, over the angles the game gives it (the pitch, within +-pi/2). (The x87's
    /// `fcos` reduces the argument with a 66-bit pi, so it differed from libm's true cosine
    /// by up to 2 ulp or that reduction's error.)
    #[test]
    fn run_over_cos_80() {
        check(
            "over_cos_80",
            &ROUNDING,
            |g| {
                (
                    g.f(),
                    match g.below(4) {
                        0 => 0x3FC9_0FDB, // pi / 2
                        1 => 0xBFC9_0FDB,
                        _ => (g.u32() as f32 / u32::MAX as f32 * 3.2 - 1.6).to_bits(),
                    },
                )
            },
            |&(a, b)| x87::over_cos_80(a, b),
        );
    }

    #[test]
    fn run_clamped_asin_80() {
        check(
            "clamped_asin_80",
            &ROUNDING,
            |g| {
                let x = match g.below(3) {
                    0 => f64_to_f80((g.u32() as f64 / u32::MAX as f64 - 0.5) * 2.4),
                    1 => g.e80(),
                    _ => f64_to_f80(f64::from(f32::from_bits(g.f())).clamp(-1.0, 1.0)),
                };
                let l = match g.below(4) {
                    0 => g.fw(),
                    1 => 0x3F80_0000,
                    _ => (g.u32() as f32 / u32::MAX as f32 * 1.5).to_bits(),
                };
                (x, g.f(), l)
            },
            |&(x, s, l)| {
                let (mut s, mut l) = (s, l);
                (x87::clamped_asin_80(x, &mut s, &mut l), s, l)
            },
        );
    }

    // ---- normalise_vector's pieces ----

    #[test]
    fn norm_squares_and_scaling() {
        check(
            "normalise squares_sum",
            &ROUNDING,
            |g| [g.f(), g.f(), g.f()],
            |&v| normalise_vector::squares_sum(v),
        );
        check(
            "normalise scaled_by_inverse",
            &ROUNDING,
            |g| ([g.f(), g.f(), g.f()], g.pos(), g.pos()),
            |&(v, r, e)| normalise_vector::scaled_by_inverse(v, r, e),
        );
    }

    /// The results' hashes (`RC2K_FREEZE_OUT` makes them again).
    const FROZEN: &[(&str, u64)] = &[
        ("c::abs_below_or_unordered", 0xefdcf7ed1d52f617),
        ("c::abs_compare", 0xa57d27a8a3d8d0a2),
        ("c::abs_div_add_mul", 0x92841428ad5ac958),
        ("c::abs_fidiv", 0xeb1a767e2b3c0ed0),
        ("c::abs_of", 0x6d3bc813b2f125f0),
        ("c::abs_sqrt_times_plus", 0xf9aa6b2b3203ac56),
        ("c::abs_to_int", 0xcca82a20930f364d),
        ("c::add3", 0x2ffe93c877df673a),
        ("c::add_to_int", 0x587ee7f6849a41e7),
        ("c::below_or_unordered", 0x07fbc0cd8d5ddd84),
        ("c::clamped_asin_80", 0x02bd8747270ab162),
        ("c::compare_80", 0x159b680164d7f4ed),
        ("c::compare_full (equal)", 0x4d11b71ea20e7725),
        ("c::compare_full", 0xd2d8d91a7437fd56),
        ("c::div_80", 0xf06fa48a7a8bc854),
        ("c::div_mul3_to_int", 0x23e34aeb00269ba6),
        ("c::div_then_mul", 0x9e09272282abbb8d),
        ("c::eased_roll", 0xab4f5284a25277d2),
        ("c::ext_add", 0x888835afd96475b7),
        ("c::ext_div2", 0xeabd61a8e8266182),
        ("c::ext_times", 0x2937055d4357017b),
        ("c::ext_to_int", 0xe6e3a348b32b4a00),
        ("c::f64_to_f80", 0xe36bb580e035c5d4),
        ("c::f80_to_f64", 0x31c9fefbec2c18ba),
        ("c::face_shares (sse)", 0x233b70d25f0546a5),
        ("c::face_shares", 0xf10173da5b8c36db),
        ("c::float_hypot", 0x56337ddfe7023d78),
        ("c::float_hypot_to_int", 0x6df31d1d7078e7f8),
        ("c::float_square_sum", 0x67d4f52727b47f6c),
        ("c::gear_step_80", 0x960c76ab8129a97b),
        ("c::int_80", 0xb25bdf0548295041),
        ("c::int_fidiv", 0x31058920941290ce),
        ("c::int_lerp", 0xc07201439038809d),
        ("c::int_mul_add", 0x657af4012b6556ea),
        ("c::int_over_float", 0x155b65e31987a251),
        ("c::int_over_int_plus", 0x714789814f6c1cc9),
        ("c::int_plus_times", 0xa34e486ad11513b0),
        ("c::int_square_sum", 0x543557d41383e3f8),
        ("c::int_square_sum_scaled_to_int", 0x6418c725881be0d8),
        ("c::int_times_int_over_to_int", 0xa5694f1e56c9f304),
        ("c::int_times_times_over_to_int", 0x1e5444648c50bb96),
        ("c::int_toward_to_int", 0x616e4050b794bc74),
        ("c::ints_diff_80", 0x3df35b67a7483241),
        ("c::length_80", 0x598b9770801192eb),
        ("c::lerp_int_over2", 0x3d31c6bff9bcc5c8),
        ("c::lerp_table", 0x62c2a5a61843c811),
        ("c::less_plus_less_product_80", 0x7f1f64d5cbe97164),
        ("c::lever_share (zeros)", 0xcfe44737473621a5),
        ("c::lever_share", 0x49c6ab6a80f70e9c),
        ("c::minus_one", 0xaeb44a46c34d7725),
        ("c::mul2_plus", 0x7216f25d14339293),
        ("c::mul2_to_int", 0x432a462f4b4fd580),
        ("c::mul3", 0xa416a64449eb3742),
        ("c::mul3_to_int", 0x76c4e48902e96ac0),
        ("c::mul_div2", 0x61e4f46c7a81964c),
        ("c::mul_div_status", 0x744b94d347feb5ce),
        ("c::mul_status", 0xdc390b82a0f1294e),
        ("c::neg_mul", 0x6379026075d5b009),
        ("c::neg_mul2", 0xf22fceed972eccc5),
        ("c::neg_product2_plus", 0x69f298341c504569),
        ("c::neg_product_plus", 0xf8169e4914be541f),
        ("c::neg_rounded", 0x1777fd326550403e),
        ("c::normalise scaled_by_inverse", 0xd5aa834da6876e1c),
        ("c::normalise squares_sum", 0x86527f767c38250c),
        ("c::one_less", 0xf7e445777d3ac734),
        ("c::one_less_products", 0x1dda37c844b29a77),
        ("c::one_less_times_plus", 0x649a7ec3b6938ebe),
        ("c::over_abs_plus_minus", 0xc20f1dfb4dde716e),
        ("c::over_cos_80", 0x178f9da45b1105e2),
        ("c::patan", 0x487494675eab42a4),
        ("c::plane_height", 0x4b12fb5669b1f634),
        ("c::product3", 0x62b0c4aede7e7ff5),
        ("c::product4_to_int", 0x4473fe25c80c2697),
        ("c::ramp", 0x29362dad07509496),
        ("c::rotate_rounding", 0x8dad910931c15eac),
        ("c::rotate_rounding_z", 0x8c619212b7dd803d),
        ("c::scale_by_80", 0xb2928e43f552cd75),
        ("c::scaled_abs_less_below", 0x877fd5bbc4113eb3),
        ("c::sine_cosine_of", 0x63eac2c62c3fbd0e),
        ("c::square", 0x9271facf3e0515c5),
        ("c::square_times_over", 0xbd41e856539b5ed4),
        ("c::square_times_plus_times", 0xd9b9aca22845582e),
        ("c::sub_div_to_int", 0xb3282fbbe61e6d4a),
        ("c::sub_mul", 0xf0cceca48b6ae14a),
        ("c::sub_mul_add", 0xd801d01455395549),
        ("c::sum_over", 0x3542515d9f2741f4),
        ("c::sum_times_int_over_minus", 0x27b1045246757f49),
        ("c::sum_times_over", 0x01be69a5684aff06),
        ("c::sun_direction", 0xfcd0d2c3328fe182),
        ("c::sun_on_screen", 0xf7f1106e1b11161e),
        ("c::sun_size", 0xff9c10c637a38e17),
        ("c::tbyte_to_int", 0xd8cbba1d59de5df9),
        ("c::times_fist_compare", 0x078e27a690026d16),
        ("c::to_tbyte", 0x0c6a9352a37b5912),
        ("c::toward_one", 0x069fc4e0bfd103fb),
        ("c::trunc_mul_to_int", 0x40945123d2358281),
        ("c::weight_force", 0xf62a08919b3a9971),
        ("c::wheel_int", 0x2696e1c0e45f4a04),
    ];
}
