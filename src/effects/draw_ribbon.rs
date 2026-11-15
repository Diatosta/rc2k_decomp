//! `effects::draw_ribbon`: a ribbon piece between two places already in the view's frame: half the
//! width (the width shifted right once, whole) kept, the four vertices' colour (alpha << 24) |
//! colour and texture places from the cell's entry in the cell places table; unless either depth is
//! at or below the nearest depth (a signed compare of the bits): both projected (each with its own
//! scale), the way between them made one long and turned a quarter, the sides that times the half
//! width and each scale, the four corners, the depths times the depth scale; `effects::clip_quad`,
//! and unless all is outside `scene::draw_fan` and the polygon's vertex count 4.
//!
//! The x87 block is plain Rust on the x87 model (`math::x87_float`). Refuses, before anything, a
//! cell past 0x40 and null marks settings (the clip's refusal).

use crate::effects::cell_x87::CellX87;
use crate::effects::clip_quad::{clip_quad, draw_fan};
use crate::effects::mask_store::Masks;
use crate::errors::{BadInput, Refused};
use crate::math::x87;

const CELLS: u32 = 0x41;

/// The two places projected and the ribbon's corners.
fn ribbon(
    front: &crate::scene::polygon_work::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    effects: &crate::effects::owner::Effects,
) {
    let frame = &effects.frame;
    ribbon_x87(front, buffer, &frame.draw_work, &frame.view_vars);
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
        // A whole number in the float cell.
        effects.frame.draw_work.half[0].set_bits(width >> 1);
        let colour = (alpha << 24) | colour;
        for item in buffer.vertices.iter().take(4) {
            item.colour.set(colour);
        }
        let at = cell << 4;
        let place = |n: usize| f32::from_bits(masks.surface.places.get_u32(at as usize + 4 * n));
        let [u0, v0, u1, v1] = [place(0), place(1), place(2), place(3)];
        for (n, (u, v)) in [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
            .into_iter()
            .enumerate()
        {
            buffer.vertices[n].u.set(u);
            buffer.vertices[n].v.set(v);
        }
        for depth in [
            effects.frame.draw_work.turned[2].get(),
            effects.frame.draw_work.second[2].get(),
        ] {
            if x87::bits_at_or_below(depth, effects.frame.draw_work.near.get()) {
                return Ok(());
            }
        }
        ribbon(scene.scratch.front_end_polygon(), buffer, effects);
        if clip_quad(buffer, scene, effects)? {
            return Ok(());
        }
        draw_fan(buffer, scene);
        scene
            .scratch
            .front_end_polygon()
            .polygon_vertex_count
            .set(4);
    }
    Ok(())
}

use crate::effects::frame_state::{DrawWork, ViewVars};
use crate::effects::plain::st_cell;
use crate::math::x87_float as bh;
use crate::math::x87_float::ld;

/// `ribbon`'s arithmetic as plain Rust on the x87 model, in the same order.
fn ribbon_x87(
    front: &crate::scene::polygon_work::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    dw: &DrawWork,
    vv: &ViewVars,
) {
    let t0 = ld(front.window.view_scale.get());
    let t1 = t0 / dw.turned[2].ld();
    let t2 = t1 * dw.turned[0].ld();
    let t3 = t1 * dw.turned[1].ld();
    let t4 = -t3;
    let t5 = t4 + front.window.view_centre[1].ld();
    dw.turned[1].st(t5);
    let t6 = t2 + front.window.view_centre[0].ld();
    dw.turned[0].st(t6);
    dw.ends_scale[0].st(t1);
    let t7 = ld(front.window.view_scale.get());
    let t8 = t7 / dw.second[2].ld();
    let t9 = t8 * dw.second[0].ld();
    let t10 = t8 * dw.second[1].ld();
    let t11 = -t10;
    let t12 = t11 + front.window.view_centre[1].ld();
    dw.second[1].st(t12);
    let t13 = t9 + front.window.view_centre[0].ld();
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

#[cfg(test)]
mod reference_vectors {
    use crate::effects::asm_layout::with_laid;
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::staged_vertices as staged;

    use crate::effects::asm_test::{Op, Spec, recorded_writes};
    use crate::effects::frame_state::{DrawWork, ViewVars};

    #[test]
    fn ribbon_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
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
        recorded_writes("effects.draw_ribbon.ribbon_x87", &SPEC, |b| {
            with_laid::<DrawWork, _>(b[2], |dw| {
                with_laid::<ViewVars, _>(b[2], |vv| {
                    staged(
                        b[1],
                        &[
                            0x0, 0x4, 0x8, 0x20, 0x24, 0x28, 0x40, 0x44, 0x48, 0x60, 0x64, 0x68,
                        ],
                        || {
                            super::ribbon_x87(
                                crate::owners::Owners::the()
                                    .scene
                                    .scratch
                                    .front_end_polygon(),
                                &crate::d3d::vertices_store::VERTICES,
                                dw,
                                vv,
                            );
                        },
                    );
                });
            });
        });
    }
}
