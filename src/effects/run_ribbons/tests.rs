//! The loop and the shared turn of `run_ribbons` against the earlier unrolled versions (`old`),
//! and the recorded vectors.

use super::*;

/// The previous versions of the x87 blocks (unrolled, one copy of the turn per block), kept as
/// the reference the loop and the shared turn are compared against.
mod old {
    use super::*;
    use crate::effects::ring_entries::{ld, st};
    use crate::math::x87_float::int;
    /// The previous version of `first_frame`'s arithmetic as plain Rust on the x87 model, in the
    /// same order.
    pub(super) fn first_frame_x87(
        view_matrix: &[crate::memory::session::PlainFloat; 16],
        ring: &Ring,
        e: u32,
        c: &MarksClock,
        ribbon_a: f32,
    ) {
        let t0 = ld(ring, e, |en| en.place[2]);
        let t1 = ld(ring, e, |en| en.place[1]);
        let t2 = ld(ring, e, |en| en.place[0]);
        let t3 = view_matrix[0].ld();
        let t4 = t3 * t2;
        let t5 = view_matrix[4].ld();
        let t6 = t5 * t1;
        let t7 = view_matrix[8].ld();
        let t8 = t7 * t0;
        let t9 = t6 + t8;
        let t10 = t4 + t9;
        let t11 = t10 + view_matrix[12].ld();
        let t12 = view_matrix[1].ld();
        let t13 = t12 * t2;
        let t14 = view_matrix[5].ld();
        let t15 = t14 * t1;
        let t16 = view_matrix[9].ld();
        let t17 = t16 * t0;
        let t18 = t15 + t17;
        let t19 = t13 + t18;
        let t20 = t19 + view_matrix[13].ld();
        let t21 = view_matrix[2].ld();
        let t22 = t2 * t21;
        let t23 = view_matrix[6].ld();
        let t24 = t1 * t23;
        let t25 = view_matrix[10].ld();
        let t26 = t0 * t25;
        let t27 = t26 + t24;
        let t28 = t22 + t27;
        let t29 = t28 + view_matrix[14].ld();
        st(ring, e, t11, |en| &mut en.last_place[0]);
        st(ring, e, t20, |en| &mut en.last_place[1]);
        st(ring, e, t29, |en| &mut en.last_place[2]);
        let t30 = ld(ring, e, |en| en.motion[0]);
        let t31 = t30 * int(c.frame_ticks.get() as i32);
        let t32 = t31 + ld(ring, e, |en| en.place[0]);
        st(ring, e, t32, |en| &mut en.place[0]);
        let t33 = ld(ring, e, |en| en.motion[1]);
        let t34 = t33 * int(c.frame_ticks.get() as i32);
        let t35 = t34 + ld(ring, e, |en| en.place[1]);
        st(ring, e, t35, |en| &mut en.place[1]);
        let t36 = ld(ring, e, |en| en.motion[2]);
        let t37 = t36 * int(c.frame_ticks.get() as i32);
        let t38 = t37 + ld(ring, e, |en| en.place[2]);
        st(ring, e, t38, |en| &mut en.place[2]);
        let t39 = bh::ld(ribbon_a);
        let t40 = t39 * int(c.frame_ticks.get() as i32);
        let t41 = t40 + ld(ring, e, |en| en.motion[1]);
        st(ring, e, t41, |en| &mut en.motion[1]);
    }

    /// The previous version of `move_`'s arithmetic as plain Rust on the x87 model, in the same
    /// order.
    pub(super) fn move_x87(ring: &Ring, e: u32, c: &MarksClock, ribbon_b: f32) {
        let t0 = ld(ring, e, |en| en.motion[0]);
        let t1 = t0 * int(c.frame_ticks.get() as i32);
        let t2 = t1 + ld(ring, e, |en| en.place[0]);
        st(ring, e, t2, |en| &mut en.place[0]);
        let t3 = ld(ring, e, |en| en.motion[1]);
        let t4 = t3 * int(c.frame_ticks.get() as i32);
        let t5 = t4 + ld(ring, e, |en| en.place[1]);
        st(ring, e, t5, |en| &mut en.place[1]);
        let t6 = ld(ring, e, |en| en.motion[2]);
        let t7 = t6 * int(c.frame_ticks.get() as i32);
        let t8 = t7 + ld(ring, e, |en| en.place[2]);
        st(ring, e, t8, |en| &mut en.place[2]);
        let t9 = bh::ld(ribbon_b);
        let t10 = t9 * int(c.frame_ticks.get() as i32);
        let t11 = t10 + ld(ring, e, |en| en.motion[1]);
        st(ring, e, t11, |en| &mut en.motion[1]);
    }

    /// The previous version of `turn_place`'s arithmetic as plain Rust on the x87 model, in the
    /// same order.
    pub(super) fn turn_place_x87(
        view_matrix: &[crate::memory::session::PlainFloat; 16],
        ring: &Ring,
        e: u32,
        dw: &DrawWork,
    ) {
        let t0 = ld(ring, e, |en| en.place[2]);
        let t1 = ld(ring, e, |en| en.place[1]);
        let t2 = ld(ring, e, |en| en.place[0]);
        let t3 = view_matrix[0].ld();
        let t4 = t3 * t2;
        let t5 = view_matrix[4].ld();
        let t6 = t5 * t1;
        let t7 = view_matrix[8].ld();
        let t8 = t7 * t0;
        let t9 = t6 + t8;
        let t10 = t4 + t9;
        let t11 = t10 + view_matrix[12].ld();
        let t12 = view_matrix[1].ld();
        let t13 = t12 * t2;
        let t14 = view_matrix[5].ld();
        let t15 = t14 * t1;
        let t16 = view_matrix[9].ld();
        let t17 = t16 * t0;
        let t18 = t15 + t17;
        let t19 = t13 + t18;
        let t20 = t19 + view_matrix[13].ld();
        let t21 = view_matrix[2].ld();
        let t22 = t2 * t21;
        let t23 = view_matrix[6].ld();
        let t24 = t1 * t23;
        let t25 = view_matrix[10].ld();
        let t26 = t0 * t25;
        let t27 = t26 + t24;
        let t28 = t22 + t27;
        let t29 = t28 + view_matrix[14].ld();
        dw.turned[0].st(t11);
        dw.turned[1].st(t20);
        dw.turned[2].st(t29);
    }
}

mod same_as_old {
    use super::*;
    use crate::test_support::own::effects;

    fn next(state: &mut u64) -> u32 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        (*state >> 16) as u32
    }

    /// A float word: mostly ordinary, sometimes zero, huge, tiny, negative or not a number.
    fn word(state: &mut u64) -> u32 {
        match next(state) % 16 {
            0 => 0,
            1 => 0x7FC0_0000,
            2 => 0x7F00_0000 | next(state) & 0x00FF_FFFF,
            3 => next(state) & 0x007F_FFFF,
            _ => f32::to_bits((next(state) as i32 % 100_000) as f32 / 37.0),
        }
    }

    /// Words are compared bit for bit except that any two NaNs count as equal: when both operands
    /// of an add or multiply are NaNs, which payload survives depends on the order the compiler
    /// gave the operands (it differs between profiles), and nothing reads a NaN's payload.
    #[test]
    fn the_ribbon_blocks_match_the_previous_versions() {
        let _lock = crate::test_support::test_lock();
        let ring = &effects().ring;
        let view_matrix = &crate::test_support::own::race()
            .frame
            .matrices
            .get()
            .view_matrix;
        let clock = &MarksClock::new();
        let kept_matrix: Vec<u32> = view_matrix
            .iter()
            .map(crate::memory::session::Cell32::bits)
            .collect();
        let figures = crate::memory::figures();
        let memory = crate::memory::ListCells::empty();
        crate::effects::ring_entries::take(ring, &memory).unwrap();
        let kept_turned: Vec<u32> = effects()
            .frame
            .draw_work
            .turned
            .iter()
            .map(crate::memory::session::Cell32::bits)
            .collect();
        let mut state = 0x9E37_79B9_7F4A_7C15u64;
        for case in 0..12_000 {
            for c in view_matrix {
                c.set_bits(word(&mut state));
            }
            clock.frame_ticks.set(next(&mut state) % 40);
            let entry =
                crate::effects::ring_entries::Entry::from_words(std::array::from_fn(|_| {
                    word(&mut state)
                }));
            let gain = f32::from_bits(word(&mut state));
            let mut results = Vec::new();
            for which in 0..2 {
                ring.entries.with(|r| r[0] = entry).unwrap();
                let turned = &effects().frame.draw_work.turned;
                for c in turned.iter() {
                    c.set_bits(0xAAAA_AAAA);
                }
                match (case % 3, which) {
                    (0, 0) => old::first_frame_x87(view_matrix, ring, 1, clock, gain),
                    (0, _) => first_frame_x87(view_matrix, ring, 1, clock, gain),
                    (1, 0) => old::move_x87(ring, 1, clock, gain),
                    (1, _) => move_x87(ring, 1, clock, gain),
                    (_, 0) => old::turn_place_x87(view_matrix, ring, 1, &effects().frame.draw_work),
                    (_, _) => turn_place_x87(view_matrix, ring, 1, &effects().frame.draw_work),
                }
                let mut bytes = ring.entries.with(|r| r[0].words().to_vec()).unwrap();
                bytes.extend(turned.iter().map(crate::memory::session::Cell32::bits));
                results.push(bytes);
            }
            let same_but_for_nan_payloads = results[0].len() == results[1].len()
                && results[0].iter().zip(&results[1]).all(|(&a, &b)| {
                    a == b || (f32::from_bits(a).is_nan() && f32::from_bits(b).is_nan())
                });
            assert!(
                same_but_for_nan_payloads,
                "case {case}: {:?} against {:?}",
                results[0], results[1]
            );
        }
        for (c, w) in view_matrix.iter().zip(kept_matrix) {
            c.set_bits(w);
        }
        for (c, w) in effects().frame.draw_work.turned.iter().zip(kept_turned) {
            c.set_bits(w);
        }
        ring.entries.give_back();
        crate::memory::with_figures(|f| *f = figures);
    }
}

mod reference_vectors {
    use crate::effects::asm_layout::with_laid;
    use crate::effects::asm_test::Kind::*;

    use crate::effects::asm_test::{Op, Spec, recorded_writes};
    use crate::effects::frame_state::DrawWork;
    use crate::test_support::own::effects;

    fn word(at: crate::effects::asm_test::Operand) -> u32 {
        // SAFETY: `at` checks that the four bytes lie in the operand's test buffer, which the
        // recorded run keeps alive and nothing else reaches meanwhile; unaligned, any bits a `u32`.
        unsafe { at.at(0, 4).cast::<u32>().read_unaligned() }
    }

    #[test]
    fn first_frame_x87_matches() {
        let ring = &effects().ring;
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[
                        (0x1EA2CC, F32),
                        (0x1EA2D0, F32),
                        (0x1EA2D4, F32),
                        (0x1EA2DC, F32),
                        (0x1EA2E0, F32),
                        (0x1EA2E4, F32),
                        (0x1EA2EC, F32),
                        (0x1EA2F0, F32),
                        (0x1EA2F4, F32),
                        (0x1EA2FC, F32),
                        (0x1EA300, F32),
                        (0x1EA304, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x90,
                    hi: 0x94,
                    reads: &[(0x90, I32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x10,
                    hi: 0x38,
                    reads: &[
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x2C, F32),
                        (0x30, F32),
                        (0x34, F32),
                    ],
                    writes: &[
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x1C, F32),
                        (0x20, F32),
                        (0x24, F32),
                        (0x30, F32),
                    ],
                },
                Op::Base {
                    lo: 0x94,
                    hi: 0x98,
                    reads: &[(0x94, F32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        recorded_writes("effects.run_ribbons.first_frame_x87", &SPEC, |b| {
            crate::effects::ring_entries::with_entry_at(ring, b[2], 0x10, 0x38, |e| {
                super::first_frame_x87(
                    &crate::test_support::own::race()
                        .frame
                        .matrices
                        .get()
                        .view_matrix,
                    ring,
                    e,
                    crate::effects::asm_layout::copy_of::<crate::effects::setup::MarksClock>(b[1]),
                    f32::from_bits(word(b[3] + 0x94)),
                );
            });
        });
    }

    #[test]
    fn move_x87_matches() {
        let ring = &effects().ring;
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x90,
                    hi: 0x94,
                    reads: &[(0x90, I32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x10,
                    hi: 0x38,
                    reads: &[
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x2C, F32),
                        (0x30, F32),
                        (0x34, F32),
                    ],
                    writes: &[(0x10, F32), (0x14, F32), (0x18, F32), (0x30, F32)],
                },
                Op::Base {
                    lo: 0x98,
                    hi: 0x9C,
                    reads: &[(0x98, F32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        recorded_writes("effects.run_ribbons.move_x87", &SPEC, |b| {
            crate::effects::ring_entries::with_entry_at(ring, b[1], 0x10, 0x38, |e| {
                super::move_x87(
                    ring,
                    e,
                    crate::effects::asm_layout::copy_of::<crate::effects::setup::MarksClock>(b[0]),
                    f32::from_bits(word(b[2] + 0x98)),
                );
            });
        });
    }

    #[test]
    fn turn_place_x87_matches() {
        let ring = &effects().ring;
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[
                        (0x1EA2CC, F32),
                        (0x1EA2D0, F32),
                        (0x1EA2D4, F32),
                        (0x1EA2DC, F32),
                        (0x1EA2E0, F32),
                        (0x1EA2E4, F32),
                        (0x1EA2EC, F32),
                        (0x1EA2F0, F32),
                        (0x1EA2F4, F32),
                        (0x1EA2FC, F32),
                        (0x1EA300, F32),
                        (0x1EA304, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x186931,
                    hi: 0x186A31,
                    reads: &[],
                    writes: &[(0x186995, F32), (0x186999, F32), (0x18699D, F32)],
                },
                Op::Base {
                    lo: 0x10,
                    hi: 0x1C,
                    reads: &[(0x10, F32), (0x14, F32), (0x18, F32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        recorded_writes("effects.run_ribbons.turn_place_x87", &SPEC, |b| {
            with_laid::<DrawWork, _>(b[1], |dw| {
                crate::effects::ring_entries::with_entry_at(ring, b[2], 0x10, 0x1C, |e| {
                    super::turn_place_x87(
                        &crate::test_support::own::race()
                            .frame
                            .matrices
                            .get()
                            .view_matrix,
                        ring,
                        e,
                        dw,
                    );
                });
            });
        });
    }
}
