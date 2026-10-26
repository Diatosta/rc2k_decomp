//! `marks::scale_axes` (0x4BFD73): the two axes at `flt_89CF1A` and `flt_89CF26` scaled: by `flt_89CECA` into `flt_89CF3E` / `flt_89CF4A`, by `flt_89CEEE` / `flt_89CEF2` and the per-part factors `flt_8BC1DE` .. into `flt_89CF6E` / `flt_89CF7A`, and the first two times `flt_8BC1F6` .. into `flt_89CF56` / `flt_89CF62`.
//!
//! In:  nothing
//! Out: nothing (no caller reads a register after it)
//!
//! Callers: 0x4bf75e, 0x4c90b3. The arithmetic is plain Rust on the x87 model
//! (`math::x87_float`), in the original's order, over the game's variables.

/// The Rust version (see the header).
pub fn scale_axes() {
    scale_axes_x87(&SQUARE_TABLES, &SQUARE_STATE);
}

use crate::marks::squares_store::{SQUARE_STATE, SQUARE_TABLES, SquareState, SquareTables};
use crate::math::x87_float as bh;

/// `scale_axes`'s x87 block as plain Rust on the x87 model (60 instructions), same order and memory places.
fn scale_axes_x87(sq: &SquareTables, ss: &SquareState) {
    {
        let t0 = bh::ld(sq.axes[0][0].bits());
        let t1 = t0 * bh::ld(sq.radii[0].bits());
        sq.scaled_by_radius[0][0].set_bits(bh::st(t1));
        let t2 = bh::ld(sq.axes[0][1].bits());
        let t3 = t2 * bh::ld(sq.radii[0].bits());
        sq.scaled_by_radius[0][1].set_bits(bh::st(t3));
        let t4 = bh::ld(sq.axes[0][2].bits());
        let t5 = t4 * bh::ld(sq.radii[0].bits());
        sq.scaled_by_radius[0][2].set_bits(bh::st(t5));
        let t6 = bh::ld(sq.axes[1][0].bits());
        let t7 = t6 * bh::ld(sq.radii[0].bits());
        sq.scaled_by_radius[1][0].set_bits(bh::st(t7));
        let t8 = bh::ld(sq.axes[1][1].bits());
        let t9 = t8 * bh::ld(sq.radii[0].bits());
        sq.scaled_by_radius[1][1].set_bits(bh::st(t9));
        let t10 = bh::ld(sq.axes[1][2].bits());
        let t11 = t10 * bh::ld(sq.radii[0].bits());
        sq.scaled_by_radius[1][2].set_bits(bh::st(t11));
        let t12 = bh::ld(sq.axes[0][0].bits());
        let t13 = t12 * bh::ld(sq.lengths[0].bits());
        let t14 = t13 * bh::ld(ss.axis_factors[0].bits());
        sq.scaled_axes[0][0].set_bits(bh::st(t14));
        let t15 = bh::ld(sq.axes[0][1].bits());
        let t16 = t15 * bh::ld(sq.lengths[0].bits());
        let t17 = t16 * bh::ld(ss.axis_factors[1].bits());
        sq.scaled_axes[0][1].set_bits(bh::st(t17));
        let t18 = bh::ld(sq.axes[0][2].bits());
        let t19 = t18 * bh::ld(sq.lengths[0].bits());
        let t20 = t19 * bh::ld(ss.axis_factors[2].bits());
        sq.scaled_axes[0][2].set_bits(bh::st(t20));
        let t21 = bh::ld(sq.axes[1][0].bits());
        let t22 = t21 * bh::ld(sq.lengths[1].bits());
        let t23 = t22 * bh::ld(ss.axis_factors[3].bits());
        sq.scaled_axes[1][0].set_bits(bh::st(t23));
        let t24 = bh::ld(sq.axes[1][1].bits());
        let t25 = t24 * bh::ld(sq.lengths[1].bits());
        let t26 = t25 * bh::ld(ss.axis_factors[4].bits());
        sq.scaled_axes[1][1].set_bits(bh::st(t26));
        let t27 = bh::ld(sq.axes[1][2].bits());
        let t28 = t27 * bh::ld(sq.lengths[1].bits());
        let t29 = t28 * bh::ld(ss.axis_factors[5].bits());
        sq.scaled_axes[1][2].set_bits(bh::st(t29));
        let t30 = bh::ld(sq.scaled_by_radius[0][0].bits());
        let t31 = t30 * bh::ld(ss.halves[0].bits());
        sq.half_axes[0][0].set_bits(bh::st(t31));
        let t32 = bh::ld(sq.scaled_by_radius[0][1].bits());
        let t33 = t32 * bh::ld(ss.halves[1].bits());
        sq.half_axes[0][1].set_bits(bh::st(t33));
        let t34 = bh::ld(sq.scaled_by_radius[0][2].bits());
        let t35 = t34 * bh::ld(ss.halves[2].bits());
        sq.half_axes[0][2].set_bits(bh::st(t35));
        let t36 = bh::ld(sq.scaled_by_radius[1][0].bits());
        let t37 = t36 * bh::ld(ss.halves[3].bits());
        sq.half_axes[1][0].set_bits(bh::st(t37));
        let t38 = bh::ld(sq.scaled_by_radius[1][1].bits());
        let t39 = t38 * bh::ld(ss.halves[4].bits());
        sq.half_axes[1][1].set_bits(bh::st(t39));
        let t40 = bh::ld(sq.scaled_by_radius[1][2].bits());
        let t41 = t40 * bh::ld(ss.halves[5].bits());
        sq.half_axes[1][2].set_bits(bh::st(t41));
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn scale_axes_x87(mk: usize) {
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
    use crate::marks::asm_test::with_copy;
    use crate::marks::asm_test::{Op, Spec, differential};
    use crate::marks::squares_store::{SquareState, SquareTables};

    #[test]
    fn scale_axes_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x137B8A,
                hi: 0x157326,
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
            |b| reference::scale_axes_x87(b[0]),
            |b| {
                with_copy::<SquareTables, _>(b[0], SquareTables::START, |sq| {
                    with_copy::<SquareState, _>(b[0], SquareState::START, |ss| {
                        super::scale_axes_x87(sq, ss)
                    })
                })
            },
        );
    }
}
