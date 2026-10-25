//! `marks::draw_square` (0x4BF80F): one square item drawn on the squares' frame. The four
//! vertices' texture places from `dword_8BC1AE` .. (u a, c, c, a; v b, b, d, d) plus the
//! item's +0xC / +0x10; its place (st(1) the across share times `flt_89CEEE`, st(0) the
//! along share times `flt_89CEF2`) along the two axes from the frame's place into
//! `flt_89CF92` ..; less `flt_89CF56` .. and `flt_89CF62` ..; the four corners (plus
//! `flt_89CF4A` .., `flt_89CF3E` .., less `flt_89CF4A` ..) into `flt_89CF92` .. `flt_89CFBE`;
//! unless the first corner's depth is at or below 4096.0 (a signed compare of the bits):
//! projected (`flt_624088` over each depth, the centre `flt_62409C` / `flt_6240A0`), depth
//! (plus `g_marksKindFloat`, times `flt_89CEEA`), colour edx, `scene::draw_fan`.
//!
//! In:  st(0) the along share, st(1) the across share (popped), esi the item, edx the
//!      colour
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BFFEC` (0x4C00FE). The x87 runs are plain Rust on the x87 model (`math::x87_float`), in the original's order, over the
//! game's variables.

use crate::framework::rva;
use crate::marks::store::{Slot, game_addr, game_read, game_write};

const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const PLACES_RVA: Slot = Slot::at(0x8BC1AE); // dword_8BC1AE .. dword_8BC1BA
const CORNERS_RVA: Slot = Slot::at(0x89CF92); // flt_89CF92 .. flt_89CFBE: four of 12
const FAR_ENOUGH: i32 = 0x4580_0000; // 4096.0

/// The original's 0x4BF845 .. 0x4BF88D: the item's texture shift added to the places.
unsafe fn shifted_places(s: u32, p: u32) {
    unsafe { shifted_places_x87(s as usize, p as usize) }
}

/// The original's 0x4BF88E .. 0x4BF9AA on the two values: the corners.
unsafe fn corners(v: &[[u8; 10]; 2]) {
    unsafe { corners_x87(crate::marks::store::base() as usize, v.as_ptr() as usize) }
}

/// The original's 0x4BF9BA .. 0x4BFAD6: the corners projected, their depth.
unsafe fn projected(p: u32) {
    unsafe {
        projected_x87(
            crate::race::store::asm_base() as usize,
            crate::marks::store::base() as usize,
            p as usize,
        )
    }
}

/// The Rust version (see the header): `item` (esi), `colour` (edx).
pub unsafe fn draw_square(item: u32, colour: u32, shares: [[u8; 10]; 2]) {
    unsafe {
        let vertices = game_addr(VERTICES_RVA) as u32;
        let [a, pb, c, d]: [u32; 4] = game_read(PLACES_RVA);
        for (n, (u, v)) in [(a, pb), (c, pb), (c, d), (a, d)].into_iter().enumerate() {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, v);
        }
        shifted_places(item, vertices);
        corners(&shares);
        if game_read::<i32>(CORNERS_RVA + 8) <= FAR_ENOUGH {
            return;
        }
        projected(vertices);
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, colour);
        }
        crate::marks::clip_quad::draw_fan();
    }
}

use crate::marks::plain::{ld, ld80, st};
use crate::math::x87_float as bh;

/// `shifted_places`'s x87 block as plain Rust on the x87 model (24 instructions), same order and memory places.
unsafe fn shifted_places_x87(s: usize, p: usize) {
    unsafe {
        let t0 = ld(s, 0xC);
        let t1 = t0 + ld(p, 0x18);
        st(p, 0x18, t1);
        let t2 = ld(s, 0xC);
        let t3 = t2 + ld(p, 0x38);
        st(p, 0x38, t3);
        let t4 = ld(s, 0xC);
        let t5 = t4 + ld(p, 0x58);
        st(p, 0x58, t5);
        let t6 = ld(s, 0xC);
        let t7 = t6 + ld(p, 0x78);
        st(p, 0x78, t7);
        let t8 = ld(s, 0x10);
        let t9 = t8 + ld(p, 0x1C);
        st(p, 0x1C, t9);
        let t10 = ld(s, 0x10);
        let t11 = t10 + ld(p, 0x3C);
        st(p, 0x3C, t11);
        let t12 = ld(s, 0x10);
        let t13 = t12 + ld(p, 0x5C);
        st(p, 0x5C, t13);
        let t14 = ld(s, 0x10);
        let t15 = t14 + ld(p, 0x7C);
        st(p, 0x7C, t15);
    }
}

/// `corners`'s x87 block as plain Rust on the x87 model (56 instructions), same order and memory places.
unsafe fn corners_x87(mk: usize, v: usize) {
    unsafe {
        let t0 = bh::from_f80(ld80(v, 0xA));
        let t1 = bh::from_f80(ld80(v, 0));
        let t2 = t1 * ld(mk, 0x137FB2);
        let t3 = t0 * ld(mk, 0x137FAE);
        let t4 = t3 * ld(mk, 0x137FDA);
        let t5 = t2 * ld(mk, 0x137FE6);
        let t6 = t4 + t5;
        let t7 = t6 + ld(mk, 0x137FB6);
        st(mk, 0x138052, t7);
        let t8 = t3 * ld(mk, 0x137FDE);
        let t9 = t2 * ld(mk, 0x137FEA);
        let t10 = t8 + t9;
        let t11 = t10 + ld(mk, 0x137FBA);
        st(mk, 0x138056, t11);
        let t12 = t2 * ld(mk, 0x137FEE);
        let t13 = t3 * ld(mk, 0x137FE2);
        let t14 = t12 + t13;
        let t15 = t14 + ld(mk, 0x137FBE);
        st(mk, 0x13805A, t15);
        let t16 = ld(mk, 0x138052);
        let t17 = t16 - ld(mk, 0x138016);
        let t18 = t17 - ld(mk, 0x138022);
        st(mk, 0x138052, t18);
        let t19 = t18 + ld(mk, 0x13800A);
        st(mk, 0x13805E, t19);
        let t20 = t19 + ld(mk, 0x137FFE);
        st(mk, 0x13806A, t20);
        let t21 = t20 - ld(mk, 0x13800A);
        st(mk, 0x138076, t21);
        let t22 = ld(mk, 0x138056);
        let t23 = t22 - ld(mk, 0x13801A);
        let t24 = t23 - ld(mk, 0x138026);
        st(mk, 0x138056, t24);
        let t25 = t24 + ld(mk, 0x13800E);
        st(mk, 0x138062, t25);
        let t26 = t25 + ld(mk, 0x138002);
        st(mk, 0x13806E, t26);
        let t27 = t26 - ld(mk, 0x13800E);
        st(mk, 0x13807A, t27);
        let t28 = ld(mk, 0x13805A);
        let t29 = t28 - ld(mk, 0x13801E);
        let t30 = t29 - ld(mk, 0x13802A);
        st(mk, 0x13805A, t30);
        let t31 = t30 + ld(mk, 0x138012);
        st(mk, 0x138066, t31);
        let t32 = t31 + ld(mk, 0x138006);
        st(mk, 0x138072, t32);
        let t33 = t32 - ld(mk, 0x138012);
        st(mk, 0x13807E, t33);
    }
}

/// `projected`'s x87 block as plain Rust on the x87 model (64 instructions), same order and memory places.
unsafe fn projected_x87(rc: usize, mk: usize, p: usize) {
    unsafe {
        let t0 = ld(rc, 0x224088);
        let t1 = t0 / ld(mk, 0x13807E);
        let t2 = ld(rc, 0x224088);
        let t3 = t2 / ld(mk, 0x138072);
        let t4 = ld(rc, 0x224088);
        let t5 = t4 / ld(mk, 0x138066);
        let t6 = ld(rc, 0x224088);
        let t7 = t6 / ld(mk, 0x13805A);
        let t8 = ld(mk, 0x138056);
        let t9 = t8 * t7;
        let t10 = t7 * ld(mk, 0x138052);
        let t11 = -t9;
        let t12 = t11 + ld(rc, 0x2240A0);
        st(p, 0x4, t12);
        let t13 = t10 + ld(rc, 0x22409C);
        st(p, 0, t13);
        let t14 = ld(mk, 0x138062);
        let t15 = t14 * t5;
        let t16 = t5 * ld(mk, 0x13805E);
        let t17 = -t15;
        let t18 = t17 + ld(rc, 0x2240A0);
        st(p, 0x24, t18);
        let t19 = t16 + ld(rc, 0x22409C);
        st(p, 0x20, t19);
        let t20 = ld(mk, 0x13806E);
        let t21 = t20 * t3;
        let t22 = t3 * ld(mk, 0x13806A);
        let t23 = -t21;
        let t24 = t23 + ld(rc, 0x2240A0);
        st(p, 0x44, t24);
        let t25 = t22 + ld(rc, 0x22409C);
        st(p, 0x40, t25);
        let t26 = ld(mk, 0x13807A);
        let t27 = t26 * t1;
        let t28 = t1 * ld(mk, 0x138076);
        let t29 = -t27;
        let t30 = t29 + ld(rc, 0x2240A0);
        st(p, 0x64, t30);
        let t31 = t28 + ld(rc, 0x22409C);
        st(p, 0x60, t31);
        let t32 = ld(mk, 0x13805A);
        let t33 = t32 + ld(mk, 0x35C);
        let t34 = t33 * ld(mk, 0x137FAA);
        let t35 = ld(mk, 0x138066);
        let t36 = t35 + ld(mk, 0x35C);
        let t37 = t36 * ld(mk, 0x137FAA);
        let t38 = ld(mk, 0x138072);
        let t39 = t38 + ld(mk, 0x35C);
        let t40 = t39 * ld(mk, 0x137FAA);
        let t41 = ld(mk, 0x13807E);
        let t42 = t41 + ld(mk, 0x35C);
        let t43 = t42 * ld(mk, 0x137FAA);
        st(p, 0x68, t43);
        st(p, 0x48, t40);
        st(p, 0x28, t37);
        st(p, 0x8, t34);
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn shifted_places_x87(s: usize, p: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{s} + 0xC]",
            "fadd dword ptr [{p} + 0x18]",
            "fstp dword ptr [{p} + 0x18]",
            "fld dword ptr [{s} + 0xC]",
            "fadd dword ptr [{p} + 0x38]",
            "fstp dword ptr [{p} + 0x38]",
            "fld dword ptr [{s} + 0xC]",
            "fadd dword ptr [{p} + 0x58]",
            "fstp dword ptr [{p} + 0x58]",
            "fld dword ptr [{s} + 0xC]",
            "fadd dword ptr [{p} + 0x78]",
            "fstp dword ptr [{p} + 0x78]",
            "fld dword ptr [{s} + 0x10]",
            "fadd dword ptr [{p} + 0x1C]",
            "fstp dword ptr [{p} + 0x1C]",
            "fld dword ptr [{s} + 0x10]",
            "fadd dword ptr [{p} + 0x3C]",
            "fstp dword ptr [{p} + 0x3C]",
            "fld dword ptr [{s} + 0x10]",
            "fadd dword ptr [{p} + 0x5C]",
            "fstp dword ptr [{p} + 0x5C]",
            "fld dword ptr [{s} + 0x10]",
            "fadd dword ptr [{p} + 0x7C]",
            "fstp dword ptr [{p} + 0x7C]",
            s = in(reg) s,
            p = in(reg) p,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn corners_x87(mk: usize, v: usize) {
        unsafe {
            core::arch::asm!(
                    "fld tbyte ptr [{v} + 10]",
            "fld tbyte ptr [{v}]",
            "fmul dword ptr [{mk} + 0x137FB2]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x137FAE]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x137FDA]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x137FE6]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{mk} + 0x137FB6]",
            "fstp dword ptr [{mk} + 0x138052]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x137FDE]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x137FEA]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{mk} + 0x137FBA]",
            "fstp dword ptr [{mk} + 0x138056]",
            "fmul dword ptr [{mk} + 0x137FEE]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x137FE2]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{mk} + 0x137FBE]",
            "fstp dword ptr [{mk} + 0x13805A]",
            "fld dword ptr [{mk} + 0x138052]",
            "fsub dword ptr [{mk} + 0x138016]",
            "fsub dword ptr [{mk} + 0x138022]",
            "fst dword ptr [{mk} + 0x138052]",
            "fadd dword ptr [{mk} + 0x13800A]",
            "fst dword ptr [{mk} + 0x13805E]",
            "fadd dword ptr [{mk} + 0x137FFE]",
            "fst dword ptr [{mk} + 0x13806A]",
            "fsub dword ptr [{mk} + 0x13800A]",
            "fstp dword ptr [{mk} + 0x138076]",
            "fld dword ptr [{mk} + 0x138056]",
            "fsub dword ptr [{mk} + 0x13801A]",
            "fsub dword ptr [{mk} + 0x138026]",
            "fst dword ptr [{mk} + 0x138056]",
            "fadd dword ptr [{mk} + 0x13800E]",
            "fst dword ptr [{mk} + 0x138062]",
            "fadd dword ptr [{mk} + 0x138002]",
            "fst dword ptr [{mk} + 0x13806E]",
            "fsub dword ptr [{mk} + 0x13800E]",
            "fstp dword ptr [{mk} + 0x13807A]",
            "fld dword ptr [{mk} + 0x13805A]",
            "fsub dword ptr [{mk} + 0x13801E]",
            "fsub dword ptr [{mk} + 0x13802A]",
            "fst dword ptr [{mk} + 0x13805A]",
            "fadd dword ptr [{mk} + 0x138012]",
            "fst dword ptr [{mk} + 0x138066]",
            "fadd dword ptr [{mk} + 0x138006]",
            "fst dword ptr [{mk} + 0x138072]",
            "fsub dword ptr [{mk} + 0x138012]",
            "fstp dword ptr [{mk} + 0x13807E]",
            mk = in(reg) mk,
            v = in(reg) v,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn projected_x87(rc: usize, mk: usize, p: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x13807E]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x138072]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x138066]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x13805A]",
            "fld dword ptr [{mk} + 0x138056]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x138052]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x4]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{p} + 0x0]",
            "fld dword ptr [{mk} + 0x138062]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x13805E]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x24]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{p} + 0x20]",
            "fld dword ptr [{mk} + 0x13806E]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x13806A]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x44]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{p} + 0x40]",
            "fld dword ptr [{mk} + 0x13807A]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x138076]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x64]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{p} + 0x60]",
            "fld dword ptr [{mk} + 0x13805A]",
            "fadd dword ptr [{mk} + 0x35C]",
            "fmul dword ptr [{mk} + 0x137FAA]",
            "fld dword ptr [{mk} + 0x138066]",
            "fadd dword ptr [{mk} + 0x35C]",
            "fmul dword ptr [{mk} + 0x137FAA]",
            "fld dword ptr [{mk} + 0x138072]",
            "fadd dword ptr [{mk} + 0x35C]",
            "fmul dword ptr [{mk} + 0x137FAA]",
            "fld dword ptr [{mk} + 0x13807E]",
            "fadd dword ptr [{mk} + 0x35C]",
            "fmul dword ptr [{mk} + 0x137FAA]",
            "fstp dword ptr [{p} + 0x68]",
            "fstp dword ptr [{p} + 0x48]",
            "fstp dword ptr [{p} + 0x28]",
            "fstp dword ptr [{p} + 0x8]",
            rc = in(reg) rc,
            mk = in(reg) mk,
            p = in(reg) p,
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
    fn shifted_places_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xC,
                    hi: 0x14,
                    reads: &[(0xC, F32), (0x10, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x18,
                    hi: 0x80,
                    reads: &[
                        (0x18, F32),
                        (0x1C, F32),
                        (0x38, F32),
                        (0x3C, F32),
                        (0x58, F32),
                        (0x5C, F32),
                        (0x78, F32),
                        (0x7C, F32),
                    ],
                    writes: &[
                        (0x18, F32),
                        (0x1C, F32),
                        (0x38, F32),
                        (0x3C, F32),
                        (0x58, F32),
                        (0x5C, F32),
                        (0x78, F32),
                        (0x7C, F32),
                    ],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::shifted_places_x87(b[0], b[1]) },
            |b| unsafe { super::shifted_places_x87(b[0], b[1]) },
        );
    }

    #[test]
    fn corners_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x137FAE,
                    hi: 0x138082,
                    reads: &[
                        (0x137FAE, F32),
                        (0x137FB2, F32),
                        (0x137FB6, F32),
                        (0x137FBA, F32),
                        (0x137FBE, F32),
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
                        (0x138016, F32),
                        (0x13801A, F32),
                        (0x13801E, F32),
                        (0x138022, F32),
                        (0x138026, F32),
                        (0x13802A, F32),
                        (0x138052, F32),
                        (0x138056, F32),
                        (0x13805A, F32),
                    ],
                    writes: &[
                        (0x138052, F32),
                        (0x138056, F32),
                        (0x13805A, F32),
                        (0x13805E, F32),
                        (0x138062, F32),
                        (0x138066, F32),
                        (0x13806A, F32),
                        (0x13806E, F32),
                        (0x138072, F32),
                        (0x138076, F32),
                        (0x13807A, F32),
                        (0x13807E, F32),
                    ],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x14,
                    reads: &[(0x0, F80), (0xA, F80)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::corners_x87(b[0], b[1]) },
            |b| unsafe { super::corners_x87(b[0], b[1]) },
        );
    }

    #[test]
    fn projected_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x224088,
                    hi: 0x2240A4,
                    reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x35C,
                    hi: 0x138082,
                    reads: &[
                        (0x35C, F32),
                        (0x137FAA, F32),
                        (0x138052, F32),
                        (0x138056, F32),
                        (0x13805A, F32),
                        (0x13805E, F32),
                        (0x138062, F32),
                        (0x138066, F32),
                        (0x13806A, F32),
                        (0x13806E, F32),
                        (0x138072, F32),
                        (0x138076, F32),
                        (0x13807A, F32),
                        (0x13807E, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x6C,
                    reads: &[],
                    writes: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0x20, F32),
                        (0x24, F32),
                        (0x28, F32),
                        (0x40, F32),
                        (0x44, F32),
                        (0x48, F32),
                        (0x60, F32),
                        (0x64, F32),
                        (0x68, F32),
                    ],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::projected_x87(b[0], b[1], b[2]) },
            |b| unsafe { super::projected_x87(b[0], b[1], b[2]) },
        );
    }
}
