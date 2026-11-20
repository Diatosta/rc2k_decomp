use crate::effects::asm_layout::with_laid;
use crate::effects::asm_test::Kind::*;

use crate::effects::asm_test::{Op, Spec, recorded_buffer_writes};
use crate::effects::windscreen_flakes_store::{WindscreenFlakeState, WindscreenFlakeTables};

#[test]
fn scale_axes_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[Op::Base {
            lo: 0x137B8A,
            hi: 0x157326,
            reads: &[
                (0x137F8A, F32),
                (0x137FAE, F32),
                (0x137FB2, F32),
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
                (0x15729E, F32),
                (0x1572A2, F32),
                (0x1572A6, F32),
                (0x1572AA, F32),
                (0x1572AE, F32),
                (0x1572B2, F32),
                (0x1572B6, F32),
                (0x1572BA, F32),
                (0x1572BE, F32),
                (0x1572C2, F32),
                (0x1572C6, F32),
                (0x1572CA, F32),
            ],
            writes: &[
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
                (0x13802E, F32),
                (0x138032, F32),
                (0x138036, F32),
                (0x13803A, F32),
                (0x13803E, F32),
                (0x138042, F32),
            ],
        }],
        sincos: false,
    };
    recorded_buffer_writes("effects.scale_axes.scale_axes_x87", &SPEC, |b| {
        with_laid::<WindscreenFlakeTables, _>(b[0], |sq| {
            with_laid::<WindscreenFlakeState, _>(b[0], |ss| {
                super::scale_axes_x87(sq, ss);
            });
        });
    });
}
