use crate::effects::asm_test::Kind::*;
use crate::effects::asm_test::{Op, Spec, recorded};
use crate::memory::session::PlainFloat;

/// The `N` bytes of the test buffer at `at`.
fn bytes<const N: usize>(at: crate::effects::asm_test::Operand) -> [u8; N] {
    // SAFETY: `host` checks that the byte lies in the operand's test buffer, which the recorded
    // run keeps alive and nothing else reaches meanwhile.
    std::array::from_fn(|n| unsafe { (at + n).host().read() })
}

/// The test buffer's dword at `at` as a float cell.
fn cell(at: crate::effects::asm_test::Operand) -> PlainFloat {
    PlainFloat::from_bits(u32::from_le_bytes(bytes::<4>(at)))
}

/// `data` into the test buffer at `at`.
fn put(at: crate::effects::asm_test::Operand, data: &[u8]) {
    for (n, &b) in data.iter().enumerate() {
        // SAFETY: `host` checks that the byte lies in the operand's test buffer, which the
        // recorded run keeps alive and nothing else reaches meanwhile.
        unsafe { (at + n).host().write(b) };
    }
}

/// The two ten-byte inputs at the test buffer's `at` (0 and 0xA).
fn pair(at: crate::effects::asm_test::Operand) -> [[u8; 10]; 2] {
    [bytes::<10>(at), bytes::<10>(at + 0xA)]
}

#[test]
fn turn_pair_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x0,
                hi: 0x14,
                reads: &[(0x0, F80), (0xA, F80)],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x14,
                reads: &[(0x0, F32), (0x4, F32), (0xC, F32), (0x10, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x14,
                reads: &[],
                writes: &[(0x0, F80), (0xA, F80)],
            },
        ],
        sincos: false,
    };
    recorded("effects.turns.turn_pair_x87", &SPEC, |b| {
        // The turn's floats 0, 1, 3 and 4 (byte offsets 0, 4, 0xC, 0x10).
        let m: [f32; 9] = std::array::from_fn(|n| {
            if n < 5 {
                cell(b[1] + 4 * n).get()
            } else {
                cell(b[1]).get()
            }
        });
        let out = super::turn_pair_x87(&pair(b[0]), &m);
        put(b[2], &out[0]);
        put(b[2] + 0xA, &out[1]);
    });
}

#[test]
fn scaled_ints_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x0,
                hi: 0x14,
                reads: &[(0x0, F80), (0xA, F80)],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x8,
                reads: &[(0x0, F32), (0x4, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x8,
                reads: &[],
                writes: &[(0x0, I32), (0x4, I32)],
            },
        ],
        sincos: false,
    };
    recorded("effects.turns.scaled_ints_x87", &SPEC, |b| {
        let (v0, v1) = super::scaled_ints_x87(&pair(b[0]), cell(b[1]), cell(b[1] + 4));
        put(b[2], &v0.to_le_bytes());
        put(b[2] + 4, &v1.to_le_bytes());
    });
}

#[test]
fn int_over_80_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x0,
                hi: 0x4,
                reads: &[(0x0, I32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x4,
                reads: &[(0x0, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0xA,
                reads: &[],
                writes: &[(0x0, F80)],
            },
        ],
        sincos: false,
    };
    recorded("effects.turns.int_over_80_x87", &SPEC, |b| {
        let v = i32::from_le_bytes(bytes::<4>(b[0]));
        put(b[2], &super::int_over_80_x87(v, cell(b[1])));
    });
}
