use crate::effects::asm_test::Kind::*;
use crate::effects::asm_test::{Op, Spec, recorded_writes};
use crate::effects::rain::tests::with_state_at;

#[test]
fn box_moved_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x1574C5,
                hi: 0x1610BD,
                reads: &[
                    (0x157585, I32),
                    (0x157589, I32),
                    (0x15758D, I32),
                    (0x157591, I32),
                    (0x157595, I32),
                    (0x157599, I32),
                    (0x15759D, I32),
                    (0x1575A1, I32),
                    (0x1575A5, I32),
                    (0x1575C1, I32),
                    (0x1575C5, I32),
                    (0x1575C9, I32),
                    (0x1575FD, I32),
                    (0x157601, I32),
                    (0x157605, I32),
                    (0x158E3D, I32),
                    (0x158E41, I32),
                    (0x158E45, I32),
                    (0x161055, F32),
                    (0x161059, F32),
                    (0x16105D, F32),
                ],
                writes: &[
                    (0x157549, F32),
                    (0x15754D, F32),
                    (0x157551, F32),
                    (0x157555, F32),
                    (0x157559, F32),
                    (0x15755D, F32),
                    (0x157561, F32),
                    (0x157565, F32),
                    (0x157569, F32),
                    (0x15756D, F32),
                    (0x157571, F32),
                    (0x157575, F32),
                    (0x157579, F32),
                    (0x15757D, F32),
                    (0x157581, F32),
                    (0x157585, F32),
                    (0x157589, F32),
                    (0x15758D, F32),
                    (0x1575C1, F32),
                    (0x1575C5, F32),
                    (0x1575C9, F32),
                ],
            },
            Op::Index(&[0, 12, 24, 36, 48]),
        ],
        sincos: false,
    };
    recorded_writes("effects.run_rain.box_moved_x87", &SPEC, |b| {
        with_state_at(b[0], |r, k| super::box_moved_x87(b[1].number() / 12, r, k));
    });
}

#[test]
fn move_length_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::RaceBlock {
                reads: &[],
                writes: &[(0x31C814, Status)],
            },
            Op::Base {
                lo: 0x1574C5,
                hi: 0x1610BD,
                reads: &[
                    (0x15753D, F32),
                    (0x157541, F32),
                    (0x157545, F32),
                    (0x161061, F32),
                ],
                writes: &[(0x158E1D, F32)],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.run_rain.move_length_x87", &SPEC, |b| {
        with_state_at(b[1], |r, k| {
            super::move_length_x87(r, k, &crate::test_support::own::effects().setup.work);
        });
    });
}

#[test]
fn move_held_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[Op::Base {
            lo: 0x1574C5,
            hi: 0x1610BD,
            reads: &[
                (0x15753D, F32),
                (0x157541, F32),
                (0x157545, F32),
                (0x158E1D, F32),
                (0x161065, F32),
                (0x161069, F32),
                (0x16106D, F32),
                (0x161071, F32),
            ],
            writes: &[(0x15753D, F32), (0x157541, F32), (0x157545, F32)],
        }],
        sincos: false,
    };
    recorded_writes("effects.run_rain.move_held_x87", &SPEC, |b| {
        with_state_at(b[0], super::move_held_x87);
    });
}

#[test]
fn keep_y_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x1574C5,
                hi: 0x1610BD,
                reads: &[(0x1575C5, I32), (0x158E41, I32)],
                writes: &[
                    (0x157589, I32),
                    (0x157595, I32),
                    (0x1575A1, I32),
                    (0x1575AD, I32),
                    (0x1575B9, I32),
                    (0x1575C5, I32),
                    (0x157601, I32),
                ],
            },
            Op::Index(&[0, 12, 24, 36, 48]),
        ],
        sincos: false,
    };
    recorded_writes("effects.run_rain.keep_y_x87", &SPEC, |b| {
        with_state_at(b[0], |r, _| super::keep_y_x87(b[1].number() / 12, r));
    });
}
