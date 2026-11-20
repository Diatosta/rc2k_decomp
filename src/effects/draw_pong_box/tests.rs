use crate::effects::asm_layout::with_laid;
use crate::effects::asm_test::Kind::*;
use crate::effects::asm_test::staged_vertices as staged;

use crate::effects::asm_test::{Op, Spec, recorded_writes};
use crate::effects::pong_store::PongState;
use crate::effects::tuning_store::MarksFloats;
use crate::effects::windscreen_flakes_store::WindscreenFlakeTables;

/// The four ten-byte values at `at` of the test buffer (0 where the buffer does not
/// reach: a block reads only the last two).
fn values(at: crate::effects::asm_test::Operand) -> [[u8; 10]; 4] {
    std::array::from_fn(|n| at.bytes_or_zero(10 * n, 10).try_into().unwrap())
}

#[test]
fn sides_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x200,
                hi: 0x13822A,
                reads: &[
                    (0x218, F32),
                    (0x137FAE, F32),
                    (0x137FB2, F32),
                    (0x137FDA, F32),
                    (0x137FDE, F32),
                    (0x137FE2, F32),
                    (0x137FE6, F32),
                    (0x137FEA, F32),
                    (0x137FEE, F32),
                ],
                writes: &[
                    (0x3718, F32),
                    (0x371C, F32),
                    (0x3720, F32),
                    (0x3724, F32),
                    (0x3728, F32),
                    (0x372C, F32),
                    (0x3730, F32),
                    (0x3734, F32),
                    (0x3738, F32),
                    (0x373C, F32),
                    (0x3740, F32),
                    (0x3744, F32),
                ],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x28,
                reads: &[(0x0, F80), (0xA, F80), (0x14, F80), (0x1E, F80)],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_pong_box.sides_x87", &SPEC, |b| {
        with_laid::<MarksFloats, _>(b[0], |f| {
            with_laid::<WindscreenFlakeTables, _>(b[0], |sq| {
                with_laid::<PongState, _>(b[0], |state| {
                    super::sides_x87(&values(b[1]), state, f, sq);
                });
            });
        });
    });
}

#[test]
fn corners_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x3718,
                hi: 0x13822A,
                reads: &[
                    (0x3718, F32),
                    (0x371C, F32),
                    (0x3720, F32),
                    (0x3724, F32),
                    (0x3728, F32),
                    (0x372C, F32),
                    (0x137FAE, F32),
                    (0x137FB2, F32),
                    (0x137FB6, F32),
                    (0x137FBA, F32),
                    (0x137FBE, F32),
                    (0x137FDA, F32),
                    (0x137FDE, F32),
                    (0x137FE2, F32),
                    (0x137FE6, F32),
                    (0x137FEA, F32),
                    (0x137FEE, F32),
                    (0x138052, F32),
                    (0x138056, F32),
                    (0x13805A, F32),
                ],
                writes: &[
                    (0x138052, F32),
                    (0x138056, F32),
                    (0x13805A, F32),
                    (0x13805E, F32),
                    (0x138062, F32),
                    (0x138066, F32),
                    (0x13806A, F32),
                    (0x13806E, F32),
                    (0x138072, F32),
                    (0x138076, F32),
                    (0x13807A, F32),
                    (0x13807E, F32),
                ],
            },
            Op::Base {
                lo: 0x14,
                hi: 0x28,
                reads: &[(0x14, F80), (0x1E, F80)],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_pong_box.corners_x87", &SPEC, |b| {
        with_laid::<WindscreenFlakeTables, _>(b[0], |sq| {
            with_laid::<PongState, _>(b[0], |state| {
                super::corners_x87(&values(b[1]), state, sq);
            });
        });
    });
}

#[test]
fn projected_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::RaceBlock {
                reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x137B8A,
                hi: 0x13822A,
                reads: &[
                    (0x137FAA, F32),
                    (0x138052, F32),
                    (0x138056, F32),
                    (0x13805A, F32),
                    (0x13805E, F32),
                    (0x138062, F32),
                    (0x138066, F32),
                    (0x13806A, F32),
                    (0x13806E, F32),
                    (0x138072, F32),
                    (0x138076, F32),
                    (0x13807A, F32),
                    (0x13807E, F32),
                ],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x6C,
                reads: &[],
                writes: &[
                    (0x0, F32),
                    (0x4, F32),
                    (0x8, F32),
                    (0x20, F32),
                    (0x24, F32),
                    (0x28, F32),
                    (0x40, F32),
                    (0x44, F32),
                    (0x48, F32),
                    (0x60, F32),
                    (0x64, F32),
                    (0x68, F32),
                ],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_pong_box.projected_x87", &SPEC, |b| {
        with_laid::<WindscreenFlakeTables, _>(b[1], |sq| {
            staged(
                b[2],
                &[
                    0x0, 0x4, 0x8, 0x20, 0x24, 0x28, 0x40, 0x44, 0x48, 0x60, 0x64, 0x68,
                ],
                || {
                    super::projected_x87(
                        crate::owners::Owners::the()
                            .scene
                            .scratch
                            .front_end_polygon(),
                        &crate::d3d::vertices_store::VERTICES,
                        sq,
                    );
                },
            );
        });
    });
}
