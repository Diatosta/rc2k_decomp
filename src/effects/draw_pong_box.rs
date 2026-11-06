//! `effects::draw_pong_box` (0x4B8599): one box of the pong game (ball, bat or a score
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
//! Callers: `effects::run_pong` (ball, bat), `effects::pong_cell`. The x87 runs are plain Rust on the x87 model
//! (`math::x87_float`), in the original's order, over the game's variables.

use crate::effects::pong::{PONG, PONG_RULES, PongState};
use crate::effects::tuning::{MARKS_FLOATS, MarksFloats};
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::math::x87;

use crate::d3d::vertices_store::VERTICES;

/// The original's 0x4B8599 .. 0x4B8659 on the four values: the sides; the place's two
/// values (left untouched) popped after.
fn sides(fl: &WindscreenFlakes, v: &[[u8; 10]; 4]) {
    sides_x87(v, &PONG, &MARKS_FLOATS, &fl.tables);
}

/// The original's 0x4B868E .. 0x4B8786 on the place: the corners.
fn corners(fl: &WindscreenFlakes, v: &[[u8; 10]; 4]) {
    corners_x87(v, &PONG, &fl.tables);
}

/// The original's 0x4B87A5 .. 0x4B88A9: the corners projected, their depth.
fn projected(fl: &WindscreenFlakes) {
    projected_x87(&fl.tables);
}

/// The Rust version (see the header).
pub fn draw_pong_box(fl: &WindscreenFlakes, v: [[u8; 10]; 4]) {
    sides(fl, &v);
    let [pa, pb, pc, pd] = PONG_RULES
        .box_places
        .each_ref()
        .map(super::super::session::Float::bits);
    for (n, (u, w)) in [(pa, pb), (pc, pb), (pc, pd), (pa, pd)]
        .into_iter()
        .enumerate()
    {
        VERTICES.vertices[n].u.set_bits(u);
        VERTICES.vertices[n].v.set_bits(w);
    }
    corners(fl, &v);
    if x87::compare_status(fl.tables.corners[0][2].bits(), PONG_RULES.near.bits()) == 0x100 {
        return;
    }
    projected(fl);
    for n in 0..4 {
        VERTICES.vertices[n].colour.set(0xFFFF_FFFFu32);
    }
    crate::effects::clip_quad::draw_fan();
}

use crate::effects::plain::st_cell;
use crate::effects::windscreen_flakes_store::WindscreenFlakeTables;
use crate::math::x87_float as bh;
use crate::scene::draw_store::DRAW;

/// `sides`'s x87 block as plain Rust on the x87 model (42 instructions), same order and memory places.
fn sides_x87(v: &[[u8; 10]; 4], p: &PongState, f: &MarksFloats, sq: &WindscreenFlakeTables) {
    let _t0 = bh::from_f80(v[3]);
    let _t1 = bh::from_f80(v[2]);
    let t2 = bh::from_f80(v[1]);
    let t3 = bh::from_f80(v[0]);
    let t4 = bh::ld(sq.axes[1][0].bits());
    let t5 = t4 * t3;
    let t6 = t5 * bh::ld(sq.lengths[1].bits());
    p.box_sides[3].set_bits(bh::st(t6));
    let t7 = t6 * bh::ld(f.half.bits());
    p.box_sides[9].set_bits(bh::st(t7));
    let t8 = bh::ld(sq.axes[1][1].bits());
    let t9 = t8 * t3;
    let t10 = t9 * bh::ld(sq.lengths[1].bits());
    p.box_sides[4].set_bits(bh::st(t10));
    let t11 = t10 * bh::ld(f.half.bits());
    p.box_sides[10].set_bits(bh::st(t11));
    let t12 = bh::ld(sq.axes[1][2].bits());
    let t13 = t3 * t12;
    let t14 = t13 * bh::ld(sq.lengths[1].bits());
    p.box_sides[5].set_bits(bh::st(t14));
    let t15 = t14 * bh::ld(f.half.bits());
    p.box_sides[11].set_bits(bh::st(t15));
    let t16 = bh::ld(sq.axes[0][0].bits());
    let t17 = t16 * t2;
    let t18 = t17 * bh::ld(sq.lengths[0].bits());
    p.box_sides[0].set_bits(bh::st(t18));
    let t19 = t18 * bh::ld(f.half.bits());
    p.box_sides[6].set_bits(bh::st(t19));
    let t20 = bh::ld(sq.axes[0][1].bits());
    let t21 = t20 * t2;
    let t22 = t21 * bh::ld(sq.lengths[0].bits());
    p.box_sides[1].set_bits(bh::st(t22));
    let t23 = t22 * bh::ld(f.half.bits());
    p.box_sides[7].set_bits(bh::st(t23));
    let t24 = bh::ld(sq.axes[0][2].bits());
    let t25 = t2 * t24;
    let t26 = t25 * bh::ld(sq.lengths[0].bits());
    p.box_sides[2].set_bits(bh::st(t26));
    let t27 = t26 * bh::ld(f.half.bits());
    p.box_sides[8].set_bits(bh::st(t27));
}

/// `corners`'s x87 block as plain Rust on the x87 model (50 instructions), same order and memory places.
fn corners_x87(v: &[[u8; 10]; 4], p: &PongState, sq: &WindscreenFlakeTables) {
    let t0 = bh::from_f80(v[3]);
    let t1 = bh::from_f80(v[2]);
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
    sq.corners[0][0].set_bits(bh::st(t16));
    let t17 = t16 + bh::ld(p.box_sides[3].bits());
    sq.corners[1][0].set_bits(bh::st(t17));
    let t18 = t17 + bh::ld(p.box_sides[0].bits());
    sq.corners[2][0].set_bits(bh::st(t18));
    let t19 = t18 - bh::ld(p.box_sides[3].bits());
    sq.corners[3][0].set_bits(bh::st(t19));
    let t20 = bh::ld(sq.corners[0][1].bits());
    sq.corners[0][1].set_bits(bh::st(t20));
    let t21 = t20 + bh::ld(p.box_sides[4].bits());
    sq.corners[1][1].set_bits(bh::st(t21));
    let t22 = t21 + bh::ld(p.box_sides[1].bits());
    sq.corners[2][1].set_bits(bh::st(t22));
    let t23 = t22 - bh::ld(p.box_sides[4].bits());
    sq.corners[3][1].set_bits(bh::st(t23));
    let t24 = bh::ld(sq.corners[0][2].bits());
    sq.corners[0][2].set_bits(bh::st(t24));
    let t25 = t24 + bh::ld(p.box_sides[5].bits());
    sq.corners[1][2].set_bits(bh::st(t25));
    let t26 = t25 + bh::ld(p.box_sides[2].bits());
    sq.corners[2][2].set_bits(bh::st(t26));
    let t27 = t26 - bh::ld(p.box_sides[5].bits());
    sq.corners[3][2].set_bits(bh::st(t27));
}

/// `projected`'s x87 block as plain Rust on the x87 model (60 instructions), same order and memory places.
fn projected_x87(sq: &WindscreenFlakeTables) {
    let t0 = bh::ld(DRAW.view_scale.bits());
    let t1 = t0 / bh::ld(sq.corners[3][2].bits());
    let t2 = bh::ld(DRAW.view_scale.bits());
    let t3 = t2 / bh::ld(sq.corners[2][2].bits());
    let t4 = bh::ld(DRAW.view_scale.bits());
    let t5 = t4 / bh::ld(sq.corners[1][2].bits());
    let t6 = bh::ld(DRAW.view_scale.bits());
    let t7 = t6 / bh::ld(sq.corners[0][2].bits());
    let t8 = bh::ld(sq.corners[0][1].bits());
    let t9 = t8 * t7;
    let t10 = t7 * bh::ld(sq.corners[0][0].bits());
    let t11 = -t9;
    let t12 = t11 + bh::ld(DRAW.view_centre[1].bits());
    st_cell(&VERTICES.vertices[0].y, t12);
    let t13 = t10 + bh::ld(DRAW.view_centre[0].bits());
    st_cell(&VERTICES.vertices[0].x, t13);
    let t14 = bh::ld(sq.corners[1][1].bits());
    let t15 = t14 * t5;
    let t16 = t5 * bh::ld(sq.corners[1][0].bits());
    let t17 = -t15;
    let t18 = t17 + bh::ld(DRAW.view_centre[1].bits());
    st_cell(&VERTICES.vertices[1].y, t18);
    let t19 = t16 + bh::ld(DRAW.view_centre[0].bits());
    st_cell(&VERTICES.vertices[1].x, t19);
    let t20 = bh::ld(sq.corners[2][1].bits());
    let t21 = t20 * t3;
    let t22 = t3 * bh::ld(sq.corners[2][0].bits());
    let t23 = -t21;
    let t24 = t23 + bh::ld(DRAW.view_centre[1].bits());
    st_cell(&VERTICES.vertices[2].y, t24);
    let t25 = t22 + bh::ld(DRAW.view_centre[0].bits());
    st_cell(&VERTICES.vertices[2].x, t25);
    let t26 = bh::ld(sq.corners[3][1].bits());
    let t27 = t26 * t1;
    let t28 = t1 * bh::ld(sq.corners[3][0].bits());
    let t29 = -t27;
    let t30 = t29 + bh::ld(DRAW.view_centre[1].bits());
    st_cell(&VERTICES.vertices[3].y, t30);
    let t31 = t28 + bh::ld(DRAW.view_centre[0].bits());
    st_cell(&VERTICES.vertices[3].x, t31);
    let t32 = bh::ld(sq.corners[0][2].bits());
    let t33 = t32 * bh::ld(sq.inverse.bits());
    let t34 = bh::ld(sq.corners[1][2].bits());
    let t35 = t34 * bh::ld(sq.inverse.bits());
    let t36 = bh::ld(sq.corners[2][2].bits());
    let t37 = t36 * bh::ld(sq.inverse.bits());
    let t38 = bh::ld(sq.corners[3][2].bits());
    let t39 = t38 * bh::ld(sq.inverse.bits());
    st_cell(&VERTICES.vertices[3].z, t39);
    st_cell(&VERTICES.vertices[2].z, t37);
    st_cell(&VERTICES.vertices[1].z, t35);
    st_cell(&VERTICES.vertices[0].z, t33);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn sides_x87(mk: usize, v: usize) {
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
            );
        }
    }

    pub(super) fn corners_x87(mk: usize, v: usize) {
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
            );
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
            );
        }
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::d3d::vertices_store::VERTICES;
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::with_copy;
    use crate::effects::asm_test::{Op, Spec, differential};
    use crate::effects::windscreen_flakes_store::WindscreenFlakeTables;
    use crate::session::Float;

    /// The four ten-byte values at `at` of the test buffer.
    fn values(at: usize) -> [[u8; 10]; 4] {
        std::array::from_fn(|n| {
            std::array::from_fn(|i| unsafe { crate::pointers::host_at(at + 10 * n + i).read() })
        })
    }

    /// The vertex float at byte `off` of the vertices (x, y or z of a vertex).
    fn vertex_cell(off: usize) -> &'static Float {
        let v = &VERTICES.vertices[off / 32];
        match off % 32 {
            0x0 => &v.x,
            0x4 => &v.y,
            0x8 => &v.z,
            _ => panic!("not a tested vertex field"),
        }
    }

    /// Runs `f` with the vertex floats at `offsets` holding the test buffer's (at `at`)
    /// bytes; what `f` leaves goes back to the buffer, the vertices as they were.
    fn staged<R>(at: usize, offsets: &[usize], f: impl FnOnce() -> R) -> R {
        let host = |off: usize| crate::pointers::host_at(at + off).cast::<u32>();
        let saved: Vec<u32> = offsets.iter().map(|&o| vertex_cell(o).bits()).collect();
        for &o in offsets {
            vertex_cell(o).set_bits(unsafe { host(o).read_unaligned() });
        }
        let out = f();
        for (&o, &old) in offsets.iter().zip(&saved) {
            unsafe { host(o).write_unaligned(vertex_cell(o).bits()) };
            vertex_cell(o).set_bits(old);
        }
        out
    }

    #[test]
    fn sides_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x200,
                    hi: 0x13822A,
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
            |b| reference::sides_x87(b[0], b[1]),
            |b| {
                crate::effects::asm_test::with_copy(
                    b[0],
                    crate::effects::tuning::MarksFloats::START,
                    |f| {
                        with_copy::<WindscreenFlakeTables, _>(
                            b[0],
                            WindscreenFlakeTables::START,
                            |sq| {
                                super::sides_x87(
                                    &values(b[1]),
                                    crate::effects::pong::state_at(b[0]),
                                    f,
                                    sq,
                                );
                            },
                        );
                    },
                );
            },
        );
    }

    #[test]
    fn corners_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x3718,
                    hi: 0x13822A,
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
            |b| reference::corners_x87(b[0], b[1]),
            |b| {
                with_copy::<WindscreenFlakeTables, _>(b[0], WindscreenFlakeTables::START, |sq| {
                    super::corners_x87(&values(b[1]), crate::effects::pong::state_at(b[0]), sq);
                });
            },
        );
    }

    #[test]
    fn projected_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x137B8A,
                    hi: 0x13822A,
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
            |b| reference::projected_x87(b[0], b[1], b[2]),
            |b| {
                with_copy::<WindscreenFlakeTables, _>(b[1], WindscreenFlakeTables::START, |sq| {
                    staged(
                        b[2],
                        &[
                            0x0, 0x4, 0x8, 0x20, 0x24, 0x28, 0x40, 0x44, 0x48, 0x60, 0x64, 0x68,
                        ],
                        || super::projected_x87(sq),
                    );
                });
            },
        );
    }
}
