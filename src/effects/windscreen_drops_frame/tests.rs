use crate::memory::session::ByteBuffer;
use crate::test_support::own::effects;

use crate::effects::typed_sync::{overlay_laid, with_overlays};

/// Runs `f` with the windscreen drops' statics holding the buffer's bytes (`typed_sync`).
fn windscreen_drops<R>(mk: crate::effects::asm_test::Operand, f: impl FnOnce() -> R) -> R {
    with_overlays(
        mk,
        &[
            overlay_laid(&crate::test_support::own::effects().windscreen_drops.tilt),
            overlay_laid(&crate::test_support::own::effects().windscreen_drops.state),
            overlay_laid(&crate::test_support::own::effects().windscreen_drops.work),
        ],
        f,
    )
}
use crate::effects::asm_test::Kind::*;
use crate::effects::asm_test::{Op, Spec, canon, recorded_writes};

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

/// The test buffer's `len` bytes at `at` run through `f` as `bytes` (a stand-in for the
/// item or record found by address): `bytes` holds them from offset 0 and
/// what `f` leaves goes back to the buffer, the stand-in as it was.
fn staged<R>(
    at: crate::effects::asm_test::Operand,
    from: usize,
    len: usize,
    get: impl Fn(usize) -> u8,
    put: impl Fn(usize, u8),
    f: impl FnOnce() -> R,
) -> R {
    let host = (at + from).at(0, len);
    let saved: Vec<u8> = (from..from + len).map(&get).collect();
    for i in 0..len {
        // SAFETY: byte `i < len` of the `len` that `at` checked lie in the operand's test
        // buffer, which the recorded run keeps alive and nothing else reaches meanwhile.
        let b = unsafe { host.add(i).read() };
        put(from + i, b);
    }
    let out = f();
    for (i, &b) in saved.iter().enumerate() {
        // SAFETY: byte `i < len` of the `len` that `at` checked lie in the operand's test
        // buffer, which the recorded run keeps alive and nothing else reaches meanwhile.
        unsafe { host.add(i).write(get(from + i)) };
        put(from + i, b);
    }
    out
}

#[test]
fn drift_cell_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0xACC83,
                hi: 0xB0250,
                reads: &[
                    (0xACC9F, F32),
                    (0xACCA3, F32),
                    (0xACCA7, F32),
                    (0xAF882, F32),
                    (0xAF886, F32),
                    (0xAF88E, F32),
                    (0xAF8E6, F32),
                    (0xAFA82, F32),
                    (0xAFC7E, F32),
                ],
                writes: &[(0xAF832, F32), (0xAF836, F32)],
            },
            Op::Index(&[0, 4, 12, 100, 512, 1020]),
        ],
        sincos: false,
    };
    recorded_writes(
        "effects.windscreen_drops_frame.drift_cell_x87",
        &SPEC,
        |b| {
            windscreen_drops(b[0], || {
                super::drift_cell_x87(
                    &crate::test_support::own::effects().windscreen_drops,
                    b[1].number(),
                );
            });
        },
    );
}

#[test]
fn drift_move_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0xACC83,
                hi: 0xC9402,
                reads: &[
                    (0xAF832, F32),
                    (0xAF836, F32),
                    (0xAFC82, F32),
                    (0xAFC86, F32),
                    (0xAFC8E, F32),
                    (0xAFCE6, F32),
                    (0xAFE82, F32),
                    (0xB007E, F32),
                    (0xB12EA, F32),
                ],
                writes: &[],
            },
            Op::Index(&[0, 4, 12, 28]),
            Op::Base {
                lo: 0x14,
                hi: 0x24,
                reads: &[(0x14, F32), (0x18, F32), (0x1C, F32), (0x20, F32)],
                writes: &[(0x14, F32), (0x18, F32), (0x1C, F32), (0x20, F32)],
            },
        ],
        sincos: false,
    };
    recorded_writes(
        "effects.windscreen_drops_frame.drift_move_x87",
        &SPEC,
        |b| {
            windscreen_drops(b[0], || {
                hex(b[0], || {
                    staged(
                        b[2],
                        0x14,
                        0x10,
                        |i| {
                            crate::test_support::own::effects()
                                .windscreen_drops
                                .hex
                                .borrow()
                                .items[i]
                        },
                        |i, v| {
                            crate::test_support::own::effects()
                                .windscreen_drops
                                .hex
                                .borrow_mut()
                                .items[i] = v;
                        },
                        || {
                            super::drift_move_x87(
                                &crate::test_support::own::effects().windscreen_drops,
                                b[1].number(),
                                0,
                            );
                        },
                    );
                });
            });
        },
    );
}

#[test]
fn sprite_depth_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[Op::Base {
            lo: 0xACC83,
            hi: 0xC9402,
            reads: &[
                (0xAFCC6, F32),
                (0xAFCCA, F32),
                (0xAFCCE, F32),
                (0xAFCD2, F32),
                (0xAFCD6, F32),
                (0xAFCDA, F32),
                (0xAFCDE, F32),
                (0xAFCE2, F32),
                (0xAFCE6, F32),
                (0xAFCEA, F32),
                (0xAFCEE, F32),
                (0xB0242, I32),
                (0xB0246, I32),
                (0xC9366, F32),
            ],
            writes: &[
                (0xAF7F4, F32),
                (0xAFD6A, F32),
                (0xAFD6E, F32),
                (0xAFD72, F32),
            ],
        }],
        sincos: false,
    };
    recorded_writes(
        "effects.windscreen_drops_frame.sprite_depth_x87",
        &SPEC,
        |b| {
            windscreen_drops(b[0], || {
                hex(b[0], || {
                    super::sprite_depth_x87(&crate::test_support::own::effects().windscreen_drops);
                });
            });
        },
    );
}

#[test]
fn sprite_corners_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::RaceBlock {
                reads: &[(0x224088, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0xACC83,
                hi: 0xB0250,
                reads: &[
                    (0xAFD6A, F32),
                    (0xAFD6E, F32),
                    (0xAFD72, F32),
                    (0xAFDBE, F32),
                    (0xAFDC2, F32),
                    (0xAFDC6, F32),
                    (0xAFDCA, F32),
                    (0xAFDCE, F32),
                    (0xAFDD2, F32),
                    (0xAFDD6, F32),
                    (0xAFDDA, F32),
                    (0xAFDDE, F32),
                    (0xAFDE2, F32),
                    (0xAFDE6, F32),
                    (0xAFDEA, F32),
                    (0xAFDEE, F32),
                    (0xAFDF2, F32),
                    (0xAFDF6, F32),
                    (0xAFDFA, F32),
                    (0xAFDFE, F32),
                    (0xAFE02, F32),
                    (0xAFE06, F32),
                    (0xAFE0A, F32),
                    (0xAFE0E, F32),
                    (0xAFE12, F32),
                    (0xAFE16, F32),
                    (0xAFE1A, F32),
                    (0xAFE1E, F32),
                    (0xAFE22, F32),
                    (0xAFE26, F32),
                    (0xAFE2A, F32),
                    (0xAFE2E, F32),
                    (0xAFE32, F32),
                    (0xAFE36, F32),
                    (0xAFE3A, F32),
                    (0xAFE3E, F32),
                    (0xAFE42, F32),
                    (0xAFE46, F32),
                    (0xAFE4A, F32),
                    (0xAFE4E, F32),
                    (0xAFE52, F32),
                    (0xAFE56, F32),
                    (0xAFE5A, F32),
                    (0xAFE5E, F32),
                    (0xAFE62, F32),
                    (0xAFE66, F32),
                    (0xAFE6A, F32),
                    (0xAFE6E, F32),
                    (0xAFE72, F32),
                    (0xAFE76, F32),
                    (0xAFE7A, F32),
                    (0xAFE7E, F32),
                    (0xAFE82, F32),
                    (0xAFE86, F32),
                    (0xAFE8A, F32),
                    (0xAFE8E, F32),
                    (0xAFE92, F32),
                    (0xAFFB6, F32),
                    (0xAFFBA, F32),
                    (0xAFFBE, F32),
                    (0xAFFC2, F32),
                    (0xAFFC6, F32),
                    (0xAFFCA, F32),
                    (0xAFFCE, F32),
                    (0xAFFD2, F32),
                    (0xAFFD6, F32),
                    (0xAFFDA, F32),
                    (0xAFFDE, F32),
                    (0xAFFE2, F32),
                    (0xAFFE6, F32),
                    (0xAFFEA, F32),
                    (0xAFFEE, F32),
                    (0xAFFF2, F32),
                    (0xAFFF6, F32),
                    (0xAFFFA, F32),
                ],
                writes: &[
                    (0xAFD76, F32),
                    (0xAFD7A, F32),
                    (0xAFD7E, F32),
                    (0xAFD82, F32),
                    (0xAFD86, F32),
                    (0xAFD8A, F32),
                    (0xAFD8E, F32),
                    (0xAFD92, F32),
                    (0xAFD96, F32),
                    (0xAFD9A, F32),
                    (0xAFD9E, F32),
                    (0xAFDA2, F32),
                    (0xAFDA6, F32),
                    (0xAFDAA, F32),
                    (0xAFDAE, F32),
                    (0xAFDB2, F32),
                    (0xAFDB6, F32),
                    (0xAFDBA, F32),
                ],
            },
            Op::Index(&[0, 72, 144, 504]),
        ],
        sincos: false,
    };
    recorded_writes(
        "effects.windscreen_drops_frame.sprite_corners_x87",
        &SPEC,
        |b| {
            canon(
                windscreen_drops(b[1], || {
                    super::sprite_corners_x87(
                        crate::owners::Owners::the()
                            .scene
                            .scratch
                            .front_end_polygon(),
                        &crate::test_support::own::effects().windscreen_drops,
                        b[2].number(),
                    )
                })
                .0,
            )
        },
    );
}

#[test]
fn sprite_project_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::RaceBlock {
                reads: &[(0x22409C, F32), (0x2240A0, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x2F8,
                hi: 0xC9402,
                reads: &[
                    (0x35C, F32),
                    (0xAF7F4, F32),
                    (0xAFCC2, F32),
                    (0xAFD5E, F32),
                    (0xAFD62, F32),
                    (0xAFD6A, F32),
                    (0xAFD6E, F32),
                    (0xAFD72, F32),
                    (0xAFD76, F32),
                    (0xAFD7A, F32),
                    (0xAFD82, F32),
                    (0xAFD86, F32),
                    (0xAFD8E, F32),
                    (0xAFD92, F32),
                    (0xAFD9A, F32),
                    (0xAFD9E, F32),
                    (0xAFDA6, F32),
                    (0xAFDAA, F32),
                    (0xAFDB2, F32),
                    (0xAFDB6, F32),
                    (0xB1252, F32),
                    (0xB1256, F32),
                    (0xB125A, F32),
                    (0xB125E, F32),
                ],
                writes: &[
                    (0xAFD5E, F32),
                    (0xAFD62, F32),
                    (0xAFD6A, F32),
                    (0xAFD6E, F32),
                    (0xAFD72, F32),
                ],
            },
            Op::Base {
                lo: 0x0,
                hi: 0xC0,
                reads: &[
                    (0x18, F32),
                    (0x1C, F32),
                    (0x38, F32),
                    (0x3C, F32),
                    (0x58, F32),
                    (0x5C, F32),
                    (0x78, F32),
                    (0x7C, F32),
                    (0x98, F32),
                    (0x9C, F32),
                    (0xB8, F32),
                    (0xBC, F32),
                ],
                writes: &[
                    (0x0, F32),
                    (0x4, F32),
                    (0x8, F32),
                    (0x18, F32),
                    (0x1C, F32),
                    (0x20, F32),
                    (0x24, F32),
                    (0x28, F32),
                    (0x38, F32),
                    (0x3C, F32),
                    (0x40, F32),
                    (0x44, F32),
                    (0x48, F32),
                    (0x58, F32),
                    (0x5C, F32),
                    (0x60, F32),
                    (0x64, F32),
                    (0x68, F32),
                    (0x78, F32),
                    (0x7C, F32),
                    (0x80, F32),
                    (0x84, F32),
                    (0x88, F32),
                    (0x98, F32),
                    (0x9C, F32),
                    (0xA0, F32),
                    (0xA4, F32),
                    (0xA8, F32),
                    (0xB8, F32),
                    (0xBC, F32),
                ],
            },
            Op::Float,
        ],
        sincos: false,
    };
    recorded_writes(
        "effects.windscreen_drops_frame.sprite_project_x87",
        &SPEC,
        |b| {
            windscreen_drops(b[1], || {
                with_overlays(b[1], &[overlay_laid(&effects().setup.kind_floats)], || {
                    hex(b[1], || {
                        staged(
                            b[2],
                            0,
                            0xC0,
                            |i| {
                                crate::test_support::own::gfx()
                                    .card
                                    .windscreen_drop_batch
                                    .records
                                    .get_byte(i)
                            },
                            |i, v| {
                                crate::test_support::own::gfx()
                                    .card
                                    .windscreen_drop_batch
                                    .records
                                    .write_bytes_at(i, &[v], "a test byte")
                                    .unwrap();
                            },
                            || {
                                super::sprite_project_x87(
                                    crate::owners::Owners::the()
                                        .scene
                                        .scratch
                                        .front_end_polygon(),
                                    &crate::test_support::own::effects().windscreen_drops,
                                    crate::test_support::own::gfx(),
                                    crate::test_support::own::effects(),
                                    0,
                                    super::bh::ld(b[3].number() as u32),
                                );
                            },
                        );
                    });
                });
            });
        },
    );
}
