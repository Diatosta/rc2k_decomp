use crate::effects::asm_layout::with_laid;
use std::cell::RefCell;

use crate::effects::asm_test::Kind::*;

use crate::effects::asm_test::{Op, Spec, recorded_writes};

use crate::effects::typed_sync::{overlay_laid, with_overlays};
use crate::effects::view_vectors_store::KeptView;
use crate::effects::windscreen_flakes_store::{WindscreenFlakeState, WindscreenFlakeTiming};

#[test]
fn rate_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::RaceBlock {
                reads: &[],
                writes: &[(0x31C814, Status)],
            },
            Op::Base {
                lo: 0x42F6,
                hi: 0x15732C,
                reads: &[
                    (0x42FE, F32),
                    (0xACC83, F32),
                    (0xACCA7, F32),
                    (0x137AF4, F32),
                    (0x137AF8, F32),
                    (0x157306, F32),
                    (0x15730A, F32),
                ],
                writes: &[(0x137AF8, F32)],
            },
        ],
        sincos: true,
    };
    recorded_writes("effects.windscreen_flakes_spawn.rate_x87", &SPEC, |b| {
        with_laid::<RefCell<KeptView>, _>(b[1], |kv| {
            with_laid::<RefCell<WindscreenFlakeTiming>, _>(b[1], |sti| {
                with_laid::<RefCell<WindscreenFlakeState>, _>(b[1], |ss| {
                    with_overlays(
                        b[1],
                        &[overlay_laid(
                            &crate::test_support::own::effects().windscreen_drops.tilt,
                        )],
                        || {
                            super::rate_x87(
                                &crate::test_support::own::effects().windscreen_drops,
                                &kv.borrow(),
                                &mut sti.borrow_mut(),
                                &ss.borrow(),
                                &crate::test_support::own::effects().setup.work,
                            );
                        },
                    );
                });
            });
        });
    });
}
