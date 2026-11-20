use crate::effects::asm_layout::with_laid;
use crate::effects::asm_test::Kind::*;
use crate::effects::asm_test::{Op, Spec, recorded_writes};
use crate::effects::ring_store::RingWork;
use crate::effects::tuning_store::MarksFloats;
use crate::race::effects_settings_store::CarEffects;

fn word(at: crate::effects::asm_test::Operand) -> u32 {
    // SAFETY: `at` checks that the four bytes lie in the operand's test buffer, which the
    // recorded run keeps alive and nothing else reaches meanwhile; unaligned, any bits a `u32`.
    unsafe { at.at(0, 4).cast::<u32>().read_unaligned() }
}

/// A car record whose words at the record `offsets` hold the test buffer's dwords at `at +
/// offset - base`.
fn car_with(at: crate::effects::asm_test::Operand, base: usize, offsets: &[usize]) -> CarEffects {
    let car = CarEffects::zeroed();
    for &o in offsets {
        crate::memory::cells::store_value(&car, o, word(at + o - base)).unwrap();
    }
    car
}

/// The car's 3 x 3 matrix (the record's inverse matrix) from the buffer at `at`.
fn car_matrix(at: crate::effects::asm_test::Operand) -> CarEffects {
    let offsets: Vec<usize> = [0x0, 0x4, 0x8, 0x10, 0x14, 0x18, 0x20, 0x24, 0x28]
        .iter()
        .map(|o| 0x94 + o)
        .collect();
    car_with(at, 0x94, &offsets)
}

/// A whole matrix whose translation (the last three of its 16 floats) is the buffer's at `at` + 0x30
/// (`at` the matrix's place).
fn whole_at(at: crate::effects::asm_test::Operand) -> [crate::memory::session::Float; 16] {
    std::array::from_fn(|i| {
        crate::memory::session::Float::from_bits(if (12..15).contains(&i) {
            word(at + 4 * i)
        } else {
            0
        })
    })
}

#[test]
fn box_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x1D06E0,
                hi: 0x1D0826,
                reads: &[(0x1D07D8, F32), (0x1D07DC, F32), (0x1D07E0, F32)],
                writes: &[
                    (0x1D0740, I32),
                    (0x1D0744, I32),
                    (0x1D07D8, F32),
                    (0x1D07DC, F32),
                    (0x1D07E0, F32),
                ],
            },
            Op::Base {
                lo: 0x10,
                hi: 0x1C,
                reads: &[(0x10, F32), (0x14, F32), (0x18, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x30,
                hi: 0x3C,
                reads: &[(0x30, F32), (0x34, F32), (0x38, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x2C,
                reads: &[
                    (0x0, F32),
                    (0x4, F32),
                    (0x8, F32),
                    (0x10, F32),
                    (0x14, F32),
                    (0x18, F32),
                    (0x20, F32),
                    (0x24, F32),
                    (0x28, F32),
                ],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.run_ring.box_x87", &SPEC, |b| {
        with_laid::<std::cell::RefCell<RingWork>, _>(b[0], |w| {
            crate::effects::ring_entries::with_entry_copy_at(b[1], 0x10, 0x1C, |e| {
                super::box_x87(e, &mut w.borrow_mut(), &whole_at(b[2]), &car_matrix(b[3]));
            });
        });
    });
}

#[test]
fn facing_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x200,
                hi: 0x1D0826,
                reads: &[
                    (0x22C, F32),
                    (0x1D07D8, F32),
                    (0x1D07DC, F32),
                    (0x1D07E0, F32),
                ],
                writes: &[
                    (0x1D0774, F32),
                    (0x1D07D8, F32),
                    (0x1D07DC, F32),
                    (0x1D07E0, F32),
                ],
            },
            Op::Base {
                lo: 0x4C,
                hi: 0x58,
                reads: &[(0x4C, F32), (0x50, F32), (0x54, F32)],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.run_ring.facing_x87", &SPEC, |b| {
        with_laid::<std::cell::RefCell<MarksFloats>, _>(b[0], |f| {
            with_laid::<std::cell::RefCell<RingWork>, _>(b[0], |w| {
                let car = car_with(b[1], 0, &[0x4C, 0x50, 0x54]);
                super::facing_x87(&car, &f.borrow(), &mut w.borrow_mut());
            });
        });
    });
}

#[test]
fn pull_length_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x1D06E0,
                hi: 0x1D07FC,
                reads: &[(0x1D07D8, F32), (0x1D07DC, F32), (0x1D07E0, F32)],
                writes: &[
                    (0x1D077C, F32),
                    (0x1D07D8, F32),
                    (0x1D07DC, F32),
                    (0x1D07E0, F32),
                ],
            },
            Op::Base {
                lo: 0x10,
                hi: 0x1C,
                reads: &[(0x10, F32), (0x14, F32), (0x18, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x30,
                hi: 0x3C,
                reads: &[(0x30, F32), (0x34, F32), (0x38, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0xA,
                reads: &[],
                writes: &[(0x0, F80)],
            },
            Op::Base {
                lo: 0x4C,
                hi: 0x50,
                reads: &[(0x4C, F32)],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.run_ring.pull_length_x87", &SPEC, |b| {
        with_laid::<std::cell::RefCell<RingWork>, _>(b[0], |w| {
            crate::effects::ring_entries::with_entry_copy_at(b[1], 0x10, 0x1C, |e| {
                let mut length = [0xAA; 10];
                let kinds = super::RingKinds {
                    near_less: f32::from_bits(word(b[4] + 0x4C)),
                    ..crate::test_support::own::effects().ring.kinds
                };
                super::pull_length_x87(
                    e,
                    &whole_at(b[2]),
                    &mut length,
                    &mut w.borrow_mut(),
                    &kinds,
                );
                // SAFETY: `at` checks that the ten bytes lie in the operand's test buffer,
                // which the recorded run keeps alive; `length` is this closure's own.
                unsafe {
                    b[3].at(0, 10).copy_from_nonoverlapping(length.as_ptr(), 10);
                }
            });
        });
    });
}

#[test]
fn pull_facing_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x200,
                hi: 0x1D0826,
                reads: &[
                    (0x22C, F32),
                    (0x1D07D8, F32),
                    (0x1D07DC, F32),
                    (0x1D07E0, F32),
                    (0x1D07E4, F32),
                    (0x1D07E8, F32),
                    (0x1D07EC, F32),
                ],
                writes: &[
                    (0x1D0774, F32),
                    (0x1D07D8, F32),
                    (0x1D07DC, F32),
                    (0x1D07E0, F32),
                    (0x1D07E4, F32),
                    (0x1D07E8, F32),
                    (0x1D07EC, F32),
                ],
            },
            Op::Base {
                lo: 0x4C,
                hi: 0x8C,
                reads: &[(0x4C, F32), (0x50, F32), (0x54, F32), (0x88, I32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0xA,
                reads: &[(0x0, F80)],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.run_ring.pull_facing_x87", &SPEC, |b| {
        with_laid::<std::cell::RefCell<MarksFloats>, _>(b[0], |f| {
            with_laid::<std::cell::RefCell<RingWork>, _>(b[0], |w| {
                let car = car_with(b[1], 0, &[0x4C, 0x50, 0x54, 0x88]);
                let length: [u8; 10] =
                    // SAFETY: `host` checks that the byte lies in the operand's test buffer,
                    // which the recorded run keeps alive and nothing else reaches meanwhile.
                    std::array::from_fn(|i| unsafe { (b[2] + i).host().read() });
                super::pull_facing_x87(&car, &length, &f.borrow(), &mut w.borrow_mut());
            });
        });
    });
}

#[test]
fn pull_strength_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x90,
                hi: 0x1D07FC,
                reads: &[(0x90, I32), (0x22C, F32), (0x29C, F32), (0x1D0780, F32)],
                writes: &[(0x1D0780, F32), (0x1D0780, I32)],
            },
            Op::Base {
                lo: 0x64,
                hi: 0x68,
                reads: &[(0x64, F32)],
                writes: &[(0x64, F32)],
            },
            Op::Base {
                lo: 0x88,
                hi: 0x8C,
                reads: &[(0x88, I32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x50,
                hi: 0x58,
                reads: &[(0x50, F32), (0x54, F32)],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.run_ring.pull_strength_x87", &SPEC, |b| {
        with_laid::<std::cell::RefCell<MarksFloats>, _>(b[0], |f| {
            with_laid::<std::cell::RefCell<RingWork>, _>(b[0], |w| {
                crate::effects::ring_entries::with_entry_copy_at(b[1], 0x64, 0x68, |e| {
                    let car = car_with(b[2], 0, &[0x88]);
                    let kinds = super::RingKinds {
                        near_scale: f32::from_bits(word(b[3] + 0x50)),
                        near_rate: f32::from_bits(word(b[3] + 0x54)),
                        ..crate::test_support::own::effects().ring.kinds
                    };
                    super::pull_strength_x87(
                        e,
                        &car,
                        &crate::effects::asm_layout::copy_of::<
                            std::cell::RefCell<crate::effects::setup_store::MarksClock>,
                        >(b[0])
                        .borrow(),
                        &f.borrow(),
                        &mut w.borrow_mut(),
                        &kinds,
                    );
                });
            });
        });
    });
}

#[test]
fn box_place_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x1D06E0,
                hi: 0x1D0826,
                reads: &[],
                writes: &[(0x1D0740, I32), (0x1D0744, I32)],
            },
            Op::Base {
                lo: 0x10,
                hi: 0x1C,
                reads: &[(0x10, F32), (0x14, F32), (0x18, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x2C,
                reads: &[
                    (0x0, F32),
                    (0x4, F32),
                    (0x8, F32),
                    (0x10, F32),
                    (0x14, F32),
                    (0x18, F32),
                    (0x20, F32),
                    (0x24, F32),
                    (0x28, F32),
                ],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.run_ring.box_place_x87", &SPEC, |b| {
        with_laid::<std::cell::RefCell<RingWork>, _>(b[0], |w| {
            crate::effects::ring_entries::with_entry_copy_at(b[1], 0x10, 0x1C, |e| {
                super::box_place_x87(e, &mut w.borrow_mut(), &car_matrix(b[2]));
            });
        });
    });
}
