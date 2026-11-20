use crate::effects::asm_layout::with_laid;
use crate::effects::asm_test::Kind::*;
use crate::effects::asm_test::staged_vertices as staged;

use crate::effects::asm_test::{Op, Spec, recorded_writes};
use crate::effects::tuning_store::MarksKindFloats;
use crate::effects::windscreen_flakes_store::{WindscreenFlakeItem, WindscreenFlakeTables};

#[test]
fn shifted_places_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0xC,
                hi: 0x14,
                reads: &[(0xC, F32), (0x10, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x18,
                hi: 0x80,
                reads: &[
                    (0x18, F32),
                    (0x1C, F32),
                    (0x38, F32),
                    (0x3C, F32),
                    (0x58, F32),
                    (0x5C, F32),
                    (0x78, F32),
                    (0x7C, F32),
                ],
                writes: &[
                    (0x18, F32),
                    (0x1C, F32),
                    (0x38, F32),
                    (0x3C, F32),
                    (0x58, F32),
                    (0x5C, F32),
                    (0x78, F32),
                    (0x7C, F32),
                ],
            },
        ],
        sincos: false,
    };
    recorded_writes(
        "effects.draw_windscreen_flake.shifted_places_x87",
        &SPEC,
        |b| {
            let word =
                // SAFETY: `at` checks that the four bytes lie in the operand's test buffer,
                // which the recorded run keeps alive and nothing else reaches meanwhile;
                // unaligned, any bits a `u32`.
                |off: usize| unsafe { (b[0] + off).at(0, 4).cast::<u32>().read_unaligned() };
            let item = WindscreenFlakeItem::new();
            item.texture_u.set_bits(word(0xC));
            item.texture_v.set_bits(word(0x10));
            staged(
                b[1],
                &[0x18, 0x1C, 0x38, 0x3C, 0x58, 0x5C, 0x78, 0x7C],
                || super::shifted_places_x87(&crate::d3d::vertices_store::VERTICES, &item),
            );
        },
    );
}

#[test]
fn corners_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x137B8A,
                hi: 0x13822A,
                reads: &[
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
                    (0x137FFE, F32),
                    (0x138002, F32),
                    (0x138006, F32),
                    (0x13800A, F32),
                    (0x13800E, F32),
                    (0x138012, F32),
                    (0x138016, F32),
                    (0x13801A, F32),
                    (0x13801E, F32),
                    (0x138022, F32),
                    (0x138026, F32),
                    (0x13802A, F32),
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
                lo: 0x0,
                hi: 0x14,
                reads: &[(0x0, F80), (0xA, F80)],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_windscreen_flake.corners_x87", &SPEC, |b| {
        with_laid::<std::cell::RefCell<WindscreenFlakeTables>, _>(b[0], |sq| {
            let v: [[u8; 10]; 2] = std::array::from_fn(|n| {
                // SAFETY: `host` checks that the byte lies in the operand's test buffer, which
                // the recorded run keeps alive and nothing else reaches meanwhile.
                std::array::from_fn(|i| unsafe { (b[1] + 10 * n + i).host().read() })
            });
            super::corners_x87(&v, &mut sq.borrow_mut());
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
                lo: 0x2F8,
                hi: 0x13822A,
                reads: &[
                    (0x35C, F32),
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
    recorded_writes("effects.draw_windscreen_flake.projected_x87", &SPEC, |b| {
        with_laid::<std::cell::RefCell<MarksKindFloats>, _>(b[1], |k| {
            with_laid::<std::cell::RefCell<WindscreenFlakeTables>, _>(b[1], |sq| {
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
                            &k.borrow(),
                            &mut sq.borrow_mut(),
                        );
                    },
                );
            });
        });
    });
}
