use crate::effects::asm_test::Kind::*;
use crate::effects::asm_test::{Op, Spec, recorded_writes};
use crate::effects::typed_sync::{overlay_laid, with_overlays};

#[test]
fn axes_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[Op::Base {
            lo: 0xAF832,
            hi: 0xC9402,
            reads: &[
                (0xAFC82, F32),
                (0xAFCDA, F32),
                (0xAFCDE, F32),
                (0xAFCE2, F32),
                (0xAFCE6, F32),
                (0xAFCEA, F32),
                (0xAFCEE, F32),
                (0xAFD2E, F32),
                (0xAFD32, F32),
                (0xAFD36, F32),
                (0xAFD3A, F32),
                (0xAFD3E, F32),
                (0xAFD42, F32),
                (0xC9346, F32),
                (0xC934A, F32),
                (0xC934E, F32),
                (0xC9352, F32),
                (0xC9356, F32),
                (0xC935A, F32),
            ],
            writes: &[
                (0xAFD2E, F32),
                (0xAFD32, F32),
                (0xAFD36, F32),
                (0xAFD3A, F32),
                (0xAFD3E, F32),
                (0xAFD42, F32),
                (0xAFD46, F32),
                (0xAFD4A, F32),
                (0xAFD4E, F32),
                (0xAFD52, F32),
                (0xAFD56, F32),
                (0xAFD5A, F32),
            ],
        }],
        sincos: false,
    };
    recorded_writes("effects.windscreen_drops_setup.axes_x87", &SPEC, |b| {
        with_overlays(
            b[0],
            &[
                overlay_laid(&crate::test_support::own::effects().windscreen_drops.work),
                overlay_laid(&crate::test_support::own::effects().windscreen_drops.hex),
            ],
            || super::axes_x87(&crate::test_support::own::effects().windscreen_drops),
        );
    });
}
