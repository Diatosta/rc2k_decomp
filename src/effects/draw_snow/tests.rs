use crate::effects::asm_layout::with_laid;
use crate::effects::asm_test::Kind::*;

use crate::effects::asm_test::{Op, Spec, recorded_writes};
use crate::effects::snow_store::{SnowLight, SnowState, SnowTables};
use crate::effects::tuning_store::MarksFloats;

#[test]
fn turn_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x16D0C0,
                hi: 0x16D3D0,
                reads: &[
                    (0x16D2C8, I32),
                    (0x16D2CC, I32),
                    (0x16D2D0, I32),
                    (0x16D2E0, F32),
                    (0x16D2E4, F32),
                    (0x16D2E8, F32),
                    (0x16D2EC, F32),
                    (0x16D2F0, F32),
                    (0x16D2F4, F32),
                    (0x16D2F8, F32),
                    (0x16D2FC, F32),
                    (0x16D300, F32),
                ],
                writes: &[
                    (0x16D2B4, I32),
                    (0x16D394, F32),
                    (0x16D398, F32),
                    (0x16D39C, F32),
                ],
            },
            Op::Base {
                lo: 0x0,
                hi: 0xC,
                reads: &[(0x0, I32), (0x4, I32), (0x8, I32)],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_snow.turn_x87", &SPEC, |b| {
        with_laid::<SnowState, _>(b[0], |s| super::turn_x87(b[1], s));
    });
}

#[test]
fn fade_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[Op::Base {
            lo: 0x200,
            hi: 0x176618,
            reads: &[
                (0x22C, F32),
                (0x24C, F32),
                (0x16D394, F32),
                (0x16D398, F32),
                (0x16D39C, F32),
                (0x1764F0, F32),
                (0x1764F4, F32),
                (0x1764F8, F32),
                (0x1764FC, F32),
                (0x176500, F32),
                (0x176504, F32),
                (0x176508, F32),
                (0x17650C, F32),
                (0x176510, F32),
            ],
            writes: &[
                (0x176508, F32),
                (0x17650C, F32),
                (0x176510, F32),
                (0x176514, F32),
                (0x176518, F32),
                (0x17651C, F32),
                (0x176520, I32),
                (0x176588, F32),
            ],
        }],
        sincos: false,
    };
    recorded_writes("effects.draw_snow.fade_x87", &SPEC, |b| {
        with_laid::<SnowState, _>(b[0], |s| {
            with_laid::<SnowLight, _>(b[0], |l| {
                with_laid::<MarksFloats, _>(b[0], |f| super::fade_x87(s, l, f));
            });
        });
    });
}

#[test]
fn square_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::RaceBlock {
                reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x7C,
                reads: &[(0x0, F32), (0x4, F32)],
                writes: &[
                    (0x0, F32),
                    (0x4, F32),
                    (0x8, F32),
                    (0x18, F32),
                    (0x20, F32),
                    (0x24, F32),
                    (0x28, F32),
                    (0x38, F32),
                    (0x40, F32),
                    (0x44, F32),
                    (0x48, F32),
                    (0x58, F32),
                    (0x60, F32),
                    (0x64, F32),
                    (0x68, F32),
                    (0x78, F32),
                ],
            },
            Op::Base {
                lo: 0x1610C0,
                hi: 0x176618,
                reads: &[
                    (0x16B0C0, F32),
                    (0x16B0C4, F32),
                    (0x16B0C8, F32),
                    (0x16B250, F32),
                    (0x16D2B4, F32),
                    (0x16D2B8, F32),
                    (0x16D394, F32),
                    (0x16D398, F32),
                    (0x16D39C, F32),
                    (0x16F0BC, F32),
                    (0x176608, F32),
                    (0x17660C, F32),
                ],
                writes: &[(0x16D2B4, F32)],
            },
            Op::Index(&[0, 4, 8, 400, 8188]),
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_snow.square_x87", &SPEC, |b| {
        with_laid::<SnowTables, _>(b[2], |t| {
            with_laid::<SnowState, _>(b[2], |s| {
                with_laid::<SnowLight, _>(b[2], |l| {
                    super::square_x87(
                        crate::owners::Owners::the()
                            .scene
                            .scratch
                            .front_end_polygon(),
                        b[1],
                        b[3].number(),
                        t,
                        s,
                        l,
                    );
                });
            });
        });
    });
}
