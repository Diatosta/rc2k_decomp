use crate::effects::asm_test::Kind::*;
use crate::effects::asm_test::{Op, Operand, Spec, recorded_buffer_writes};
use crate::math::x87_mem::Base;

/// The float at `base + off` of a test buffer.
fn at(base: Operand, off: usize) -> f32 {
    f32::from_bits(base.dword(off))
}

/// The status word written as `fnstsw` does.
fn put(base: Operand, status: u16) {
    base.set_word(0, status);
}

#[test]
fn sum_compare_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x0,
                hi: 0xC,
                reads: &[(0x0, F32), (0x4, F32), (0x8, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x2,
                reads: &[],
                writes: &[(0x0, Status)],
            },
        ],
        sincos: false,
    };
    recorded_buffer_writes("effects.pong.sum_compare_x87", &SPEC, |b| {
        put(
            b[1],
            super::sum_status(at(b[0], 0), at(b[0], 4), at(b[0], 8)),
        );
    });
}

#[test]
fn less_compare_x87_1_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x0,
                hi: 0x10,
                reads: &[(0x0, F32), (0x4, F32), (0x8, F32), (0xC, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x2,
                reads: &[],
                writes: &[(0x0, Status)],
            },
        ],
        sincos: false,
    };
    recorded_buffer_writes("effects.pong.less_compare_x87_1", &SPEC, |b| {
        let m = Some(at(b[0], 0xC));
        put(
            b[1],
            super::less_status(at(b[0], 0), at(b[0], 4), at(b[0], 8), m),
        );
    });
}

#[test]
fn less_compare_x87_2_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x0,
                hi: 0xC,
                reads: &[(0x0, F32), (0x4, F32), (0x8, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x2,
                reads: &[],
                writes: &[(0x0, Status)],
            },
        ],
        sincos: false,
    };
    recorded_buffer_writes("effects.pong.less_compare_x87_2", &SPEC, |b| {
        put(
            b[1],
            super::less_status(at(b[0], 0), at(b[0], 4), at(b[0], 8), None),
        );
    });
}
