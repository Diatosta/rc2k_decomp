use crate::effects::asm_layout::with_laid;
use crate::effects::asm_test::Kind::*;
use crate::effects::asm_test::staged_vertices as staged;

use crate::effects::asm_test::{Op, Spec, recorded_writes};
use crate::effects::frame_state_store::{DrawWork, ViewVars};

#[test]
fn ribbon_x87_matches() {
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
                hi: 0x18EA8D,
                reads: &[
                    (0x186975, I32),
                    (0x18698D, F32),
                    (0x186991, F32),
                    (0x186995, F32),
                    (0x186999, F32),
                    (0x18699D, F32),
                    (0x1869A1, F32),
                    (0x1869A5, F32),
                    (0x1869A9, F32),
                    (0x186A2D, F32),
                    (0x18EA45, F32),
                    (0x18EA49, F32),
                    (0x18EA51, F32),
                    (0x18EA55, F32),
                    (0x18EA5D, F32),
                    (0x18EA61, F32),
                ],
                writes: &[
                    (0x18698D, F32),
                    (0x186991, F32),
                    (0x186995, F32),
                    (0x186999, F32),
                    (0x1869A1, F32),
                    (0x1869A5, F32),
                    (0x18EA45, F32),
                    (0x18EA49, F32),
                    (0x18EA51, F32),
                    (0x18EA55, F32),
                    (0x18EA5D, F32),
                    (0x18EA61, F32),
                ],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_ribbon.ribbon_x87", &SPEC, |b| {
        with_laid::<DrawWork, _>(b[2], |dw| {
            with_laid::<ViewVars, _>(b[2], |vv| {
                staged(
                    b[1],
                    &[
                        0x0, 0x4, 0x8, 0x20, 0x24, 0x28, 0x40, 0x44, 0x48, 0x60, 0x64, 0x68,
                    ],
                    || {
                        super::ribbon_x87(
                            crate::owners::Owners::the()
                                .scene
                                .scratch
                                .front_end_polygon(),
                            &crate::d3d::vertices_store::VERTICES,
                            dw,
                            vv,
                        );
                    },
                );
            });
        });
    });
}
