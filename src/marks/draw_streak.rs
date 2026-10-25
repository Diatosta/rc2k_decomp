//! `marks::draw_streak` (0x4C1DCD): a streak (a point and where it was) drawn as a quad
//! facing the viewer. The point (st(0) .. st(2), scaled by `flt_8BDD65` .. into
//! `flt_8BDE49` ..) turned by the matrix at `dword_8BDD89` .. plus the whole numbers at
//! `dword_8BDD71` .. into `flt_8BDE3D` .. (its depth also `flt_8BDD5D`); the point less
//! `dword_8BC47D` .. turned the same into `flt_8BDE49` ... Unless the first depth is at or
//! below 0x20000 or the second at or below 0x48000000 (signed compares of the bits), or
//! `dword_8BC405` is 0: both projected (`flt_624088` over the depth; scales `flt_8C5FE5`,
//! `flt_8C5FE9`, centre `flt_62409C`, `flt_6240A0`) into `flt_8BDD41` .., the half width
//! `flt_8C5FED` over the screen length across the line, the four vertices' x, y and depth
//! (times `flt_8BDD61`), `marks::clip_quad`, and unless all is outside `scene::draw_fan`
//! and `g_polygonVertexCount` 4. Rain, by the look of it (a reading).
//!
//! In:  st(0) .. st(2) the point (popped)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4C16F9` (0x4C1AD0). The arithmetic is plain Rust on the x87 model (`math::x87_float`), in the original's order, over the
//! game's variables. Refuses null marks settings (`clip_quad`'s
//! refusal), before anything.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::clip_quad::{clip_quad, draw_fan};
use crate::marks::store::{Slot, game_addr, game_read, game_write};

// flt_8BDD29 .. flt_8BDE51
const DEPTH_RVA: Slot = Slot::at(0x8BDD5D); // flt_8BDD5D
const FAR_RVA: Slot = Slot::at(0x8BDE51); // flt_8BDE51
const SWITCH_RVA: Slot = Slot::at(0x8BC405); // dword_8BC405
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount

/// The original's 0x4C1DCD .. 0x4C1F11: the point scaled and turned, and the last place.
unsafe fn turn(p: &[[u8; 10]; 3]) {
    unsafe { turn_x87(crate::marks::store::base() as usize, p.as_ptr() as usize) }
}

/// The original's 0x4C1F4A .. 0x4C20B3: both projected and the quad's x, y and depth.
unsafe fn project() {
    unsafe {
        project_x87(
            crate::race::store::asm_base() as usize,
            crate::race::store::buffers_asm_base() as usize,
            crate::marks::store::base() as usize,
        )
    }
}

/// The Rust version (see the header).
pub unsafe fn draw_streak(place: [[u8; 10]; 3]) -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(crate::marks::setup::SETTINGS_RVA) == 0 {
            return Err(BadInput("no marks settings").into());
        }
        turn(&place);
        if game_read::<i32>(DEPTH_RVA) <= 0x2_0000
            || game_read::<u32>(SWITCH_RVA) == 0
            || game_read::<i32>(FAR_RVA) <= 0x4800_0000
        {
            return Ok(());
        }
        project();
        if clip_quad(game_addr(VERTICES_RVA) as u32)? {
            return Ok(());
        }
        draw_fan();
        game_write(COUNT_RVA, 4u32);
    }
    Ok(())
}

use crate::marks::plain::{ld, ld80, ldi, st};
use crate::math::x87_float as bh;

/// `turn`'s x87 block as plain Rust on the x87 model (79 instructions), same order and memory places.
unsafe fn turn_x87(mk: usize, p: usize) {
    unsafe {
        let t0 = bh::from_f80(ld80(p, 0x14));
        let t1 = bh::from_f80(ld80(p, 0xA));
        let t2 = bh::from_f80(ld80(p, 0));
        let t3 = t0 * ld(mk, 0x158E25);
        st(mk, 0x158F09, t3);
        let t4 = t1 * ld(mk, 0x158E29);
        st(mk, 0x158F0D, t4);
        let t5 = t2 * ld(mk, 0x158E2D);
        st(mk, 0x158F11, t5);
        let t6 = t3 * ld(mk, 0x158E49);
        let t7 = t4 * ld(mk, 0x158E55);
        let t8 = t5 * ld(mk, 0x158E61);
        let t9 = t7 + t8;
        let t10 = t6 + t9;
        let t11 = t10 + ldi(mk, 0x158E31);
        st(mk, 0x158EFD, t11);
        let t12 = t3 * ld(mk, 0x158E4D);
        let t13 = t4 * ld(mk, 0x158E59);
        let t14 = t5 * ld(mk, 0x158E65);
        let t15 = t13 + t14;
        let t16 = t12 + t15;
        let t17 = t16 + ldi(mk, 0x158E35);
        st(mk, 0x158F01, t17);
        let t18 = t3 * ld(mk, 0x158E51);
        let t19 = t4 * ld(mk, 0x158E5D);
        let t20 = t5 * ld(mk, 0x158E69);
        let t21 = t19 + t20;
        let t22 = t18 + t21;
        let t23 = t22 + ldi(mk, 0x158E39);
        st(mk, 0x158E1D, t23);
        st(mk, 0x158F05, t23);
        let t24 = ld(mk, 0x158F0D);
        let t25 = t24 - ld(mk, 0x157541);
        let t26 = ld(mk, 0x158F09);
        let t27 = t26 - ld(mk, 0x15753D);
        let t28 = ld(mk, 0x158F11);
        let t29 = t28 - ld(mk, 0x157545);
        let t30 = t27 * ld(mk, 0x158E49);
        let t31 = t25 * ld(mk, 0x158E55);
        let t32 = t29 * ld(mk, 0x158E61);
        let t33 = t31 + t32;
        let t34 = t30 + t33;
        let t35 = t34 + ldi(mk, 0x158E31);
        st(mk, 0x158F09, t35);
        let t36 = t27 * ld(mk, 0x158E4D);
        let t37 = t25 * ld(mk, 0x158E59);
        let t38 = t29 * ld(mk, 0x158E65);
        let t39 = t37 + t38;
        let t40 = t36 + t39;
        let t41 = t40 + ldi(mk, 0x158E35);
        st(mk, 0x158F0D, t41);
        let t42 = t27 * ld(mk, 0x158E51);
        let t43 = t25 * ld(mk, 0x158E5D);
        let t44 = t29 * ld(mk, 0x158E69);
        let t45 = t43 + t44;
        let t46 = t42 + t45;
        let t47 = t46 + ldi(mk, 0x158E39);
        st(mk, 0x158F11, t47);
    }
}

/// `project`'s x87 block as plain Rust on the x87 model (85 instructions), same order and memory places.
unsafe fn project_x87(rc: usize, bf: usize, mk: usize) {
    unsafe {
        let t0 = ld(mk, 0x158F11);
        let t1 = ld(rc, 0x224088) / t0;
        let t2 = ld(mk, 0x158F05);
        let t3 = ld(rc, 0x224088) / t2;
        let t4 = ld(mk, 0x158F09);
        let t5 = ld(mk, 0x158F0D);
        let t6 = ld(mk, 0x158EFD);
        let t7 = ld(mk, 0x158F01);
        let t8 = t4 * t1;
        let t9 = t5 * t1;
        let t10 = t6 * t3;
        let t11 = t7 * t3;
        let t12 = t1 * ld(mk, 0x1610A5);
        let t13 = t3 * ld(mk, 0x1610A9);
        let t14 = t8 + ld(rc, 0x22409C);
        st(mk, 0x158E01, t14);
        let t15 = -t9;
        let t16 = t15 + ld(rc, 0x2240A0);
        st(mk, 0x158E05, t16);
        let t17 = t10 + ld(rc, 0x22409C);
        st(mk, 0x158E0D, t17);
        let t18 = -t11;
        let t19 = t18 + ld(rc, 0x2240A0);
        st(mk, 0x158E11, t19);
        st(mk, 0x158E09, t12);
        st(mk, 0x158E15, t13);
        let t20 = bh::ONE;
        let t21 = t20 / ld(rc, 0x2241C8);
        let t22 = ld(mk, 0x158E0D);
        let t23 = t22 - ld(mk, 0x158E01);
        st(mk, 0x158DED, t23);
        let t24 = t23 * t23;
        let t25 = ld(mk, 0x158E11);
        let t26 = t25 - ld(mk, 0x158E05);
        let t27 = -t26;
        st(mk, 0x158DE9, t27);
        let t28 = t27 * t27;
        let t29 = t24 + t28;
        let t30 = t29.sqrt();
        let t31 = ld(mk, 0x1610AD) / t30;
        let t32 = t31 * ld(mk, 0x158DE9);
        let t33 = t32 * ld(mk, 0x158E09);
        st(mk, 0x158DE9, t33);
        let t34 = t32 * ld(mk, 0x158E15);
        st(mk, 0x158DF5, t34);
        let t35 = t31 * ld(mk, 0x158DED);
        let t36 = t35 * ld(mk, 0x158E09);
        st(mk, 0x158DED, t36);
        let t37 = t35 * ld(mk, 0x158E15);
        st(mk, 0x158DF9, t37);
        st(mk, 0x158E21, t21);
        let t38 = ld(mk, 0x158F11);
        let t39 = t38 * ld(mk, 0x158E21);
        st(bf, 0x5605A8, t39);
        st(bf, 0x5605C8, t39);
        st(bf, 0x5605E8, t39);
        st(bf, 0x560608, t39);
        let t40 = ld(mk, 0x158E01);
        st(bf, 0x5605A0, t40);
        let t41 = t40 - ld(mk, 0x158DE9);
        st(bf, 0x5605C0, t41);
        let t42 = ld(mk, 0x158E0D);
        st(bf, 0x560600, t42);
        let t43 = t42 - ld(mk, 0x158DF5);
        st(bf, 0x5605E0, t43);
        let t44 = ld(mk, 0x158E05);
        st(bf, 0x5605A4, t44);
        let t45 = t44 - ld(mk, 0x158DED);
        st(bf, 0x5605C4, t45);
        let t46 = ld(mk, 0x158E11);
        st(bf, 0x560604, t46);
        let t47 = t46 - ld(mk, 0x158DF9);
        st(bf, 0x5605E4, t47);
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn turn_x87(mk: usize, p: usize) {
        unsafe {
            core::arch::asm!(
                    "fld tbyte ptr [{p} + 20]",
            "fld tbyte ptr [{p} + 10]",
            "fld tbyte ptr [{p}]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{mk} + 0x158E25]",
            "fst dword ptr [{mk} + 0x158F09]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x158E29]",
            "fst dword ptr [{mk} + 0x158F0D]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{mk} + 0x158E2D]",
            "fst dword ptr [{mk} + 0x158F11]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x158E49]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{mk} + 0x158E55]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{mk} + 0x158E61]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x158E31]",
            "fstp dword ptr [{mk} + 0x158EFD]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x158E4D]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{mk} + 0x158E59]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{mk} + 0x158E65]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x158E35]",
            "fstp dword ptr [{mk} + 0x158F01]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x158E51]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{mk} + 0x158E5D]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x158E69]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x158E39]",
            "fst dword ptr [{mk} + 0x158E1D]",
            "fstp dword ptr [{mk} + 0x158F05]",
            "fld dword ptr [{mk} + 0x158F0D]",
            "fsub dword ptr [{mk} + 0x157541]",
            "fld dword ptr [{mk} + 0x158F09]",
            "fsub dword ptr [{mk} + 0x15753D]",
            "fld dword ptr [{mk} + 0x158F11]",
            "fsub dword ptr [{mk} + 0x157545]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x158E49]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{mk} + 0x158E55]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{mk} + 0x158E61]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x158E31]",
            "fstp dword ptr [{mk} + 0x158F09]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x158E4D]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{mk} + 0x158E59]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{mk} + 0x158E65]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x158E35]",
            "fstp dword ptr [{mk} + 0x158F0D]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x158E51]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{mk} + 0x158E5D]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x158E69]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x158E39]",
            "fstp dword ptr [{mk} + 0x158F11]",
            mk = in(reg) mk,
            p = in(reg) p,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn project_x87(rc: usize, bf: usize, mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x158F11]",
            "fdivr dword ptr [{rc} + 0x224088]",
            "fld dword ptr [{mk} + 0x158F05]",
            "fdivr dword ptr [{rc} + 0x224088]",
            "fld dword ptr [{mk} + 0x158F09]",
            "fld dword ptr [{mk} + 0x158F0D]",
            "fld dword ptr [{mk} + 0x158EFD]",
            "fld dword ptr [{mk} + 0x158F01]",
            ".byte 0xD9, 0xCB",
            ".byte 0xD8, 0xCD",
            ".byte 0xD9, 0xCA",
            ".byte 0xD8, 0xCD",
            ".byte 0xD9, 0xC9",
            ".byte 0xD8, 0xCC",
            ".byte 0xD9, 0xCB",
            ".byte 0xD8, 0xCC",
            ".byte 0xD9, 0xCD",
            "fmul dword ptr [{mk} + 0x1610A5]",
            ".byte 0xD9, 0xCC",
            "fmul dword ptr [{mk} + 0x1610A9]",
            ".byte 0xD9, 0xCA",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x158E01]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x158E05]",
            ".byte 0xD9, 0xC9",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x158E0D]",
            ".byte 0xD9, 0xCA",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x158E11]",
            "fstp dword ptr [{mk} + 0x158E09]",
            "fstp dword ptr [{mk} + 0x158E15]",
            ".byte 0xD9, 0xE8",
            "fdiv dword ptr [{rc} + 0x2241C8]",
            "fld dword ptr [{mk} + 0x158E0D]",
            "fsub dword ptr [{mk} + 0x158E01]",
            "fst dword ptr [{mk} + 0x158DED]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x158E11]",
            "fsub dword ptr [{mk} + 0x158E05]",
            ".byte 0xD9, 0xE0",
            "fst dword ptr [{mk} + 0x158DE9]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdivr dword ptr [{mk} + 0x1610AD]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x158DE9]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x158E09]",
            "fstp dword ptr [{mk} + 0x158DE9]",
            "fmul dword ptr [{mk} + 0x158E15]",
            "fstp dword ptr [{mk} + 0x158DF5]",
            "fmul dword ptr [{mk} + 0x158DED]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x158E09]",
            "fstp dword ptr [{mk} + 0x158DED]",
            "fmul dword ptr [{mk} + 0x158E15]",
            "fstp dword ptr [{mk} + 0x158DF9]",
            "fstp dword ptr [{mk} + 0x158E21]",
            "fld dword ptr [{mk} + 0x158F11]",
            "fmul dword ptr [{mk} + 0x158E21]",
            "fst dword ptr [{bf} + 0x5605A8]",
            "fst dword ptr [{bf} + 0x5605C8]",
            "fst dword ptr [{bf} + 0x5605E8]",
            "fstp dword ptr [{bf} + 0x560608]",
            "fld dword ptr [{mk} + 0x158E01]",
            "fst dword ptr [{bf} + 0x5605A0]",
            "fsub dword ptr [{mk} + 0x158DE9]",
            "fstp dword ptr [{bf} + 0x5605C0]",
            "fld dword ptr [{mk} + 0x158E0D]",
            "fst dword ptr [{bf} + 0x560600]",
            "fsub dword ptr [{mk} + 0x158DF5]",
            "fstp dword ptr [{bf} + 0x5605E0]",
            "fld dword ptr [{mk} + 0x158E05]",
            "fst dword ptr [{bf} + 0x5605A4]",
            "fsub dword ptr [{mk} + 0x158DED]",
            "fstp dword ptr [{bf} + 0x5605C4]",
            "fld dword ptr [{mk} + 0x158E11]",
            "fst dword ptr [{bf} + 0x560604]",
            "fsub dword ptr [{mk} + 0x158DF9]",
            "fstp dword ptr [{bf} + 0x5605E4]",
            rc = in(reg) rc,
            bf = in(reg) bf,
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
    fn turn_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x15753D,
                    hi: 0x158F15,
                    reads: &[
                        (0x15753D, F32),
                        (0x157541, F32),
                        (0x157545, F32),
                        (0x158E25, F32),
                        (0x158E29, F32),
                        (0x158E2D, F32),
                        (0x158E31, I32),
                        (0x158E35, I32),
                        (0x158E39, I32),
                        (0x158E49, F32),
                        (0x158E4D, F32),
                        (0x158E51, F32),
                        (0x158E55, F32),
                        (0x158E59, F32),
                        (0x158E5D, F32),
                        (0x158E61, F32),
                        (0x158E65, F32),
                        (0x158E69, F32),
                        (0x158F09, F32),
                        (0x158F0D, F32),
                        (0x158F11, F32),
                    ],
                    writes: &[
                        (0x158E1D, F32),
                        (0x158EFD, F32),
                        (0x158F01, F32),
                        (0x158F05, F32),
                        (0x158F09, F32),
                        (0x158F0D, F32),
                        (0x158F11, F32),
                    ],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x1E,
                    reads: &[(0x0, F80), (0xA, F80), (0x14, F80)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::turn_x87(b[0], b[1]) },
            |b| unsafe { super::turn_x87(b[0], b[1]) },
        );
    }

    #[test]
    fn project_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x224088,
                    hi: 0x2241CC,
                    reads: &[
                        (0x224088, F32),
                        (0x22409C, F32),
                        (0x2240A0, F32),
                        (0x2241C8, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x5605A0,
                    hi: 0x56060C,
                    reads: &[],
                    writes: &[
                        (0x5605A0, F32),
                        (0x5605A4, F32),
                        (0x5605A8, F32),
                        (0x5605C0, F32),
                        (0x5605C4, F32),
                        (0x5605C8, F32),
                        (0x5605E0, F32),
                        (0x5605E4, F32),
                        (0x5605E8, F32),
                        (0x560600, F32),
                        (0x560604, F32),
                        (0x560608, F32),
                    ],
                },
                Op::Base {
                    lo: 0x158DE9,
                    hi: 0x1610B1,
                    reads: &[
                        (0x158DE9, F32),
                        (0x158DED, F32),
                        (0x158DF5, F32),
                        (0x158DF9, F32),
                        (0x158E01, F32),
                        (0x158E05, F32),
                        (0x158E09, F32),
                        (0x158E0D, F32),
                        (0x158E11, F32),
                        (0x158E15, F32),
                        (0x158E21, F32),
                        (0x158EFD, F32),
                        (0x158F01, F32),
                        (0x158F05, F32),
                        (0x158F09, F32),
                        (0x158F0D, F32),
                        (0x158F11, F32),
                        (0x1610A5, F32),
                        (0x1610A9, F32),
                        (0x1610AD, F32),
                    ],
                    writes: &[
                        (0x158DE9, F32),
                        (0x158DED, F32),
                        (0x158DF5, F32),
                        (0x158DF9, F32),
                        (0x158E01, F32),
                        (0x158E05, F32),
                        (0x158E09, F32),
                        (0x158E0D, F32),
                        (0x158E11, F32),
                        (0x158E15, F32),
                        (0x158E21, F32),
                    ],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::project_x87(b[0], b[1], b[2]) },
            |b| unsafe { super::project_x87(b[0], b[1], b[2]) },
        );
    }
}
