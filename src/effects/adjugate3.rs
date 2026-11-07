//! `effects::adjugate3` (0x4BCED6): the adjugate of the 3 x 3 matrix at esi (rows 12 bytes
//! apart) into edi's: each entry a difference of two products (`fsubp`), rounded as written
//! (so an output over the input is read back as the original does).
//!
//! Takes: the matrix, the result (here two arrays of nine floats)
//!
//! Caller: `sub_4BCB1D` (0x4BCD63). The arithmetic is plain Rust on the x87 model
//! (`math::x87_float`), in the original's order.

use crate::session::Float;

/// The Rust version (see the header): `s` the matrix, `d` the result (another array).
pub fn adjugate3(s: &[Float; 9], d: &[Float; 9]) {
    adjugate3_x87(s, d);
}

use crate::effects::plain::{ld_cell, st_cell};

/// `adjugate3`'s x87 block as plain Rust on the x87 model (54 instructions), same order and memory places.
fn adjugate3_x87(s: &[Float; 9], d: &[Float; 9]) {
    let t0 = ld_cell(&s[4]);
    let t1 = t0 * ld_cell(&s[8]);
    let t2 = ld_cell(&s[5]);
    let t3 = t2 * ld_cell(&s[7]);
    let t4 = t1 - t3;
    st_cell(&d[0], t4);
    let t5 = ld_cell(&s[2]);
    let t6 = t5 * ld_cell(&s[7]);
    let t7 = ld_cell(&s[1]);
    let t8 = t7 * ld_cell(&s[8]);
    let t9 = t6 - t8;
    st_cell(&d[1], t9);
    let t10 = ld_cell(&s[1]);
    let t11 = t10 * ld_cell(&s[5]);
    let t12 = ld_cell(&s[2]);
    let t13 = t12 * ld_cell(&s[4]);
    let t14 = t11 - t13;
    st_cell(&d[2], t14);
    let t15 = ld_cell(&s[5]);
    let t16 = t15 * ld_cell(&s[6]);
    let t17 = ld_cell(&s[3]);
    let t18 = t17 * ld_cell(&s[8]);
    let t19 = t16 - t18;
    st_cell(&d[3], t19);
    let t20 = ld_cell(&s[0]);
    let t21 = t20 * ld_cell(&s[8]);
    let t22 = ld_cell(&s[2]);
    let t23 = t22 * ld_cell(&s[6]);
    let t24 = t21 - t23;
    st_cell(&d[4], t24);
    let t25 = ld_cell(&s[2]);
    let t26 = t25 * ld_cell(&s[3]);
    let t27 = ld_cell(&s[0]);
    let t28 = t27 * ld_cell(&s[5]);
    let t29 = t26 - t28;
    st_cell(&d[5], t29);
    let t30 = ld_cell(&s[3]);
    let t31 = t30 * ld_cell(&s[7]);
    let t32 = ld_cell(&s[4]);
    let t33 = t32 * ld_cell(&s[6]);
    let t34 = t31 - t33;
    st_cell(&d[6], t34);
    let t35 = ld_cell(&s[1]);
    let t36 = t35 * ld_cell(&s[6]);
    let t37 = ld_cell(&s[0]);
    let t38 = t37 * ld_cell(&s[7]);
    let t39 = t36 - t38;
    st_cell(&d[7], t39);
    let t40 = ld_cell(&s[0]);
    let t41 = t40 * ld_cell(&s[4]);
    let t42 = ld_cell(&s[1]);
    let t43 = t42 * ld_cell(&s[3]);
    let t44 = t41 - t43;
    st_cell(&d[8], t44);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn adjugate3_x87(s: usize, d: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{s} + 0x10]",
            "fmul dword ptr [{s} + 0x20]",
            "fld dword ptr [{s} + 0x14]",
            "fmul dword ptr [{s} + 0x1C]",
            ".byte 0xDE, 0xE9",
            "fstp dword ptr [{d} + 0x0]",
            "fld dword ptr [{s} + 0x8]",
            "fmul dword ptr [{s} + 0x1C]",
            "fld dword ptr [{s} + 0x4]",
            "fmul dword ptr [{s} + 0x20]",
            ".byte 0xDE, 0xE9",
            "fstp dword ptr [{d} + 0x4]",
            "fld dword ptr [{s} + 0x4]",
            "fmul dword ptr [{s} + 0x14]",
            "fld dword ptr [{s} + 0x8]",
            "fmul dword ptr [{s} + 0x10]",
            ".byte 0xDE, 0xE9",
            "fstp dword ptr [{d} + 0x8]",
            "fld dword ptr [{s} + 0x14]",
            "fmul dword ptr [{s} + 0x18]",
            "fld dword ptr [{s} + 0xC]",
            "fmul dword ptr [{s} + 0x20]",
            ".byte 0xDE, 0xE9",
            "fstp dword ptr [{d} + 0xC]",
            "fld dword ptr [{s} + 0x0]",
            "fmul dword ptr [{s} + 0x20]",
            "fld dword ptr [{s} + 0x8]",
            "fmul dword ptr [{s} + 0x18]",
            ".byte 0xDE, 0xE9",
            "fstp dword ptr [{d} + 0x10]",
            "fld dword ptr [{s} + 0x8]",
            "fmul dword ptr [{s} + 0xC]",
            "fld dword ptr [{s} + 0x0]",
            "fmul dword ptr [{s} + 0x14]",
            ".byte 0xDE, 0xE9",
            "fstp dword ptr [{d} + 0x14]",
            "fld dword ptr [{s} + 0xC]",
            "fmul dword ptr [{s} + 0x1C]",
            "fld dword ptr [{s} + 0x10]",
            "fmul dword ptr [{s} + 0x18]",
            ".byte 0xDE, 0xE9",
            "fstp dword ptr [{d} + 0x18]",
            "fld dword ptr [{s} + 0x4]",
            "fmul dword ptr [{s} + 0x18]",
            "fld dword ptr [{s} + 0x0]",
            "fmul dword ptr [{s} + 0x1C]",
            ".byte 0xDE, 0xE9",
            "fstp dword ptr [{d} + 0x1C]",
            "fld dword ptr [{s} + 0x0]",
            "fmul dword ptr [{s} + 0x10]",
            "fld dword ptr [{s} + 0x4]",
            "fmul dword ptr [{s} + 0xC]",
            ".byte 0xDE, 0xE9",
            "fstp dword ptr [{d} + 0x20]",
            s = in(reg) s,
            d = in(reg) d,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Spec, differential};
    use crate::session::Float;

    /// Nine floats holding the test buffer's (at `at`) dwords.
    fn staged(at: usize) -> [Float; 9] {
        std::array::from_fn(|n| {
            Float::from_bits(unsafe {
                crate::pointers::host_at(at + 4 * n)
                    .cast::<u32>()
                    .read_unaligned()
            })
        })
    }

    /// The nine floats back into the test buffer at `at`.
    fn put_back(at: usize, cells: &[Float; 9]) {
        for (n, cell) in cells.iter().enumerate() {
            unsafe {
                crate::pointers::host_at(at + 4 * n)
                    .cast::<u32>()
                    .write_unaligned(cell.bits());
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
        differential(
            &SPEC,
            |b| reference::adjugate3_x87(b[0], b[1]),
            |b| {
                let (s, d) = (staged(b[0]), staged(b[1]));
                super::adjugate3_x87(&s, &d);
                put_back(b[1], &d);
            },
        );
    }
}
