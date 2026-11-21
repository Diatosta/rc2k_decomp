use crate::effects::asm_layout::with_laid;
use crate::effects::asm_test::Kind::*;
use std::cell::RefCell;

use crate::effects::asm_test::with_settings;
use crate::effects::asm_test::{Op, Spec, recorded_writes};
use crate::effects::typed_sync::{overlay_laid, with_overlays};

use crate::effects::windscreen_flakes_store::{
    WindscreenFlakeState, WindscreenFlakeTables, WindscreenFlakeTiming,
};

#[test]
fn frame_axes_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x137AF0,
                hi: 0x157326,
                reads: &[
                    (0x137FDA, F32),
                    (0x137FDE, F32),
                    (0x137FE2, F32),
                    (0x15725E, F32),
                    (0x157262, F32),
                ],
                writes: &[
                    (0x137AF4, F32),
                    (0x137FAE, F32),
                    (0x137FB6, F32),
                    (0x137FBA, F32),
                    (0x137FBE, F32),
                    (0x137FDA, F32),
                    (0x137FDE, F32),
                    (0x137FE2, F32),
                ],
            },
            Op::Base {
                lo: 0,
                hi: 0x2768,
                reads: &[
                    (0x2720, I32),
                    (0x2724, I32),
                    (0x2728, I32),
                    (0x272C, I32),
                    (0x2730, I32),
                    (0x2734, I32),
                    (0x2740, I32),
                ],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes(
        "effects.windscreen_flakes_frame.frame_axes_x87",
        &SPEC,
        |b| {
            with_laid::<RefCell<WindscreenFlakeTiming>, _>(b[0], |sti| {
                with_laid::<std::cell::RefCell<WindscreenFlakeTables>, _>(b[0], |sq| {
                    with_laid::<std::cell::RefCell<WindscreenFlakeState>, _>(b[0], |ss| {
                        with_settings(b[1], |m| {
                            super::frame_axes_x87(
                                m,
                                &mut sti.borrow_mut(),
                                &mut sq.borrow_mut(),
                                &ss.borrow(),
                            );
                        });
                    });
                });
            });
        },
    );
}

#[test]
fn frame_up_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x137B8A,
                hi: 0x157326,
                reads: &[
                    (0x137FE6, F32),
                    (0x137FEA, F32),
                    (0x137FEE, F32),
                    (0x157266, F32),
                ],
                writes: &[
                    (0x137FB2, F32),
                    (0x137FE6, F32),
                    (0x137FEA, F32),
                    (0x137FEE, F32),
                ],
            },
            Op::Base {
                lo: 0,
                hi: 0x2768,
                reads: &[
                    (0x2708, I32),
                    (0x270C, I32),
                    (0x2710, I32),
                    (0x2720, I32),
                    (0x2724, I32),
                    (0x2728, I32),
                ],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.windscreen_flakes_frame.frame_up_x87", &SPEC, |b| {
        with_laid::<std::cell::RefCell<WindscreenFlakeTables>, _>(b[0], |sq| {
            with_laid::<std::cell::RefCell<WindscreenFlakeState>, _>(b[0], |ss| {
                with_settings(b[1], |m| {
                    super::frame_up_x87(m, &mut sq.borrow_mut(), &ss.borrow());
                });
            });
        });
    });
}

#[test]
fn frame_place_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0xB1252,
                hi: 0xC9402,
                reads: &[],
                writes: &[(0xB1262, F32), (0xB1266, F32), (0xB126A, F32)],
            },
            Op::Base {
                lo: 0,
                hi: 0x2768,
                reads: &[(0x2720, I32), (0x2724, I32), (0x2728, I32)],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes(
        "effects.windscreen_flakes_frame.frame_place_x87",
        &SPEC,
        |b| {
            with_overlays(
                b[0],
                &[overlay_laid(
                    &crate::test_support::own::effects().windscreen_drops.hex,
                )],
                || {
                    with_settings(b[1], |m| {
                        super::frame_place_x87(
                            &crate::test_support::own::effects().windscreen_drops,
                            m,
                        );
                    });
                },
            );
        },
    );
}

#[test]
fn frame_facing_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[Op::Base {
            lo: 0xAF7D0,
            hi: 0x13822A,
            reads: &[
                (0xB1262, F32),
                (0xB1266, F32),
                (0xB126A, F32),
                (0x137FF2, F32),
                (0x137FF6, F32),
                (0x137FFA, F32),
            ],
            writes: &[(0xAF7D8, F32)],
        }],
        sincos: false,
    };
    recorded_writes(
        "effects.windscreen_flakes_frame.frame_facing_x87",
        &SPEC,
        |b| {
            with_overlays(
                b[0],
                &[
                    overlay_laid(&crate::test_support::own::effects().windscreen_drops.state),
                    overlay_laid(&crate::test_support::own::effects().windscreen_drops.hex),
                ],
                || {
                    with_laid::<std::cell::RefCell<WindscreenFlakeTables>, _>(b[0], |m| {
                        super::frame_facing_x87(
                            &crate::test_support::own::effects().windscreen_drops,
                            &mut m.borrow_mut(),
                        );
                    });
                },
            );
        },
    );
}
