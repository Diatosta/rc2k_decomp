use crate::effects::asm_test::Kind::*;
use crate::effects::asm_test::{Op, Spec, recorded};

/// Nine floats holding the test buffer's (at `at`) dwords.
fn staged(at: crate::effects::asm_test::Operand) -> [f32; 9] {
    std::array::from_fn(|n| {
        // SAFETY: `at` checks that the four bytes lie in the operand's test buffer, which the
        // recorded run keeps alive and nothing else reaches meanwhile; unaligned, any bits a
        // `u32`.
        f32::from_bits(unsafe { (at + 4 * n).at(0, 4).cast::<u32>().read_unaligned() })
    })
}

/// The nine floats back into the test buffer at `at`.
fn put_back(at: crate::effects::asm_test::Operand, values: &[f32; 9]) {
    for (n, cell) in values.iter().enumerate() {
        // SAFETY: `at` checks that the four bytes lie in the operand's test buffer, which the
        // recorded run keeps alive and nothing else reaches meanwhile; unaligned, any bits a
        // `u32`.
        unsafe {
            (at + 4 * n)
                .at(0, 4)
                .cast::<u32>()
                .write_unaligned(cell.to_bits());
        }
    }
}

#[test]
fn adjugate3_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
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
                hi: 0x24,
                reads: &[],
                writes: &[
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
            },
        ],
        sincos: false,
    };
    recorded("effects.adjugate3.adjugate3_x87", &SPEC, |b| {
        let (s, mut d) = (staged(b[0]), staged(b[1]));
        super::adjugate3_x87(&s, &mut d);
        put_back(b[1], &d);
    });
}
