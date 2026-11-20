use crate::effects::asm_test::Kind::*;
use crate::effects::asm_test::with_settings;
use crate::effects::asm_test::{Op, Spec, recorded_writes};

use crate::effects::typed_sync::{overlay_laid, with_overlays};

/// Runs `f` with `HEX` holding the buffer's bytes (`typed_sync`).
fn hex<R>(mk: crate::effects::asm_test::Operand, f: impl FnOnce() -> R) -> R {
    with_overlays(
        mk,
        &[overlay_laid(
            &crate::test_support::own::effects().windscreen_drops.hex,
        )],
        f,
    )
}

/// The `N` floats at the test address `at`.
fn floats<const N: usize>(at: crate::effects::asm_test::Operand) -> [f32; N] {
    std::array::from_fn(|i| {
        // SAFETY: `at` checks that the four bytes lie in the operand's test buffer, which the
        // recorded run keeps alive and nothing else reaches meanwhile; unaligned, any bits a
        // `u32`.
        let word = unsafe { (at + 4 * i).at(0, 4).cast::<u32>().read_unaligned() };
        f32::from_bits(word)
    })
}

/// The floats written back to the test address `at`.
fn put_floats(at: crate::effects::asm_test::Operand, cells: &[f32]) {
    for (i, c) in cells.iter().enumerate() {
        // SAFETY: `at` checks that the four bytes lie in the operand's test buffer, which the
        // recorded run keeps alive and nothing else reaches meanwhile; unaligned, any bits a
        // `u32`.
        unsafe {
            (at + 4 * i)
                .at(0, 4)
                .cast::<u32>()
                .write_unaligned(c.to_bits());
        }
    }
}

/// The 30 bytes at the test address `at` as a buffer.
fn kept(at: crate::effects::asm_test::Operand) -> [u8; 30] {
    // SAFETY: `host` checks that the byte lies in the operand's test buffer, which the recorded
    // run keeps alive and nothing else reaches meanwhile.
    std::array::from_fn(|i| unsafe { (at + i).host().read() })
}

fn put_kept(at: crate::effects::asm_test::Operand, kept: &[u8; 30]) {
    for (i, &byte) in kept.iter().enumerate() {
        // SAFETY: `host` checks that the byte lies in the operand's test buffer, which the
        // recorded run keeps alive and nothing else reaches meanwhile.
        unsafe { (at + i).host().write(byte) };
    }
}

#[test]
fn view_moved_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0xA0,
                hi: 0xC9402,
                reads: &[
                    (0xA0, F32),
                    (0xB1296, F32),
                    (0xB129A, F32),
                    (0xB129E, F32),
                    (0xB12A2, F32),
                    (0xB12A6, F32),
                    (0xB12AA, F32),
                    (0xB12AE, F32),
                    (0xB12B2, F32),
                    (0xB12B6, F32),
                    (0xB12BA, F32),
                    (0xB12BE, F32),
                    (0xB12C2, F32),
                    (0xC93E6, F32),
                    (0xC93EA, F32),
                    (0xC93EE, F32),
                ],
                writes: &[
                    (0xB12AE, F32),
                    (0xB12B2, F32),
                    (0xB12B6, F32),
                    (0xB12BA, F32),
                    (0xB12BE, F32),
                    (0xB12C2, F32),
                ],
            },
            Op::Base {
                lo: 0,
                hi: 0x2768,
                reads: &[(0x26AC, F32), (0x26B0, F32), (0x26B4, F32)],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes(
        "effects.windscreen_drops_drift.view_moved_x87",
        &SPEC,
        |b| {
            hex(b[0], || {
                with_settings(b[1], |m| {
                    super::view_moved_x87(
                        &mut crate::test_support::own::effects()
                            .windscreen_drops
                            .hex
                            .borrow_mut(),
                        m,
                        &crate::effects::asm_layout::copy_of::<
                            std::cell::RefCell<crate::effects::setup_store::MarksClock>,
                        >(b[0])
                        .borrow(),
                    );
                });
            });
        },
    );
}

#[test]
fn drift_turned_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0xB1252,
                hi: 0xC9402,
                reads: &[(0xB12BA, F32), (0xB12BE, F32), (0xB12C2, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x24,
                reads: &[
                    (0x0, F32),
                    (0x4, F32),
                    (0x8, F32),
                    (0xC, F32),
                    (0x10, F32),
                    (0x14, F32),
                    (0x18, F32),
                    (0x1C, F32),
                    (0x20, F32),
                ],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x8,
                reads: &[(0x0, F32), (0x4, F32)],
                writes: &[(0x0, F32), (0x4, F32)],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x1E,
                reads: &[],
                writes: &[(0x0, F80), (0xA, F80), (0x14, F80)],
            },
        ],
        sincos: false,
    };
    recorded_writes(
        "effects.windscreen_drops_drift.drift_turned_x87",
        &SPEC,
        |b| {
            hex(b[0], || {
                let (m, mut temps, mut l) = (floats::<9>(b[1]), floats::<2>(b[2]), kept(b[3]));
                super::drift_turned_x87(
                    &crate::test_support::own::effects()
                        .windscreen_drops
                        .hex
                        .borrow(),
                    &m,
                    &mut temps,
                    &mut l,
                );
                put_floats(b[2], &temps);
                put_kept(b[3], &l);
            });
        },
    );
}

#[test]
fn drift_back_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0xACC83,
                hi: 0xC9402,
                reads: &[(0xC93F6, F32), (0xC93FA, F32), (0xC93FE, F32)],
                writes: &[
                    (0xACC9F, F32),
                    (0xACCA3, F32),
                    (0xACCA7, F32),
                    (0xB12DE, F32),
                    (0xB12E2, F32),
                    (0xB12E6, F32),
                ],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x24,
                reads: &[
                    (0x0, F32),
                    (0x4, F32),
                    (0x8, F32),
                    (0xC, F32),
                    (0x10, F32),
                    (0x14, F32),
                    (0x18, F32),
                    (0x1C, F32),
                    (0x20, F32),
                ],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x8,
                reads: &[(0x0, F32), (0x4, F32)],
                writes: &[(0x0, F32), (0x4, F32)],
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
    recorded_writes(
        "effects.windscreen_drops_drift.drift_back_x87",
        &SPEC,
        |b| {
            with_overlays(
                b[0],
                &[
                    overlay_laid(&crate::test_support::own::effects().windscreen_drops.tilt),
                    overlay_laid(&crate::test_support::own::effects().windscreen_drops.hex),
                ],
                || {
                    let (m, mut temps, l) = (floats::<9>(b[1]), floats::<2>(b[2]), kept(b[3]));
                    let drops = &crate::test_support::own::effects().windscreen_drops;
                    super::drift_back_x87(&mut drops.hex.borrow_mut(), drops, &m, &mut temps, &l);
                    put_floats(b[2], &temps);
                },
            );
        },
    );
}

#[test]
fn move_kept_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0xA0,
                hi: 0xC9402,
                reads: &[(0xA0, F32), (0xB1296, F32), (0xB129A, F32), (0xB129E, F32)],
                writes: &[(0xB12A2, F32), (0xB12A6, F32), (0xB12AA, F32)],
            },
            Op::Base {
                lo: 0,
                hi: 0x2768,
                reads: &[(0x26AC, F32), (0x26B0, F32), (0x26B4, F32)],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.windscreen_drops_drift.move_kept_x87", &SPEC, |b| {
        hex(b[0], || {
            with_settings(b[1], |m| {
                super::move_kept_x87(
                    &mut crate::test_support::own::effects()
                        .windscreen_drops
                        .hex
                        .borrow_mut(),
                    m,
                    &crate::effects::asm_layout::copy_of::<
                        std::cell::RefCell<crate::effects::setup_store::MarksClock>,
                    >(b[0])
                    .borrow(),
                );
            });
        });
    });
}
