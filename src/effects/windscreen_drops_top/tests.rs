use crate::effects::asm_test::Kind::*;
use crate::effects::asm_test::with_settings;
use crate::effects::asm_test::{Op, Spec, recorded_writes};
use crate::effects::typed_sync::{overlay_laid, with_overlays};

#[test]
fn frame_axes_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x90,
                hi: 0xC9402,
                reads: &[
                    (0x90, I32),
                    (0xAFCDA, F32),
                    (0xAFCDE, F32),
                    (0xAFCE2, F32),
                    (0xC932A, F32),
                    (0xC932E, F32),
                    (0xC9332, F32),
                ],
                writes: &[
                    (0xAF7E8, F32),
                    (0xAFCC6, F32),
                    (0xAFCCE, F32),
                    (0xAFCD2, F32),
                    (0xAFCD6, F32),
                    (0xAFCDA, F32),
                    (0xAFCDE, F32),
                    (0xAFCE2, F32),
                    (0xB12EA, F32),
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
                    (0x2738, I32),
                ],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.windscreen_drops_top.frame_axes_x87", &SPEC, |b| {
        with_overlays(
            b[0],
            &[
                overlay_laid(&crate::test_support::own::effects().windscreen_drops.state),
                overlay_laid(&crate::test_support::own::effects().windscreen_drops.work),
                overlay_laid(&crate::test_support::own::effects().windscreen_drops.hex),
            ],
            || {
                with_settings(b[1], |m| {
                    let drops = &crate::test_support::own::effects().windscreen_drops;
                    super::frame_axes_x87(
                        &mut drops.hex.borrow_mut(),
                        &mut drops.work.borrow_mut(),
                        drops,
                        m,
                        &crate::effects::asm_layout::copy_of::<
                            std::cell::RefCell<crate::effects::setup_store::MarksClock>,
                        >(b[0])
                        .borrow(),
                    );
                });
            },
        );
    });
}

#[test]
fn frame_up_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0xAF832,
                hi: 0xC9402,
                reads: &[
                    (0xAFCE6, F32),
                    (0xAFCEA, F32),
                    (0xAFCEE, F32),
                    (0xC9336, F32),
                ],
                writes: &[
                    (0xAFCCA, F32),
                    (0xAFCE6, F32),
                    (0xAFCEA, F32),
                    (0xAFCEE, F32),
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
    recorded_writes("effects.windscreen_drops_top.frame_up_x87", &SPEC, |b| {
        with_overlays(
            b[0],
            &[
                overlay_laid(&crate::test_support::own::effects().windscreen_drops.work),
                overlay_laid(&crate::test_support::own::effects().windscreen_drops.hex),
            ],
            || {
                with_settings(b[1], |m| {
                    let drops = &crate::test_support::own::effects().windscreen_drops;
                    super::frame_up_x87(&drops.hex.borrow(), &mut drops.work.borrow_mut(), m);
                });
            },
        );
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
    recorded_writes("effects.windscreen_drops_top.frame_place_x87", &SPEC, |b| {
        with_overlays(
            b[0],
            &[overlay_laid(
                &crate::test_support::own::effects().windscreen_drops.hex,
            )],
            || {
                with_settings(b[1], |m| {
                    let drops = &crate::test_support::own::effects().windscreen_drops;
                    super::frame_place_x87(&mut drops.hex.borrow_mut(), m);
                });
            },
        );
    });
}

#[test]
fn frame_facing_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[Op::Base {
            lo: 0xAF7D0,
            hi: 0xC9402,
            reads: &[
                (0xAFCF2, F32),
                (0xAFCF6, F32),
                (0xAFCFA, F32),
                (0xB1262, F32),
                (0xB1266, F32),
                (0xB126A, F32),
            ],
            writes: &[(0xAF7D8, F32)],
        }],
        sincos: false,
    };
    recorded_writes(
        "effects.windscreen_drops_top.frame_facing_x87",
        &SPEC,
        |b| {
            with_overlays(
                b[0],
                &[
                    overlay_laid(&crate::test_support::own::effects().windscreen_drops.state),
                    overlay_laid(&crate::test_support::own::effects().windscreen_drops.work),
                    overlay_laid(&crate::test_support::own::effects().windscreen_drops.hex),
                ],
                || {
                    let drops = &crate::test_support::own::effects().windscreen_drops;
                    super::frame_facing_x87(&drops.hex.borrow(), &drops.work.borrow(), drops);
                },
            );
        },
    );
}

#[test]
fn point_turned_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0xAF832,
                hi: 0xB0250,
                reads: &[
                    (0xAFCDA, F32),
                    (0xAFCDE, F32),
                    (0xAFCE2, F32),
                    (0xAFCE6, F32),
                    (0xAFCEA, F32),
                    (0xAFCEE, F32),
                ],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0xC,
                reads: &[(0x0, F32), (0x4, F32)],
                writes: &[(0x0, F32), (0x4, F32), (0x8, F32)],
            },
        ],
        sincos: false,
    };
    recorded_writes(
        "effects.windscreen_drops_top.point_turned_x87",
        &SPEC,
        |b| {
            with_overlays(
                b[0],
                &[overlay_laid(
                    &crate::test_support::own::effects().windscreen_drops.work,
                )],
                || {
                    // The point's three floats, as the buffer holds them at b[1].
                    let host = |n: usize| (b[1] + 4 * n).at(0, 4).cast::<u32>();
                    let mut p: [f32; 3] = std::array::from_fn(|n| {
                        // SAFETY: `at` checks that the four bytes lie in the operand's test
                        // buffer, which the recorded run keeps alive and nothing else reaches
                        // meanwhile; unaligned, any bits a `u32`.
                        f32::from_bits(unsafe { host(n).read_unaligned() })
                    });
                    let axes = crate::test_support::own::effects()
                        .windscreen_drops
                        .work
                        .borrow()
                        .axes;
                    super::point_turned_x87(&axes, &mut p);
                    for (n, cell) in p.iter().enumerate() {
                        // SAFETY: `at` checks that the four bytes lie in the operand's test
                        // buffer, which the recorded run keeps alive and nothing else reaches
                        // meanwhile; unaligned, any bits a `u32`.
                        unsafe { host(n).write_unaligned(cell.to_bits()) };
                    }
                },
            );
        },
    );
}
