use crate::effects::asm_layout::with_laid;
use crate::effects::asm_test::Kind::*;
use crate::effects::asm_test::staged_vertices as staged;
use crate::effects::asm_test::{Op, Spec, recorded_writes};
use crate::effects::frame_state::DrawWork;
use crate::effects::ring_store::RingWork;

#[test]
fn turn_x87_matches() {
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
                hi: 0x1D0826,
                reads: &[],
                writes: &[
                    (0x186995, F32),
                    (0x186999, F32),
                    (0x18699D, F32),
                    (0x1D0780, I32),
                ],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x1E,
                reads: &[(0x0, F80), (0xA, F80), (0x14, F80)],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_subcell_sprite.turn_x87", &SPEC, |b| {
        with_laid::<RingWork, _>(b[1], |w| {
            with_laid::<DrawWork, _>(b[1], |dw| {
                // SAFETY: `at` checks that the ten bytes lie in the operand's test buffer,
                // which the recorded run keeps alive and nothing else reaches meanwhile;
                // unaligned.
                let place: [[u8; 10]; 3] = std::array::from_fn(|n| unsafe {
                    (b[2] + 10 * n)
                        .at(0, 10)
                        .cast::<[u8; 10]>()
                        .read_unaligned()
                });
                super::turn_x87(
                    &crate::test_support::own::race()
                        .frame
                        .matrices
                        .get()
                        .view_matrix,
                    &place,
                    w,
                    dw,
                );
            });
        });
    });
}

#[test]
fn square_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::RaceBlock {
                reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
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
            Op::Base {
                lo: 0x186931,
                hi: 0x186A31,
                reads: &[
                    (0x186975, F32),
                    (0x186979, F32),
                    (0x186995, F32),
                    (0x186999, F32),
                    (0x18699D, F32),
                    (0x186A2D, F32),
                ],
                writes: &[(0x186975, F32), (0x186979, F32)],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_subcell_sprite.square_x87", &SPEC, |b| {
        with_laid::<DrawWork, _>(b[2], |dw| {
            staged(
                b[1],
                &[
                    0x0, 0x4, 0x8, 0x20, 0x24, 0x28, 0x40, 0x44, 0x48, 0x60, 0x64, 0x68,
                ],
                || {
                    super::square_x87(
                        crate::owners::Owners::the()
                            .scene
                            .scratch
                            .front_end_polygon(),
                        &crate::d3d::vertices_store::VERTICES,
                        dw,
                    );
                },
            );
        });
    });
}
