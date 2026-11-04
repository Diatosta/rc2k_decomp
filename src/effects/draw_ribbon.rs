//! `effects::draw_ribbon` (0x4C5E63): a ribbon piece between two places already in the
//! view's frame (`flt_8EB8D5` .. and `dword_8EB8E1` ..): half the width (edx >> 1, whole)
//! in `dword_8EB8B5`, the four vertices' colour (ecx << 24) | esi and texture places from
//! cell eax of `g_marksCellPlaces`; unless either depth is at or below `dword_8EB961` (a
//! signed compare of the bits): both projected (`flt_8EB8CD`, `flt_8EB8D1` their scales),
//! the way between them made one long and turned a quarter (`flt_8F3985`, `flt_8F3989`),
//! the sides that times the half width and each scale (`flt_8F3991` ..), the four corners,
//! the depths times `flt_8EB96D`; `effects::clip_quad`, and unless all is outside
//! `scene::draw_fan` and `g_polygonVertexCount` 4.
//!
//! In:  eax the cell, ecx the alpha, edx the width, esi the colour (and the two places)
//! Out: nothing (its caller reads no register after it)
//!
//! Callers: the marks' lines (0x4C5D65) and 0x4CFAA3. The x87 block is plain Rust on the x87 model (`math::x87_float`), in the original's order, over the game's variables. Refuses, before anything, a cell past 0x40 and
//! null marks settings (the clip's refusal).

use crate::effects::clip_quad::{clip_quad, draw_fan};
use crate::effects::surface_store::SURFACE;
use crate::errors::{BadInput, Refused};
use crate::scene::draw_store::DRAW;

use crate::d3d::vertices_store::VERTICES;
const CELLS: u32 = 0x41;

/// The original's 0x4C5EF7 .. 0x4C60E9: the two projected and the ribbon's corners.
fn ribbon() {
    ribbon_x87(&DRAW_WORK, &VIEW_VARS);
}

/// The Rust version (see the header): `cell` (eax), `alpha` (ecx), `width` (edx),
/// `colour` (esi). Its one caller reads no register after it.
pub fn draw_ribbon(cell: u32, alpha: u32, width: u32, colour: u32) -> Result<(), Refused> {
    if cell >= CELLS {
        return Err(BadInput("a mark cell past g_marksCellPlaces").into());
    }
    {
        if SETTINGS.is_set.get() == 0 {
            return Err(BadInput("no marks settings").into());
        }
        DRAW_WORK.half[0].set_bits(width >> 1);
        let colour = (alpha << 24) | colour;
        for n in 0..4 {
            VERTICES.vertices[n].colour.set(colour);
        }
        let at = cell << 4;
        let place = |n: usize| SURFACE.places.get_u32(at as usize + 4 * n);
        let [u0, v0, u1, v1] = [place(0), place(1), place(2), place(3)];
        for (n, (u, v)) in [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
            .into_iter()
            .enumerate()
        {
            VERTICES.vertices[n].u.set_bits(u);
            VERTICES.vertices[n].v.set_bits(v);
        }
        for depth in [DRAW_WORK.turned[2].bits(), DRAW_WORK.second[2].bits()] {
            if depth as i32 <= DRAW_WORK.near.bits() as i32 {
                return Ok(());
            }
        }
        ribbon();
        if clip_quad()? {
            return Ok(());
        }
        draw_fan();
        DRAW.polygon_vertex_count.set_bits(4u32);
    }
    Ok(())
}

use crate::effects::frame_state::{DRAW_WORK, DrawWork, VIEW_VARS, ViewVars};
use crate::effects::plain::st_cell;
use crate::effects::setup::SETTINGS;
use crate::math::x87_float as bh;
use crate::math::x87_float::ld;

/// `ribbon`'s x87 block as plain Rust on the x87 model (93 instructions), same order and memory places.
fn ribbon_x87(dw: &DrawWork, vv: &ViewVars) {
    let t0 = ld(DRAW.view_scale.bits());
    let t1 = t0 / bh::ld(dw.turned[2].bits());
    let t2 = t1 * bh::ld(dw.turned[0].bits());
    let t3 = t1 * bh::ld(dw.turned[1].bits());
    let t4 = -t3;
    let t5 = t4 + ld(DRAW.view_centre[1].bits());
    dw.turned[1].set_bits(bh::st(t5));
    let t6 = t2 + ld(DRAW.view_centre[0].bits());
    dw.turned[0].set_bits(bh::st(t6));
    dw.ends_scale[0].set_bits(bh::st(t1));
    let t7 = ld(DRAW.view_scale.bits());
    let t8 = t7 / bh::ld(dw.second[2].bits());
    let t9 = t8 * bh::ld(dw.second[0].bits());
    let t10 = t8 * bh::ld(dw.second[1].bits());
    let t11 = -t10;
    let t12 = t11 + ld(DRAW.view_centre[1].bits());
    dw.second[1].set_bits(bh::st(t12));
    let t13 = t9 + ld(DRAW.view_centre[0].bits());
    dw.second[0].set_bits(bh::st(t13));
    dw.ends_scale[1].set_bits(bh::st(t8));
    let t14 = bh::ld(dw.second[0].bits());
    let t15 = t14 - bh::ld(dw.turned[0].bits());
    vv.ribbon_way[0].set_bits(bh::st(t15));
    let t16 = bh::ld(dw.second[1].bits());
    let t17 = t16 - bh::ld(dw.turned[1].bits());
    vv.ribbon_way[1].set_bits(bh::st(t17));
    let t18 = bh::ld(vv.ribbon_way[0].bits());
    let t19 = t18 * t18;
    let t20 = bh::ld(vv.ribbon_way[1].bits());
    let t21 = t20 * t20;
    let t22 = t19 + t21;
    let t23 = t22.sqrt();
    let t24 = bh::ONE;
    let t25 = t24 / t23;
    let t26 = bh::ld(vv.ribbon_way[0].bits());
    let t27 = bh::ld(vv.ribbon_way[1].bits());
    let t28 = t27 * t25;
    vv.ribbon_way[0].set_bits(bh::st(t28));
    let t29 = t25 * t26;
    let t30 = -t29;
    vv.ribbon_way[1].set_bits(bh::st(t30));
    let t31 = bh::ld(vv.ribbon_way[0].bits());
    let t32 = t31 * bh::int(dw.half[0].bits() as i32);
    let t33 = t32 * bh::ld(dw.ends_scale[0].bits());
    vv.ribbon_sides[0].set_bits(bh::st(t33));
    let t34 = bh::ld(vv.ribbon_way[1].bits());
    let t35 = t34 * bh::int(dw.half[0].bits() as i32);
    let t36 = t35 * bh::ld(dw.ends_scale[0].bits());
    vv.ribbon_sides[1].set_bits(bh::st(t36));
    let t37 = bh::ld(vv.ribbon_way[0].bits());
    let t38 = t37 * bh::int(dw.half[0].bits() as i32);
    let t39 = t38 * bh::ld(dw.ends_scale[1].bits());
    vv.ribbon_sides_2[0].set_bits(bh::st(t39));
    let t40 = bh::ld(vv.ribbon_way[1].bits());
    let t41 = t40 * bh::int(dw.half[0].bits() as i32);
    let t42 = t41 * bh::ld(dw.ends_scale[1].bits());
    vv.ribbon_sides_2[1].set_bits(bh::st(t42));
    let t43 = bh::ld(dw.turned[0].bits());
    let t44 = t43 - bh::ld(vv.ribbon_sides[0].bits());
    st_cell(&VERTICES.vertices[0].x, t44);
    let t45 = bh::ld(dw.turned[1].bits());
    let t46 = t45 - bh::ld(vv.ribbon_sides[1].bits());
    st_cell(&VERTICES.vertices[0].y, t46);
    let t47 = bh::ld(dw.turned[0].bits());
    let t48 = t47 + bh::ld(vv.ribbon_sides[0].bits());
    st_cell(&VERTICES.vertices[1].x, t48);
    let t49 = bh::ld(dw.turned[1].bits());
    let t50 = t49 + bh::ld(vv.ribbon_sides[1].bits());
    st_cell(&VERTICES.vertices[1].y, t50);
    let t51 = bh::ld(dw.second[0].bits());
    let t52 = t51 + bh::ld(vv.ribbon_sides_2[0].bits());
    st_cell(&VERTICES.vertices[2].x, t52);
    let t53 = bh::ld(dw.second[1].bits());
    let t54 = t53 + bh::ld(vv.ribbon_sides_2[1].bits());
    st_cell(&VERTICES.vertices[2].y, t54);
    let t55 = bh::ld(dw.second[0].bits());
    let t56 = t55 - bh::ld(vv.ribbon_sides_2[0].bits());
    st_cell(&VERTICES.vertices[3].x, t56);
    let t57 = bh::ld(dw.second[1].bits());
    let t58 = t57 - bh::ld(vv.ribbon_sides_2[1].bits());
    st_cell(&VERTICES.vertices[3].y, t58);
    let t59 = bh::ld(dw.turned[2].bits());
    let t60 = t59 * bh::ld(dw.z_k.bits());
    st_cell(&VERTICES.vertices[0].z, t60);
    st_cell(&VERTICES.vertices[1].z, t60);
    let t61 = bh::ld(dw.second[2].bits());
    let t62 = t61 * bh::ld(dw.z_k.bits());
    st_cell(&VERTICES.vertices[2].z, t62);
    st_cell(&VERTICES.vertices[3].z, t62);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn ribbon_x87(rc: usize, bf: usize, mk: usize) {
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
    use crate::effects::frame_state::{DrawWork, ViewVars};
    use crate::session::Float;

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
    fn ribbon_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
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
                    lo: 0x186931,
                    hi: 0x18EA8D,
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
            |b| reference::ribbon_x87(b[0], b[1], b[2]),
            |b| {
                with_copy::<DrawWork, _>(b[2], DrawWork::START, |dw| {
                    with_copy::<ViewVars, _>(b[2], ViewVars::START, |vv| {
                        staged(
                            b[1].wrapping_add(0x5605A0),
                            &[
                                0x0, 0x4, 0x8, 0x20, 0x24, 0x28, 0x40, 0x44, 0x48, 0x60, 0x64, 0x68,
                            ],
                            || super::ribbon_x87(dw, vv),
                        );
                    });
                });
            },
        );
    }
}
