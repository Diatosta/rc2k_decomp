use crate::effects::asm_test::Kind::*;
use crate::effects::asm_test::staged_vertices as staged;
use crate::effects::asm_test::{Op, Spec, recorded_writes};
use crate::effects::rain_store::tests::with_state_at;

/// The three ten-byte values at `at` of the test buffer.
fn values(at: crate::effects::asm_test::Operand) -> [[u8; 10]; 3] {
    // SAFETY: `host` checks that the byte lies in the operand's test buffer, which the recorded
    // run keeps alive and nothing else reaches meanwhile.
    std::array::from_fn(|n| std::array::from_fn(|i| unsafe { (at + 10 * n + i).host().read() }))
}

#[test]
fn turn_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x1574C5,
                hi: 0x1610BD,
                reads: &[
                    (0x15753D, F32),
                    (0x157541, F32),
                    (0x157545, F32),
                    (0x158E25, F32),
                    (0x158E29, F32),
                    (0x158E2D, F32),
                    (0x158E31, I32),
                    (0x158E35, I32),
                    (0x158E39, I32),
                    (0x158E49, F32),
                    (0x158E4D, F32),
                    (0x158E51, F32),
                    (0x158E55, F32),
                    (0x158E59, F32),
                    (0x158E5D, F32),
                    (0x158E61, F32),
                    (0x158E65, F32),
                    (0x158E69, F32),
                    (0x158F09, F32),
                    (0x158F0D, F32),
                    (0x158F11, F32),
                ],
                writes: &[
                    (0x158E1D, F32),
                    (0x158EFD, F32),
                    (0x158F01, F32),
                    (0x158F05, F32),
                    (0x158F09, F32),
                    (0x158F0D, F32),
                    (0x158F11, F32),
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
    recorded_writes("effects.draw_rain_streak.turn_x87", &SPEC, |b| {
        with_state_at(b[0], |r, _| super::turn_x87(&values(b[1]), r));
    });
}

#[test]
fn project_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::RaceBlock {
                reads: &[
                    (0x224088, F32),
                    (0x22409C, F32),
                    (0x2240A0, F32),
                    (0x2241C8, F32),
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
            Op::Base {
                lo: 0x1574C5,
                hi: 0x1610BD,
                reads: &[
                    (0x158DE9, F32),
                    (0x158DED, F32),
                    (0x158DF5, F32),
                    (0x158DF9, F32),
                    (0x158E01, F32),
                    (0x158E05, F32),
                    (0x158E09, F32),
                    (0x158E0D, F32),
                    (0x158E11, F32),
                    (0x158E15, F32),
                    (0x158E21, F32),
                    (0x158EFD, F32),
                    (0x158F01, F32),
                    (0x158F05, F32),
                    (0x158F09, F32),
                    (0x158F0D, F32),
                    (0x158F11, F32),
                    (0x1610A5, F32),
                    (0x1610A9, F32),
                    (0x1610AD, F32),
                ],
                writes: &[
                    (0x158DE9, F32),
                    (0x158DED, F32),
                    (0x158DF5, F32),
                    (0x158DF9, F32),
                    (0x158E01, F32),
                    (0x158E05, F32),
                    (0x158E09, F32),
                    (0x158E0D, F32),
                    (0x158E11, F32),
                    (0x158E15, F32),
                    (0x158E21, F32),
                ],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_rain_streak.project_x87", &SPEC, |b| {
        with_state_at(b[2], |r, k| {
            staged(
                b[1],
                &[
                    0x0, 0x4, 0x8, 0x20, 0x24, 0x28, 0x40, 0x44, 0x48, 0x60, 0x64, 0x68,
                ],
                || {
                    super::project_x87(
                        crate::owners::Owners::the()
                            .scene
                            .scratch
                            .front_end_polygon(),
                        &crate::d3d::vertices_store::VERTICES,
                        r,
                        k,
                    );
                },
            );
        });
    });
}
