//! `marks::scale_axes` (0x4BFD73): the two axes at `flt_89CF1A` and `flt_89CF26` scaled: by `flt_89CECA` into `flt_89CF3E` / `flt_89CF4A`, by `flt_89CEEE` / `flt_89CEF2` and the per-part factors `flt_8BC1DE` .. into `flt_89CF6E` / `flt_89CF7A`, and the first two times `flt_8BC1F6` .. into `flt_89CF56` / `flt_89CF62`.
//!
//! In:  nothing
//! Out: nothing (no caller reads a register after it)
//!
//! Callers: 0x4bf75e, 0x4c90b3. The arithmetic is plain Rust on the x87 model
//! (`math::x87_float`), in the original's order, over the game's variables.

/// The Rust version (see the header).
pub unsafe fn scale_axes() {
    unsafe {
        scale_axes_x87(crate::marks::store::base() as usize);
    }
}

use crate::marks::plain::{ld, st};

/// `scale_axes`'s x87 block as plain Rust on the x87 model (60 instructions), same order and memory places.
unsafe fn scale_axes_x87(mk: usize) {
    unsafe {
        let t0 = ld(mk, 0x137FDA);
        let t1 = t0 * ld(mk, 0x137F8A);
        st(mk, 0x137FFE, t1);
        let t2 = ld(mk, 0x137FDE);
        let t3 = t2 * ld(mk, 0x137F8A);
        st(mk, 0x138002, t3);
        let t4 = ld(mk, 0x137FE2);
        let t5 = t4 * ld(mk, 0x137F8A);
        st(mk, 0x138006, t5);
        let t6 = ld(mk, 0x137FE6);
        let t7 = t6 * ld(mk, 0x137F8A);
        st(mk, 0x13800A, t7);
        let t8 = ld(mk, 0x137FEA);
        let t9 = t8 * ld(mk, 0x137F8A);
        st(mk, 0x13800E, t9);
        let t10 = ld(mk, 0x137FEE);
        let t11 = t10 * ld(mk, 0x137F8A);
        st(mk, 0x138012, t11);
        let t12 = ld(mk, 0x137FDA);
        let t13 = t12 * ld(mk, 0x137FAE);
        let t14 = t13 * ld(mk, 0x15729E);
        st(mk, 0x13802E, t14);
        let t15 = ld(mk, 0x137FDE);
        let t16 = t15 * ld(mk, 0x137FAE);
        let t17 = t16 * ld(mk, 0x1572A2);
        st(mk, 0x138032, t17);
        let t18 = ld(mk, 0x137FE2);
        let t19 = t18 * ld(mk, 0x137FAE);
        let t20 = t19 * ld(mk, 0x1572A6);
        st(mk, 0x138036, t20);
        let t21 = ld(mk, 0x137FE6);
        let t22 = t21 * ld(mk, 0x137FB2);
        let t23 = t22 * ld(mk, 0x1572AA);
        st(mk, 0x13803A, t23);
        let t24 = ld(mk, 0x137FEA);
        let t25 = t24 * ld(mk, 0x137FB2);
        let t26 = t25 * ld(mk, 0x1572AE);
        st(mk, 0x13803E, t26);
        let t27 = ld(mk, 0x137FEE);
        let t28 = t27 * ld(mk, 0x137FB2);
        let t29 = t28 * ld(mk, 0x1572B2);
        st(mk, 0x138042, t29);
        let t30 = ld(mk, 0x137FFE);
        let t31 = t30 * ld(mk, 0x1572B6);
        st(mk, 0x138016, t31);
        let t32 = ld(mk, 0x138002);
        let t33 = t32 * ld(mk, 0x1572BA);
        st(mk, 0x13801A, t33);
        let t34 = ld(mk, 0x138006);
        let t35 = t34 * ld(mk, 0x1572BE);
        st(mk, 0x13801E, t35);
        let t36 = ld(mk, 0x13800A);
        let t37 = t36 * ld(mk, 0x1572C2);
        st(mk, 0x138022, t37);
        let t38 = ld(mk, 0x13800E);
        let t39 = t38 * ld(mk, 0x1572C6);
        st(mk, 0x138026, t39);
        let t40 = ld(mk, 0x138012);
        let t41 = t40 * ld(mk, 0x1572CA);
        st(mk, 0x13802A, t41);
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn scale_axes_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x137FDA]",
            "fmul dword ptr [{mk} + 0x137F8A]",
            "fstp dword ptr [{mk} + 0x137FFE]",
            "fld dword ptr [{mk} + 0x137FDE]",
            "fmul dword ptr [{mk} + 0x137F8A]",
            "fstp dword ptr [{mk} + 0x138002]",
            "fld dword ptr [{mk} + 0x137FE2]",
            "fmul dword ptr [{mk} + 0x137F8A]",
            "fstp dword ptr [{mk} + 0x138006]",
            "fld dword ptr [{mk} + 0x137FE6]",
            "fmul dword ptr [{mk} + 0x137F8A]",
            "fstp dword ptr [{mk} + 0x13800A]",
            "fld dword ptr [{mk} + 0x137FEA]",
            "fmul dword ptr [{mk} + 0x137F8A]",
            "fstp dword ptr [{mk} + 0x13800E]",
            "fld dword ptr [{mk} + 0x137FEE]",
            "fmul dword ptr [{mk} + 0x137F8A]",
            "fstp dword ptr [{mk} + 0x138012]",
            "fld dword ptr [{mk} + 0x137FDA]",
            "fmul dword ptr [{mk} + 0x137FAE]",
            "fmul dword ptr [{mk} + 0x15729E]",
            "fstp dword ptr [{mk} + 0x13802E]",
            "fld dword ptr [{mk} + 0x137FDE]",
            "fmul dword ptr [{mk} + 0x137FAE]",
            "fmul dword ptr [{mk} + 0x1572A2]",
            "fstp dword ptr [{mk} + 0x138032]",
            "fld dword ptr [{mk} + 0x137FE2]",
            "fmul dword ptr [{mk} + 0x137FAE]",
            "fmul dword ptr [{mk} + 0x1572A6]",
            "fstp dword ptr [{mk} + 0x138036]",
            "fld dword ptr [{mk} + 0x137FE6]",
            "fmul dword ptr [{mk} + 0x137FB2]",
            "fmul dword ptr [{mk} + 0x1572AA]",
            "fstp dword ptr [{mk} + 0x13803A]",
            "fld dword ptr [{mk} + 0x137FEA]",
            "fmul dword ptr [{mk} + 0x137FB2]",
            "fmul dword ptr [{mk} + 0x1572AE]",
            "fstp dword ptr [{mk} + 0x13803E]",
            "fld dword ptr [{mk} + 0x137FEE]",
            "fmul dword ptr [{mk} + 0x137FB2]",
            "fmul dword ptr [{mk} + 0x1572B2]",
            "fstp dword ptr [{mk} + 0x138042]",
            "fld dword ptr [{mk} + 0x137FFE]",
            "fmul dword ptr [{mk} + 0x1572B6]",
            "fstp dword ptr [{mk} + 0x138016]",
            "fld dword ptr [{mk} + 0x138002]",
            "fmul dword ptr [{mk} + 0x1572BA]",
            "fstp dword ptr [{mk} + 0x13801A]",
            "fld dword ptr [{mk} + 0x138006]",
            "fmul dword ptr [{mk} + 0x1572BE]",
            "fstp dword ptr [{mk} + 0x13801E]",
            "fld dword ptr [{mk} + 0x13800A]",
            "fmul dword ptr [{mk} + 0x1572C2]",
            "fstp dword ptr [{mk} + 0x138022]",
            "fld dword ptr [{mk} + 0x13800E]",
            "fmul dword ptr [{mk} + 0x1572C6]",
            "fstp dword ptr [{mk} + 0x138026]",
            "fld dword ptr [{mk} + 0x138012]",
            "fmul dword ptr [{mk} + 0x1572CA]",
            "fstp dword ptr [{mk} + 0x13802A]",
            mk = in(reg) mk,
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
    fn scale_axes_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x137F8A,
                hi: 0x1572CE,
                reads: &[
                    (0x137F8A, F32),
                    (0x137FAE, F32),
                    (0x137FB2, F32),
                    (0x137FDA, F32),
                    (0x137FDE, F32),
                    (0x137FE2, F32),
                    (0x137FE6, F32),
                    (0x137FEA, F32),
                    (0x137FEE, F32),
                    (0x137FFE, F32),
                    (0x138002, F32),
                    (0x138006, F32),
                    (0x13800A, F32),
                    (0x13800E, F32),
                    (0x138012, F32),
                    (0x15729E, F32),
                    (0x1572A2, F32),
                    (0x1572A6, F32),
                    (0x1572AA, F32),
                    (0x1572AE, F32),
                    (0x1572B2, F32),
                    (0x1572B6, F32),
                    (0x1572BA, F32),
                    (0x1572BE, F32),
                    (0x1572C2, F32),
                    (0x1572C6, F32),
                    (0x1572CA, F32),
                ],
                writes: &[
                    (0x137FFE, F32),
                    (0x138002, F32),
                    (0x138006, F32),
                    (0x13800A, F32),
                    (0x13800E, F32),
                    (0x138012, F32),
                    (0x138016, F32),
                    (0x13801A, F32),
                    (0x13801E, F32),
                    (0x138022, F32),
                    (0x138026, F32),
                    (0x13802A, F32),
                    (0x13802E, F32),
                    (0x138032, F32),
                    (0x138036, F32),
                    (0x13803A, F32),
                    (0x13803E, F32),
                    (0x138042, F32),
                ],
            }],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::scale_axes_x87(b[0]) },
            |b| unsafe { super::scale_axes_x87(b[0]) },
        );
    }
}
