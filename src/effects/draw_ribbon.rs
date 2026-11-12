//! `effects::draw_ribbon` (0x4C5E63): a ribbon piece between two places already in the
//! view's frame: half the width (the width shifted right once, whole) kept, the four
//! vertices' colour (alpha << 24) | colour and texture places from the cell's entry in the
//! cell places table; unless either depth is at or below the nearest depth (a signed compare
//! of the bits): both projected (each with its own scale), the way between them made one long
//! and turned a quarter, the sides that times the half width and each scale, the four
//! corners, the depths times the depth scale; `effects::clip_quad`, and unless all is outside
//! `scene::draw_fan` and the polygon's vertex count 4.
//!
//! Takes: the cell, the alpha, the width, the colour (and the two places)
//!
//! Callers: the effects' lines and `effects::run_ribbons`. The x87 block is
//! plain Rust on the x87 model (`math::x87_float`), in the original's order, over the game's
//! variables. Refuses, before anything, a cell past 0x40 and null marks settings (the clip's
//! refusal).

use crate::effects::cell_x87::CellX87;
use crate::effects::clip_quad::{clip_quad, draw_fan};
use crate::effects::mask_store::Masks;
use crate::errors::{BadInput, Refused};

const CELLS: u32 = 0x41;

/// The two places projected and the ribbon's corners.
fn ribbon(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    effects: &crate::effects::owner::Effects,
) {
    let frame = &effects.frame;
    ribbon_x87(buffer, &frame.draw_work, &frame.view_vars);
}

/// Draws a ribbon piece of `colour` and `alpha` and `width`, with the texture places of
/// `cell`.
#[allow(
    clippy::too_many_arguments,
    reason = "the effects' groups, the owners and the vertex buffer"
)]
pub fn draw_ribbon(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    masks: &Masks,
    cell: u32,
    alpha: u32,
    width: u32,
    colour: u32,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    if cell >= CELLS {
        return Err(BadInput("a mark cell past g_marksCellPlaces").into());
    }
    {
        if effects.setup.settings.is_set.get() == 0 {
            return Err(BadInput("no marks settings").into());
        }
        effects.frame.draw_work.half[0].set_bits(width >> 1);
        let colour = (alpha << 24) | colour;
        for n in 0..4 {
            buffer.vertices[n].colour.set(colour);
        }
        let at = cell << 4;
        let place = |n: usize| masks.surface.places.get_u32(at as usize + 4 * n);
        let [u0, v0, u1, v1] = [place(0), place(1), place(2), place(3)];
        for (n, (u, v)) in [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
            .into_iter()
            .enumerate()
        {
            buffer.vertices[n].u.set_bits(u);
            buffer.vertices[n].v.set_bits(v);
        }
        for depth in [
            effects.frame.draw_work.turned[2].bits(),
            effects.frame.draw_work.second[2].bits(),
        ] {
            if depth as i32 <= effects.frame.draw_work.near.bits() as i32 {
                return Ok(());
            }
        }
        ribbon(buffer, effects);
        if clip_quad(buffer, effects)? {
            return Ok(());
        }
        draw_fan(buffer, scene);
        crate::scene::polygon_work::front_end()
            .polygon_vertex_count
            .set_bits(4u32);
    }
    Ok(())
}

use crate::effects::frame_state::{DrawWork, ViewVars};
use crate::effects::plain::st_cell;
use crate::math::x87_float as bh;
use crate::math::x87_float::ld;

/// `ribbon`'s x87 block as plain Rust on the x87 model (93 instructions), same order and memory places.
fn ribbon_x87(buffer: &crate::d3d::vertices_store::PolygonVertices, dw: &DrawWork, vv: &ViewVars) {
    let t0 = ld(crate::scene::polygon_work::front_end()
        .window
        .view_scale
        .get());
    let t1 = t0 / dw.turned[2].ld();
    let t2 = t1 * dw.turned[0].ld();
    let t3 = t1 * dw.turned[1].ld();
    let t4 = -t3;
    let t5 = t4 + crate::scene::polygon_work::front_end().window.view_centre[1].ld();
    dw.turned[1].st(t5);
    let t6 = t2 + crate::scene::polygon_work::front_end().window.view_centre[0].ld();
    dw.turned[0].st(t6);
    dw.ends_scale[0].st(t1);
    let t7 = ld(crate::scene::polygon_work::front_end()
        .window
        .view_scale
        .get());
    let t8 = t7 / dw.second[2].ld();
    let t9 = t8 * dw.second[0].ld();
    let t10 = t8 * dw.second[1].ld();
    let t11 = -t10;
    let t12 = t11 + crate::scene::polygon_work::front_end().window.view_centre[1].ld();
    dw.second[1].st(t12);
    let t13 = t9 + crate::scene::polygon_work::front_end().window.view_centre[0].ld();
    dw.second[0].st(t13);
    dw.ends_scale[1].st(t8);
    let t14 = dw.second[0].ld();
    let t15 = t14 - dw.turned[0].ld();
    vv.ribbon_way[0].st(t15);
    let t16 = dw.second[1].ld();
    let t17 = t16 - dw.turned[1].ld();
    vv.ribbon_way[1].st(t17);
    let t18 = vv.ribbon_way[0].ld();
    let t19 = t18 * t18;
    let t20 = vv.ribbon_way[1].ld();
    let t21 = t20 * t20;
    let t22 = t19 + t21;
    let t23 = t22.sqrt();
    let t24 = bh::ONE;
    let t25 = t24 / t23;
    let t26 = vv.ribbon_way[0].ld();
    let t27 = vv.ribbon_way[1].ld();
    let t28 = t27 * t25;
    vv.ribbon_way[0].st(t28);
    let t29 = t25 * t26;
    let t30 = -t29;
    vv.ribbon_way[1].st(t30);
    let t31 = vv.ribbon_way[0].ld();
    let t32 = t31 * bh::int(dw.half[0].bits() as i32);
    let t33 = t32 * dw.ends_scale[0].ld();
    vv.first_end_half_width[0].st(t33);
    let t34 = vv.ribbon_way[1].ld();
    let t35 = t34 * bh::int(dw.half[0].bits() as i32);
    let t36 = t35 * dw.ends_scale[0].ld();
    vv.first_end_half_width[1].st(t36);
    let t37 = vv.ribbon_way[0].ld();
    let t38 = t37 * bh::int(dw.half[0].bits() as i32);
    let t39 = t38 * dw.ends_scale[1].ld();
    vv.second_end_half_width[0].st(t39);
    let t40 = vv.ribbon_way[1].ld();
    let t41 = t40 * bh::int(dw.half[0].bits() as i32);
    let t42 = t41 * dw.ends_scale[1].ld();
    vv.second_end_half_width[1].st(t42);
    let t43 = dw.turned[0].ld();
    let t44 = t43 - vv.first_end_half_width[0].ld();
    st_cell(&buffer.vertices[0].x, t44);
    let t45 = dw.turned[1].ld();
    let t46 = t45 - vv.first_end_half_width[1].ld();
    st_cell(&buffer.vertices[0].y, t46);
    let t47 = dw.turned[0].ld();
    let t48 = t47 + vv.first_end_half_width[0].ld();
    st_cell(&buffer.vertices[1].x, t48);
    let t49 = dw.turned[1].ld();
    let t50 = t49 + vv.first_end_half_width[1].ld();
    st_cell(&buffer.vertices[1].y, t50);
    let t51 = dw.second[0].ld();
    let t52 = t51 + vv.second_end_half_width[0].ld();
    st_cell(&buffer.vertices[2].x, t52);
    let t53 = dw.second[1].ld();
    let t54 = t53 + vv.second_end_half_width[1].ld();
    st_cell(&buffer.vertices[2].y, t54);
    let t55 = dw.second[0].ld();
    let t56 = t55 - vv.second_end_half_width[0].ld();
    st_cell(&buffer.vertices[3].x, t56);
    let t57 = dw.second[1].ld();
    let t58 = t57 - vv.second_end_half_width[1].ld();
    st_cell(&buffer.vertices[3].y, t58);
    let t59 = dw.turned[2].ld();
    let t60 = t59 * dw.z_k.ld();
    st_cell(&buffer.vertices[0].z, t60);
    st_cell(&buffer.vertices[1].z, t60);
    let t61 = dw.second[2].ld();
    let t62 = t61 * dw.z_k.ld();
    st_cell(&buffer.vertices[2].z, t62);
    st_cell(&buffer.vertices[3].z, t62);
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

    /// The vertex float at byte `off` of the vertices (x, y or z of a vertex).
    fn vertex_cell(off: usize) -> &'static crate::session::Float {
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
                            || super::ribbon_x87(&crate::d3d::vertices_store::VERTICES, dw, vv),
                        );
                    });
                });
            },
        );
    }
}
