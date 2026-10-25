//! `marks::draw_ribbon` (0x4C5E63): a ribbon piece between two places already in the
//! view's frame (`flt_8EB8D5` .. and `dword_8EB8E1` ..): half the width (edx >> 1, whole)
//! in `dword_8EB8B5`, the four vertices' colour (ecx << 24) | esi and texture places from
//! cell eax of `g_marksCellPlaces`; unless either depth is at or below `dword_8EB961` (a
//! signed compare of the bits): both projected (`flt_8EB8CD`, `flt_8EB8D1` their scales),
//! the way between them made one long and turned a quarter (`flt_8F3985`, `flt_8F3989`),
//! the sides that times the half width and each scale (`flt_8F3991` ..), the four corners,
//! the depths times `flt_8EB96D`; `marks::clip_quad`, and unless all is outside
//! `scene::draw_fan` and `g_polygonVertexCount` 4.
//!
//! In:  eax the cell, ecx the alpha, edx the width, esi the colour (and the two places)
//! Out: nothing (its caller reads no register after it)
//!
//! Callers: the marks' lines (0x4C5D65) and 0x4CFAA3. The x87 block is plain Rust on the x87 model (`math::x87_float`), in the original's order, over the game's variables. Refuses, before anything, a cell past 0x40 and
//! null marks settings (the clip's refusal).

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::clip_quad::{clip_quad, draw_fan};
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::scene::draw_store::DRAW;

const HALF_RVA: Slot = Slot::at(0x8EB8B5); // dword_8EB8B5
const DEPTHS_RVA: [Slot; 2] = [Slot::at(0x8EB8DD), Slot::at(0x8EB8E9)];
const NEAR_RVA: Slot = Slot::at(0x8EB961); // dword_8EB961
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const CELLS_RVA: Slot = Slot::at(0x935766); // g_marksCellPlaces: 0x41 of 16 bytes
const CELLS: u32 = 0x41;

/// The original's 0x4C5EF7 .. 0x4C60E9: the two projected and the ribbon's corners.
unsafe fn ribbon() {
    unsafe {
        ribbon_x87(
            crate::race::store::asm_base() as usize,
            crate::race::store::buffers_asm_base() as usize,
            crate::marks::store::base() as usize,
        )
    }
}

/// The Rust version (see the header): `cell` (eax), `alpha` (ecx), `width` (edx),
/// `colour` (esi). Its one caller reads no register after it.
pub unsafe fn draw_ribbon(cell: u32, alpha: u32, width: u32, colour: u32) -> Result<(), Refused> {
    if cell >= CELLS {
        return Err(BadInput("a mark cell past g_marksCellPlaces").into());
    }
    unsafe {
        if crate::marks::setup::SETTINGS.pointer.get() == 0 {
            return Err(BadInput("no marks settings").into());
        }
        game_write(HALF_RVA, width >> 1);
        let colour = (alpha << 24) | colour;
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, colour);
        }
        let at = cell << 4;
        let place = |n: usize| game_read::<u32>(CELLS_RVA + at as usize + 4 * n);
        let [u0, v0, u1, v1] = [place(0), place(1), place(2), place(3)];
        for (n, (u, v)) in [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
            .into_iter()
            .enumerate()
        {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, v);
        }
        for r in DEPTHS_RVA {
            if game_read::<i32>(r) <= game_read::<i32>(NEAR_RVA) {
                return Ok(());
            }
        }
        ribbon();
        if clip_quad(game_addr(VERTICES_RVA) as u32)? {
            return Ok(());
        }
        draw_fan();
        DRAW.polygon_vertex_count.set_bits(4u32);
    }
    Ok(())
}

use crate::marks::plain::{ld, ldi, st};
use crate::math::x87_float as bh;

/// `ribbon`'s x87 block as plain Rust on the x87 model (93 instructions), same order and memory places.
unsafe fn ribbon_x87(rc: usize, bf: usize, mk: usize) {
    unsafe {
        let t0 = ld(rc, 0x224088);
        let t1 = t0 / ld(mk, 0x18699D);
        let t2 = t1 * ld(mk, 0x186995);
        let t3 = t1 * ld(mk, 0x186999);
        let t4 = -t3;
        let t5 = t4 + ld(rc, 0x2240A0);
        st(mk, 0x186999, t5);
        let t6 = t2 + ld(rc, 0x22409C);
        st(mk, 0x186995, t6);
        st(mk, 0x18698D, t1);
        let t7 = ld(rc, 0x224088);
        let t8 = t7 / ld(mk, 0x1869A9);
        let t9 = t8 * ld(mk, 0x1869A1);
        let t10 = t8 * ld(mk, 0x1869A5);
        let t11 = -t10;
        let t12 = t11 + ld(rc, 0x2240A0);
        st(mk, 0x1869A5, t12);
        let t13 = t9 + ld(rc, 0x22409C);
        st(mk, 0x1869A1, t13);
        st(mk, 0x186991, t8);
        let t14 = ld(mk, 0x1869A1);
        let t15 = t14 - ld(mk, 0x186995);
        st(mk, 0x18EA45, t15);
        let t16 = ld(mk, 0x1869A5);
        let t17 = t16 - ld(mk, 0x186999);
        st(mk, 0x18EA49, t17);
        let t18 = ld(mk, 0x18EA45);
        let t19 = t18 * t18;
        let t20 = ld(mk, 0x18EA49);
        let t21 = t20 * t20;
        let t22 = t19 + t21;
        let t23 = t22.sqrt();
        let t24 = bh::ONE;
        let t25 = t24 / t23;
        let t26 = ld(mk, 0x18EA45);
        let t27 = ld(mk, 0x18EA49);
        let t28 = t27 * t25;
        st(mk, 0x18EA45, t28);
        let t29 = t25 * t26;
        let t30 = -t29;
        st(mk, 0x18EA49, t30);
        let t31 = ld(mk, 0x18EA45);
        let t32 = t31 * ldi(mk, 0x186975);
        let t33 = t32 * ld(mk, 0x18698D);
        st(mk, 0x18EA51, t33);
        let t34 = ld(mk, 0x18EA49);
        let t35 = t34 * ldi(mk, 0x186975);
        let t36 = t35 * ld(mk, 0x18698D);
        st(mk, 0x18EA55, t36);
        let t37 = ld(mk, 0x18EA45);
        let t38 = t37 * ldi(mk, 0x186975);
        let t39 = t38 * ld(mk, 0x186991);
        st(mk, 0x18EA5D, t39);
        let t40 = ld(mk, 0x18EA49);
        let t41 = t40 * ldi(mk, 0x186975);
        let t42 = t41 * ld(mk, 0x186991);
        st(mk, 0x18EA61, t42);
        let t43 = ld(mk, 0x186995);
        let t44 = t43 - ld(mk, 0x18EA51);
        st(bf, 0x5605A0, t44);
        let t45 = ld(mk, 0x186999);
        let t46 = t45 - ld(mk, 0x18EA55);
        st(bf, 0x5605A4, t46);
        let t47 = ld(mk, 0x186995);
        let t48 = t47 + ld(mk, 0x18EA51);
        st(bf, 0x5605C0, t48);
        let t49 = ld(mk, 0x186999);
        let t50 = t49 + ld(mk, 0x18EA55);
        st(bf, 0x5605C4, t50);
        let t51 = ld(mk, 0x1869A1);
        let t52 = t51 + ld(mk, 0x18EA5D);
        st(bf, 0x5605E0, t52);
        let t53 = ld(mk, 0x1869A5);
        let t54 = t53 + ld(mk, 0x18EA61);
        st(bf, 0x5605E4, t54);
        let t55 = ld(mk, 0x1869A1);
        let t56 = t55 - ld(mk, 0x18EA5D);
        st(bf, 0x560600, t56);
        let t57 = ld(mk, 0x1869A5);
        let t58 = t57 - ld(mk, 0x18EA61);
        st(bf, 0x560604, t58);
        let t59 = ld(mk, 0x18699D);
        let t60 = t59 * ld(mk, 0x186A2D);
        st(bf, 0x5605A8, t60);
        st(bf, 0x5605C8, t60);
        let t61 = ld(mk, 0x1869A9);
        let t62 = t61 * ld(mk, 0x186A2D);
        st(bf, 0x5605E8, t62);
        st(bf, 0x560608, t62);
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn ribbon_x87(rc: usize, bf: usize, mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x18699D]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x186995]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x186999]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x186999]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x186995]",
            "fstp dword ptr [{mk} + 0x18698D]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1869A9]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1869A1]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x1869A5]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1869A5]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1869A1]",
            "fstp dword ptr [{mk} + 0x186991]",
            "fld dword ptr [{mk} + 0x1869A1]",
            "fsub dword ptr [{mk} + 0x186995]",
            "fstp dword ptr [{mk} + 0x18EA45]",
            "fld dword ptr [{mk} + 0x1869A5]",
            "fsub dword ptr [{mk} + 0x186999]",
            "fstp dword ptr [{mk} + 0x18EA49]",
            "fld dword ptr [{mk} + 0x18EA45]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x18EA49]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            ".byte 0xD9, 0xE8",
            ".byte 0xDE, 0xF1",
            "fld dword ptr [{mk} + 0x18EA45]",
            "fld dword ptr [{mk} + 0x18EA49]",
            ".byte 0xD8, 0xCA",
            "fstp dword ptr [{mk} + 0x18EA45]",
            ".byte 0xDE, 0xC9",
            ".byte 0xD9, 0xE0",
            "fstp dword ptr [{mk} + 0x18EA49]",
            "fld dword ptr [{mk} + 0x18EA45]",
            "fimul dword ptr [{mk} + 0x186975]",
            "fmul dword ptr [{mk} + 0x18698D]",
            "fstp dword ptr [{mk} + 0x18EA51]",
            "fld dword ptr [{mk} + 0x18EA49]",
            "fimul dword ptr [{mk} + 0x186975]",
            "fmul dword ptr [{mk} + 0x18698D]",
            "fstp dword ptr [{mk} + 0x18EA55]",
            "fld dword ptr [{mk} + 0x18EA45]",
            "fimul dword ptr [{mk} + 0x186975]",
            "fmul dword ptr [{mk} + 0x186991]",
            "fstp dword ptr [{mk} + 0x18EA5D]",
            "fld dword ptr [{mk} + 0x18EA49]",
            "fimul dword ptr [{mk} + 0x186975]",
            "fmul dword ptr [{mk} + 0x186991]",
            "fstp dword ptr [{mk} + 0x18EA61]",
            "fld dword ptr [{mk} + 0x186995]",
            "fsub dword ptr [{mk} + 0x18EA51]",
            "fstp dword ptr [{bf} + 0x5605A0]",
            "fld dword ptr [{mk} + 0x186999]",
            "fsub dword ptr [{mk} + 0x18EA55]",
            "fstp dword ptr [{bf} + 0x5605A4]",
            "fld dword ptr [{mk} + 0x186995]",
            "fadd dword ptr [{mk} + 0x18EA51]",
            "fstp dword ptr [{bf} + 0x5605C0]",
            "fld dword ptr [{mk} + 0x186999]",
            "fadd dword ptr [{mk} + 0x18EA55]",
            "fstp dword ptr [{bf} + 0x5605C4]",
            "fld dword ptr [{mk} + 0x1869A1]",
            "fadd dword ptr [{mk} + 0x18EA5D]",
            "fstp dword ptr [{bf} + 0x5605E0]",
            "fld dword ptr [{mk} + 0x1869A5]",
            "fadd dword ptr [{mk} + 0x18EA61]",
            "fstp dword ptr [{bf} + 0x5605E4]",
            "fld dword ptr [{mk} + 0x1869A1]",
            "fsub dword ptr [{mk} + 0x18EA5D]",
            "fstp dword ptr [{bf} + 0x560600]",
            "fld dword ptr [{mk} + 0x1869A5]",
            "fsub dword ptr [{mk} + 0x18EA61]",
            "fstp dword ptr [{bf} + 0x560604]",
            "fld dword ptr [{mk} + 0x18699D]",
            "fmul dword ptr [{mk} + 0x186A2D]",
            "fst dword ptr [{bf} + 0x5605A8]",
            "fstp dword ptr [{bf} + 0x5605C8]",
            "fld dword ptr [{mk} + 0x1869A9]",
            "fmul dword ptr [{mk} + 0x186A2D]",
            "fst dword ptr [{bf} + 0x5605E8]",
            "fstp dword ptr [{bf} + 0x560608]",
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
    fn ribbon_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x224088,
                    hi: 0x2240A4,
                    reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
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
                    lo: 0x186975,
                    hi: 0x18EA65,
                    reads: &[
                        (0x186975, I32),
                        (0x18698D, F32),
                        (0x186991, F32),
                        (0x186995, F32),
                        (0x186999, F32),
                        (0x18699D, F32),
                        (0x1869A1, F32),
                        (0x1869A5, F32),
                        (0x1869A9, F32),
                        (0x186A2D, F32),
                        (0x18EA45, F32),
                        (0x18EA49, F32),
                        (0x18EA51, F32),
                        (0x18EA55, F32),
                        (0x18EA5D, F32),
                        (0x18EA61, F32),
                    ],
                    writes: &[
                        (0x18698D, F32),
                        (0x186991, F32),
                        (0x186995, F32),
                        (0x186999, F32),
                        (0x1869A1, F32),
                        (0x1869A5, F32),
                        (0x18EA45, F32),
                        (0x18EA49, F32),
                        (0x18EA51, F32),
                        (0x18EA55, F32),
                        (0x18EA5D, F32),
                        (0x18EA61, F32),
                    ],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::ribbon_x87(b[0], b[1], b[2]) },
            |b| unsafe { super::ribbon_x87(b[0], b[1], b[2]) },
        );
    }
}
