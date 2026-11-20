use crate::effects::asm_layout::with_laid;
use crate::effects::asm_test::Kind::*;

use crate::effects::asm_test::{Op, Spec, recorded_writes};
use crate::effects::snow_store::SnowLight;
use crate::test_support::own::race;

#[test]
fn turn_light_x87_matches() {
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
                    (0x1EA3CC, F32),
                    (0x1EA3D0, F32),
                    (0x1EA3D4, F32),
                    (0x1EA3DC, F32),
                    (0x1EA3E0, F32),
                    (0x1EA3E4, F32),
                    (0x1EA3EC, F32),
                    (0x1EA3F0, F32),
                    (0x1EA3F4, F32),
                    (0x1EA3FC, F32),
                    (0x1EA400, F32),
                    (0x1EA404, F32),
                ],
                writes: &[],
            },
            Op::Base {
                lo: 0x1764F0,
                hi: 0x176618,
                reads: &[
                    (0x1765CC, F32),
                    (0x1765D0, F32),
                    (0x1765D4, F32),
                    (0x1765D8, F32),
                    (0x1765DC, F32),
                    (0x1765E0, F32),
                ],
                writes: &[
                    (0x1764F0, F32),
                    (0x1764F4, F32),
                    (0x1764F8, F32),
                    (0x1764FC, F32),
                    (0x176500, F32),
                    (0x176504, F32),
                ],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.run_snow.turn_light_x87", &SPEC, |b| {
        with_laid::<SnowLight, _>(b[1], |l| {
            super::turn_light_x87(&race().frame, &race().frame.matrices.get().view_matrix, l);
        });
    });
}
