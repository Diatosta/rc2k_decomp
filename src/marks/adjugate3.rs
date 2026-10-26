//! `marks::adjugate3` (0x4BCED6): the adjugate of the 3 x 3 matrix at esi (rows 12 bytes
//! apart) into edi's: each entry a difference of two products (`fsubp`), rounded as written
//! (so an output over the input is read back as the original does).
//!
//! In:  esi the matrix, edi the result
//! Out: every register kept
//!
//! Caller: `sub_4BCB1D` (0x4BCD63). The arithmetic is plain Rust on the x87 model
//! (`math::x87_float`), in the original's order. Refuses a null pointer.

use crate::errors::{BadInput, Refused};

/// The Rust version (see the header).
pub fn adjugate3(s: u32, d: u32) -> Result<(), Refused> {
    if s == 0 || d == 0 {
        return Err(BadInput("a null matrix").into());
    }
    adjugate3_x87(s as usize, d as usize);
    Ok(())
}

use crate::marks::plain::{ld, st};

/// `adjugate3`'s x87 block as plain Rust on the x87 model (54 instructions), same order and memory places.
fn adjugate3_x87(s: usize, d: usize) {
    let t0 = ld(s, 0x10);
    let t1 = t0 * ld(s, 0x20);
    let t2 = ld(s, 0x14);
    let t3 = t2 * ld(s, 0x1C);
    let t4 = t1 - t3;
    st(d, 0, t4);
    let t5 = ld(s, 0x8);
    let t6 = t5 * ld(s, 0x1C);
    let t7 = ld(s, 0x4);
    let t8 = t7 * ld(s, 0x20);
    let t9 = t6 - t8;
    st(d, 0x4, t9);
    let t10 = ld(s, 0x4);
    let t11 = t10 * ld(s, 0x14);
    let t12 = ld(s, 0x8);
    let t13 = t12 * ld(s, 0x10);
    let t14 = t11 - t13;
    st(d, 0x8, t14);
    let t15 = ld(s, 0x14);
    let t16 = t15 * ld(s, 0x18);
    let t17 = ld(s, 0xC);
    let t18 = t17 * ld(s, 0x20);
    let t19 = t16 - t18;
    st(d, 0xC, t19);
    let t20 = ld(s, 0);
    let t21 = t20 * ld(s, 0x20);
    let t22 = ld(s, 0x8);
    let t23 = t22 * ld(s, 0x18);
    let t24 = t21 - t23;
    st(d, 0x10, t24);
    let t25 = ld(s, 0x8);
    let t26 = t25 * ld(s, 0xC);
    let t27 = ld(s, 0);
    let t28 = t27 * ld(s, 0x14);
    let t29 = t26 - t28;
    st(d, 0x14, t29);
    let t30 = ld(s, 0xC);
    let t31 = t30 * ld(s, 0x1C);
    let t32 = ld(s, 0x10);
    let t33 = t32 * ld(s, 0x18);
    let t34 = t31 - t33;
    st(d, 0x18, t34);
    let t35 = ld(s, 0x4);
    let t36 = t35 * ld(s, 0x18);
    let t37 = ld(s, 0);
    let t38 = t37 * ld(s, 0x1C);
    let t39 = t36 - t38;
    st(d, 0x1C, t39);
    let t40 = ld(s, 0);
    let t41 = t40 * ld(s, 0x10);
    let t42 = ld(s, 0x4);
    let t43 = t42 * ld(s, 0xC);
    let t44 = t41 - t43;
    st(d, 0x20, t44);
}

#[cfg(test)]
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
            )
        }
    }
}

#[cfg(test)]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::marks::asm_test::Kind::*;
    use crate::marks::asm_test::{Op, Spec, differential};

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
            |b| super::adjugate3_x87(b[0], b[1]),
        );
    }
}
