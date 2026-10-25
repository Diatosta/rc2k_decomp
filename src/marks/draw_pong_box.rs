//! `marks::draw_pong_box` (0x4B8599): one box of the pong game (ball, bat or a score
//! cell) drawn on the squares' frame: its height (st(0)) along the second axis times
//! `flt_89CEF2` into `flt_768664` .. (and times `flt_765158` into `flt_76867C` ..), its
//! width (st(1)) along the first times `flt_89CEEE` into `flt_768658` .. (`flt_768670` ..);
//! the texture places from `dword_76920A` ..; its place (st(2) along, st(3) across) on the
//! frame into `flt_89CF92` .., the four corners; unless the first corner's depth is below
//! `flt_76921A`: projected, depth times `flt_89CEEA`, colour -1, `scene::draw_fan`.
//!
//! In:  st(0) height, st(1) width, st(2) along, st(3) across (popped)
//! Out: nothing (its caller reads no register after it)
//!
//! Callers: `marks::run_pong` (ball, bat), `marks::pong_cell`. The x87 runs are plain Rust on the x87 model
//! (`math::x87_float`), in the original's order, over the game's variables.

use crate::framework::rva;
use crate::marks::pong::{PONG, PONG_RULES, PongState};
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87;

const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const CORNERS_RVA: Slot = Slot::at(0x89CF92); // flt_89CF92 .. flt_89CFBE

/// The original's 0x4B8599 .. 0x4B8659 on the four values: the sides; the place's two
/// values (left untouched) popped after.
unsafe fn sides(v: &[[u8; 10]; 4]) {
    unsafe {
        sides_x87(
            crate::marks::store::base() as usize,
            v.as_ptr() as usize,
            &PONG,
        )
    }
}

/// The original's 0x4B868E .. 0x4B8786 on the place: the corners.
unsafe fn corners(v: &[[u8; 10]; 4]) {
    unsafe {
        corners_x87(
            crate::marks::store::base() as usize,
            v.as_ptr() as usize,
            &PONG,
        )
    }
}

/// The original's 0x4B87A5 .. 0x4B88A9: the corners projected, their depth.
unsafe fn projected(p: u32) {
    unsafe {
        projected_x87(
            crate::race::store::asm_base() as usize,
            crate::marks::store::base() as usize,
            p as usize,
        )
    }
}

/// The Rust version (see the header).
pub unsafe fn draw_pong_box(v: [[u8; 10]; 4]) {
    unsafe {
        sides(&v);
        let [pa, pb, pc, pd] = PONG_RULES.box_places.each_ref().map(|f| f.bits());
        for (n, (u, w)) in [(pa, pb), (pc, pb), (pc, pd), (pa, pd)]
            .into_iter()
            .enumerate()
        {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, w);
        }
        corners(&v);
        if x87::compare_status(game_read(CORNERS_RVA + 8), PONG_RULES.near.bits()) == 0x100 {
            return;
        }
        projected(game_addr(VERTICES_RVA) as u32);
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, 0xFFFF_FFFFu32);
        }
        crate::marks::clip_quad::draw_fan();
    }
}

use crate::marks::plain::{ld, ld80, st};
use crate::math::x87_float as bh;

/// `sides`'s x87 block as plain Rust on the x87 model (42 instructions), same order and memory places.
unsafe fn sides_x87(mk: usize, v: usize, p: &PongState) {
    unsafe {
        let _t0 = bh::from_f80(ld80(v, 0x1E));
        let _t1 = bh::from_f80(ld80(v, 0x14));
        let t2 = bh::from_f80(ld80(v, 0xA));
        let t3 = bh::from_f80(ld80(v, 0));
        let t4 = ld(mk, 0x137FE6);
        let t5 = t4 * t3;
        let t6 = t5 * ld(mk, 0x137FB2);
        p.box_sides[3].set_bits(bh::st(t6));
        let t7 = t6 * ld(mk, 0x218);
        p.box_sides[9].set_bits(bh::st(t7));
        let t8 = ld(mk, 0x137FEA);
        let t9 = t8 * t3;
        let t10 = t9 * ld(mk, 0x137FB2);
        p.box_sides[4].set_bits(bh::st(t10));
        let t11 = t10 * ld(mk, 0x218);
        p.box_sides[10].set_bits(bh::st(t11));
        let t12 = ld(mk, 0x137FEE);
        let t13 = t3 * t12;
        let t14 = t13 * ld(mk, 0x137FB2);
        p.box_sides[5].set_bits(bh::st(t14));
        let t15 = t14 * ld(mk, 0x218);
        p.box_sides[11].set_bits(bh::st(t15));
        let t16 = ld(mk, 0x137FDA);
        let t17 = t16 * t2;
        let t18 = t17 * ld(mk, 0x137FAE);
        p.box_sides[0].set_bits(bh::st(t18));
        let t19 = t18 * ld(mk, 0x218);
        p.box_sides[6].set_bits(bh::st(t19));
        let t20 = ld(mk, 0x137FDE);
        let t21 = t20 * t2;
        let t22 = t21 * ld(mk, 0x137FAE);
        p.box_sides[1].set_bits(bh::st(t22));
        let t23 = t22 * ld(mk, 0x218);
        p.box_sides[7].set_bits(bh::st(t23));
        let t24 = ld(mk, 0x137FE2);
        let t25 = t2 * t24;
        let t26 = t25 * ld(mk, 0x137FAE);
        p.box_sides[2].set_bits(bh::st(t26));
        let t27 = t26 * ld(mk, 0x218);
        p.box_sides[8].set_bits(bh::st(t27));
    }
}

/// `corners`'s x87 block as plain Rust on the x87 model (50 instructions), same order and memory places.
unsafe fn corners_x87(mk: usize, v: usize, p: &PongState) {
    unsafe {
        let t0 = bh::from_f80(ld80(v, 0x1E));
        let t1 = bh::from_f80(ld80(v, 0x14));
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
        st(mk, 0x138052, t16);
        let t17 = t16 + bh::ld(p.box_sides[3].bits());
        st(mk, 0x13805E, t17);
        let t18 = t17 + bh::ld(p.box_sides[0].bits());
        st(mk, 0x13806A, t18);
        let t19 = t18 - bh::ld(p.box_sides[3].bits());
        st(mk, 0x138076, t19);
        let t20 = ld(mk, 0x138056);
        st(mk, 0x138056, t20);
        let t21 = t20 + bh::ld(p.box_sides[4].bits());
        st(mk, 0x138062, t21);
        let t22 = t21 + bh::ld(p.box_sides[1].bits());
        st(mk, 0x13806E, t22);
        let t23 = t22 - bh::ld(p.box_sides[4].bits());
        st(mk, 0x13807A, t23);
        let t24 = ld(mk, 0x13805A);
        st(mk, 0x13805A, t24);
        let t25 = t24 + bh::ld(p.box_sides[5].bits());
        st(mk, 0x138066, t25);
        let t26 = t25 + bh::ld(p.box_sides[2].bits());
        st(mk, 0x138072, t26);
        let t27 = t26 - bh::ld(p.box_sides[5].bits());
        st(mk, 0x13807E, t27);
    }
}

/// `projected`'s x87 block as plain Rust on the x87 model (60 instructions), same order and memory places.
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
        let t33 = t32 * ld(mk, 0x137FAA);
        let t34 = ld(mk, 0x138066);
        let t35 = t34 * ld(mk, 0x137FAA);
        let t36 = ld(mk, 0x138072);
        let t37 = t36 * ld(mk, 0x137FAA);
        let t38 = ld(mk, 0x13807E);
        let t39 = t38 * ld(mk, 0x137FAA);
        st(p, 0x68, t39);
        st(p, 0x48, t37);
        st(p, 0x28, t35);
        st(p, 0x8, t33);
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn sides_x87(mk: usize, v: usize) {
        unsafe {
            core::arch::asm!(
                    "fld tbyte ptr [{v} + 30]",
            "fld tbyte ptr [{v} + 20]",
            "fld tbyte ptr [{v} + 10]",
            "fld tbyte ptr [{v}]",
            "fld dword ptr [{mk} + 0x137FE6]",
            ".byte 0xD8, 0xC9",
            "fmul dword ptr [{mk} + 0x137FB2]",
            "fst dword ptr [{mk} + 0x3724]",
            "fmul dword ptr [{mk} + 0x218]",
            "fstp dword ptr [{mk} + 0x373C]",
            "fld dword ptr [{mk} + 0x137FEA]",
            ".byte 0xD8, 0xC9",
            "fmul dword ptr [{mk} + 0x137FB2]",
            "fst dword ptr [{mk} + 0x3728]",
            "fmul dword ptr [{mk} + 0x218]",
            "fstp dword ptr [{mk} + 0x3740]",
            "fld dword ptr [{mk} + 0x137FEE]",
            ".byte 0xDE, 0xC9",
            "fmul dword ptr [{mk} + 0x137FB2]",
            "fst dword ptr [{mk} + 0x372C]",
            "fmul dword ptr [{mk} + 0x218]",
            "fstp dword ptr [{mk} + 0x3744]",
            "fld dword ptr [{mk} + 0x137FDA]",
            ".byte 0xD8, 0xC9",
            "fmul dword ptr [{mk} + 0x137FAE]",
            "fst dword ptr [{mk} + 0x3718]",
            "fmul dword ptr [{mk} + 0x218]",
            "fstp dword ptr [{mk} + 0x3730]",
            "fld dword ptr [{mk} + 0x137FDE]",
            ".byte 0xD8, 0xC9",
            "fmul dword ptr [{mk} + 0x137FAE]",
            "fst dword ptr [{mk} + 0x371C]",
            "fmul dword ptr [{mk} + 0x218]",
            "fstp dword ptr [{mk} + 0x3734]",
            "fld dword ptr [{mk} + 0x137FE2]",
            ".byte 0xDE, 0xC9",
            "fmul dword ptr [{mk} + 0x137FAE]",
            "fst dword ptr [{mk} + 0x3720]",
            "fmul dword ptr [{mk} + 0x218]",
            "fstp dword ptr [{mk} + 0x3738]",
            "fstp st(0)",
            "fstp st(0)",
            mk = in(reg) mk,
            v = in(reg) v,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn corners_x87(mk: usize, v: usize) {
        unsafe {
            core::arch::asm!(
                    "fld tbyte ptr [{v} + 30]",
            "fld tbyte ptr [{v} + 20]",
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
            "fst dword ptr [{mk} + 0x138052]",
            "fadd dword ptr [{mk} + 0x3724]",
            "fst dword ptr [{mk} + 0x13805E]",
            "fadd dword ptr [{mk} + 0x3718]",
            "fst dword ptr [{mk} + 0x13806A]",
            "fsub dword ptr [{mk} + 0x3724]",
            "fstp dword ptr [{mk} + 0x138076]",
            "fld dword ptr [{mk} + 0x138056]",
            "fst dword ptr [{mk} + 0x138056]",
            "fadd dword ptr [{mk} + 0x3728]",
            "fst dword ptr [{mk} + 0x138062]",
            "fadd dword ptr [{mk} + 0x371C]",
            "fst dword ptr [{mk} + 0x13806E]",
            "fsub dword ptr [{mk} + 0x3728]",
            "fstp dword ptr [{mk} + 0x13807A]",
            "fld dword ptr [{mk} + 0x13805A]",
            "fst dword ptr [{mk} + 0x13805A]",
            "fadd dword ptr [{mk} + 0x372C]",
            "fst dword ptr [{mk} + 0x138066]",
            "fadd dword ptr [{mk} + 0x3720]",
            "fst dword ptr [{mk} + 0x138072]",
            "fsub dword ptr [{mk} + 0x372C]",
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
            "fmul dword ptr [{mk} + 0x137FAA]",
            "fld dword ptr [{mk} + 0x138066]",
            "fmul dword ptr [{mk} + 0x137FAA]",
            "fld dword ptr [{mk} + 0x138072]",
            "fmul dword ptr [{mk} + 0x137FAA]",
            "fld dword ptr [{mk} + 0x13807E]",
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
    fn sides_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x218,
                    hi: 0x137FF2,
                    reads: &[
                        (0x218, F32),
                        (0x137FAE, F32),
                        (0x137FB2, F32),
                        (0x137FDA, F32),
                        (0x137FDE, F32),
                        (0x137FE2, F32),
                        (0x137FE6, F32),
                        (0x137FEA, F32),
                        (0x137FEE, F32),
                    ],
                    writes: &[
                        (0x3718, F32),
                        (0x371C, F32),
                        (0x3720, F32),
                        (0x3724, F32),
                        (0x3728, F32),
                        (0x372C, F32),
                        (0x3730, F32),
                        (0x3734, F32),
                        (0x3738, F32),
                        (0x373C, F32),
                        (0x3740, F32),
                        (0x3744, F32),
                    ],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x28,
                    reads: &[(0x0, F80), (0xA, F80), (0x14, F80), (0x1E, F80)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::sides_x87(b[0], b[1]) },
            |b| unsafe { super::sides_x87(b[0], b[1], crate::marks::pong::state_at(b[0])) },
        );
    }

    #[test]
    fn corners_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x3718,
                    hi: 0x138082,
                    reads: &[
                        (0x3718, F32),
                        (0x371C, F32),
                        (0x3720, F32),
                        (0x3724, F32),
                        (0x3728, F32),
                        (0x372C, F32),
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
                    lo: 0x14,
                    hi: 0x28,
                    reads: &[(0x14, F80), (0x1E, F80)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::corners_x87(b[0], b[1]) },
            |b| unsafe { super::corners_x87(b[0], b[1], crate::marks::pong::state_at(b[0])) },
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
                    lo: 0x137FAA,
                    hi: 0x138082,
                    reads: &[
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
