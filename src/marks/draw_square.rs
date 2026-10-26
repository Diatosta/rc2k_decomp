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

use crate::marks::squares_store::SQUARE_STATE;
use crate::marks::tuning::{MARKS_KIND_FLOATS, MarksKindFloats};
use crate::session::read_bits;

use crate::d3d::vertices_store::VERTICES;
const FAR_ENOUGH: i32 = 0x4580_0000; // 4096.0

/// The original's 0x4BF845 .. 0x4BF88D: the item's texture shift added to the places.
fn shifted_places(s: u32, p: u32) {
    shifted_places_x87(s as usize, p as usize)
}

/// The original's 0x4BF88E .. 0x4BF9AA on the two values: the corners.
fn corners(v: &[[u8; 10]; 2]) {
    corners_x87(
        (crate::pointers::game_address(v.as_ptr())) as usize,
        &SQUARE_TABLES,
    )
}

/// The original's 0x4BF9BA .. 0x4BFAD6: the corners projected, their depth.
fn projected(p: u32) {
    projected_x87(
        crate::race::store::asm_base() as usize,
        p as usize,
        &MARKS_KIND_FLOATS,
        &SQUARE_TABLES,
    )
}

/// The Rust version (see the header): `item` (esi), `colour` (edx).
pub fn draw_square(item: u32, colour: u32, shares: [[u8; 10]; 2]) {
    let vertices = crate::d3d::vertices_store::piece_addr();
    let [a, pb, c, d]: [u32; 4] = read_bits(&SQUARE_STATE.places);
    for (n, (u, v)) in [(a, pb), (c, pb), (c, d), (a, d)].into_iter().enumerate() {
        VERTICES.vertices[n].u.set_bits(u);
        VERTICES.vertices[n].v.set_bits(v);
    }
    shifted_places(item, vertices);
    corners(&shares);
    if SQUARE_TABLES.corners[0][2].bits() as i32 <= FAR_ENOUGH {
        return;
    }
    projected(vertices);
    for n in 0..4 {
        VERTICES.vertices[n].colour.set(colour);
    }
    crate::marks::clip_quad::draw_fan();
}

use crate::marks::plain::{ld, ld80, st};
use crate::marks::squares_store::{SQUARE_TABLES, SquareTables};
use crate::math::x87_float as bh;

/// `shifted_places`'s x87 block as plain Rust on the x87 model (24 instructions), same order and memory places.
fn shifted_places_x87(s: usize, p: usize) {
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

/// `corners`'s x87 block as plain Rust on the x87 model (56 instructions), same order and memory places.
fn corners_x87(v: usize, sq: &SquareTables) {
    let t0 = bh::from_f80(ld80(v, 0xA));
    let t1 = bh::from_f80(ld80(v, 0));
    let t2 = t1 * bh::ld(sq.lengths[1].bits());
    let t3 = t0 * bh::ld(sq.lengths[0].bits());
    let t4 = t3 * bh::ld(sq.axes[0][0].bits());
    let t5 = t2 * bh::ld(sq.axes[1][0].bits());
    let t6 = t4 + t5;
    let t7 = t6 + bh::ld(sq.place[0].bits());
    sq.corners[0][0].set_bits(bh::st(t7));
    let t8 = t3 * bh::ld(sq.axes[0][1].bits());
    let t9 = t2 * bh::ld(sq.axes[1][1].bits());
    let t10 = t8 + t9;
    let t11 = t10 + bh::ld(sq.place[1].bits());
    sq.corners[0][1].set_bits(bh::st(t11));
    let t12 = t2 * bh::ld(sq.axes[1][2].bits());
    let t13 = t3 * bh::ld(sq.axes[0][2].bits());
    let t14 = t12 + t13;
    let t15 = t14 + bh::ld(sq.place[2].bits());
    sq.corners[0][2].set_bits(bh::st(t15));
    let t16 = bh::ld(sq.corners[0][0].bits());
    let t17 = t16 - bh::ld(sq.half_axes[0][0].bits());
    let t18 = t17 - bh::ld(sq.half_axes[1][0].bits());
    sq.corners[0][0].set_bits(bh::st(t18));
    let t19 = t18 + bh::ld(sq.scaled_by_radius[1][0].bits());
    sq.corners[1][0].set_bits(bh::st(t19));
    let t20 = t19 + bh::ld(sq.scaled_by_radius[0][0].bits());
    sq.corners[2][0].set_bits(bh::st(t20));
    let t21 = t20 - bh::ld(sq.scaled_by_radius[1][0].bits());
    sq.corners[3][0].set_bits(bh::st(t21));
    let t22 = bh::ld(sq.corners[0][1].bits());
    let t23 = t22 - bh::ld(sq.half_axes[0][1].bits());
    let t24 = t23 - bh::ld(sq.half_axes[1][1].bits());
    sq.corners[0][1].set_bits(bh::st(t24));
    let t25 = t24 + bh::ld(sq.scaled_by_radius[1][1].bits());
    sq.corners[1][1].set_bits(bh::st(t25));
    let t26 = t25 + bh::ld(sq.scaled_by_radius[0][1].bits());
    sq.corners[2][1].set_bits(bh::st(t26));
    let t27 = t26 - bh::ld(sq.scaled_by_radius[1][1].bits());
    sq.corners[3][1].set_bits(bh::st(t27));
    let t28 = bh::ld(sq.corners[0][2].bits());
    let t29 = t28 - bh::ld(sq.half_axes[0][2].bits());
    let t30 = t29 - bh::ld(sq.half_axes[1][2].bits());
    sq.corners[0][2].set_bits(bh::st(t30));
    let t31 = t30 + bh::ld(sq.scaled_by_radius[1][2].bits());
    sq.corners[1][2].set_bits(bh::st(t31));
    let t32 = t31 + bh::ld(sq.scaled_by_radius[0][2].bits());
    sq.corners[2][2].set_bits(bh::st(t32));
    let t33 = t32 - bh::ld(sq.scaled_by_radius[1][2].bits());
    sq.corners[3][2].set_bits(bh::st(t33));
}

/// `projected`'s x87 block as plain Rust on the x87 model (64 instructions), same order and memory places.
fn projected_x87(rc: usize, p: usize, k: &MarksKindFloats, sq: &SquareTables) {
    let t0 = ld(rc, 0x224088);
    let t1 = t0 / bh::ld(sq.corners[3][2].bits());
    let t2 = ld(rc, 0x224088);
    let t3 = t2 / bh::ld(sq.corners[2][2].bits());
    let t4 = ld(rc, 0x224088);
    let t5 = t4 / bh::ld(sq.corners[1][2].bits());
    let t6 = ld(rc, 0x224088);
    let t7 = t6 / bh::ld(sq.corners[0][2].bits());
    let t8 = bh::ld(sq.corners[0][1].bits());
    let t9 = t8 * t7;
    let t10 = t7 * bh::ld(sq.corners[0][0].bits());
    let t11 = -t9;
    let t12 = t11 + ld(rc, 0x2240A0);
    st(p, 0x4, t12);
    let t13 = t10 + ld(rc, 0x22409C);
    st(p, 0, t13);
    let t14 = bh::ld(sq.corners[1][1].bits());
    let t15 = t14 * t5;
    let t16 = t5 * bh::ld(sq.corners[1][0].bits());
    let t17 = -t15;
    let t18 = t17 + ld(rc, 0x2240A0);
    st(p, 0x24, t18);
    let t19 = t16 + ld(rc, 0x22409C);
    st(p, 0x20, t19);
    let t20 = bh::ld(sq.corners[2][1].bits());
    let t21 = t20 * t3;
    let t22 = t3 * bh::ld(sq.corners[2][0].bits());
    let t23 = -t21;
    let t24 = t23 + ld(rc, 0x2240A0);
    st(p, 0x44, t24);
    let t25 = t22 + ld(rc, 0x22409C);
    st(p, 0x40, t25);
    let t26 = bh::ld(sq.corners[3][1].bits());
    let t27 = t26 * t1;
    let t28 = t1 * bh::ld(sq.corners[3][0].bits());
    let t29 = -t27;
    let t30 = t29 + ld(rc, 0x2240A0);
    st(p, 0x64, t30);
    let t31 = t28 + ld(rc, 0x22409C);
    st(p, 0x60, t31);
    let t32 = bh::ld(sq.corners[0][2].bits());
    let t33 = t32 + bh::ld(k.kind_float.bits());
    let t34 = t33 * bh::ld(sq.inverse.bits());
    let t35 = bh::ld(sq.corners[1][2].bits());
    let t36 = t35 + bh::ld(k.kind_float.bits());
    let t37 = t36 * bh::ld(sq.inverse.bits());
    let t38 = bh::ld(sq.corners[2][2].bits());
    let t39 = t38 + bh::ld(k.kind_float.bits());
    let t40 = t39 * bh::ld(sq.inverse.bits());
    let t41 = bh::ld(sq.corners[3][2].bits());
    let t42 = t41 + bh::ld(k.kind_float.bits());
    let t43 = t42 * bh::ld(sq.inverse.bits());
    st(p, 0x68, t43);
    st(p, 0x48, t40);
    st(p, 0x28, t37);
    st(p, 0x8, t34);
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn shifted_places_x87(s: usize, p: usize) {
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

    pub(super) fn corners_x87(mk: usize, v: usize) {
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

    pub(super) fn projected_x87(rc: usize, mk: usize, p: usize) {
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
    use crate::marks::asm_test::with_copy;
    use crate::marks::asm_test::{Op, Spec, differential};
    use crate::marks::squares_store::SquareTables;
    use crate::marks::tuning::MarksKindFloats;

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
            |b| reference::shifted_places_x87(b[0], b[1]),
            |b| super::shifted_places_x87(b[0], b[1]),
        );
    }

    #[test]
    fn corners_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x137B8A,
                    hi: 0x13822A,
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
            |b| reference::corners_x87(b[0], b[1]),
            |b| {
                with_copy::<SquareTables, _>(b[0], SquareTables::START, |sq| {
                    super::corners_x87(b[1], sq)
                })
            },
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
                    lo: 0x2F8,
                    hi: 0x13822A,
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
            |b| reference::projected_x87(b[0], b[1], b[2]),
            |b| {
                with_copy::<MarksKindFloats, _>(b[1], MarksKindFloats::START, |k| {
                    with_copy::<SquareTables, _>(b[1], SquareTables::START, |sq| {
                        super::projected_x87(b[0], b[2], k, sq)
                    })
                })
            },
        );
    }
}
