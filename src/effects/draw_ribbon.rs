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

use crate::effects::clip_quad::{clip_quad, draw_fan};
use crate::effects::mask_store::Masks;
use crate::errors::{BadInput, Refused};
use crate::math::x87;

const CELLS: u32 = 0x41;

/// The two places projected and the ribbon's corners.
fn ribbon(
    front: &crate::scene::polygon_work_store::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    effects: &crate::effects::owner::Effects,
) {
    let frame = &effects.frame;
    ribbon_x87(
        front,
        buffer,
        &mut frame.draw_work.borrow_mut(),
        &mut frame.view_vars.borrow_mut(),
    );
}

/// Draws a ribbon piece of `colour` and `alpha` and `width`, with the texture places of
/// `cell`.
#[allow(
    clippy::too_many_arguments,
    reason = "the effects' groups, the owners and the vertex buffer"
)]
pub(super) fn draw_ribbon(
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
        effects.frame.draw_work.borrow_mut().half[0] = f32::from_bits(width >> 1);
        let colour = (alpha << 24) | colour;
        for item in buffer.vertices.iter().take(4) {
            item.colour.set(colour);
        }
        let at = cell << 4;
        let place =
            |n: usize| f32::from_bits(masks.surface.borrow().places.get_u32(at as usize + 4 * n));
        let [u0, v0, u1, v1] = [place(0), place(1), place(2), place(3)];
        for (n, (u, v)) in [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
            .into_iter()
            .enumerate()
        {
            buffer.vertices[n].u.set(u);
            buffer.vertices[n].v.set(v);
        }
        for depth in [
            effects.frame.draw_work.borrow().turned[2],
            effects.frame.draw_work.borrow().second[2],
        ] {
            if x87::bits_at_or_below(depth, effects.frame.draw_work.borrow().near) {
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

use crate::effects::frame_state_store::{DrawWork, ViewVars};
use crate::effects::plain::st_cell;
use crate::math::x87_float as bh;
use crate::math::x87_float::ld;

/// `ribbon`'s arithmetic as plain Rust on the x87 model, in the same order.
fn ribbon_x87(
    front: &crate::scene::polygon_work_store::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    dw: &mut DrawWork,
    vv: &mut ViewVars,
) {
    let t0 = ld(front.window.borrow().view_scale);
    let t1 = t0 / bh::ld(dw.turned[2]);
    let t2 = t1 * bh::ld(dw.turned[0]);
    let t3 = t1 * bh::ld(dw.turned[1]);
    let t4 = -t3;
    let t5 = t4 + crate::math::x87_float::ld(front.window.borrow().view_centre[1]);
    dw.turned[1] = bh::stf(t5);
    let t6 = t2 + crate::math::x87_float::ld(front.window.borrow().view_centre[0]);
    dw.turned[0] = bh::stf(t6);
    dw.ends_scale[0] = bh::stf(t1);
    let t7 = ld(front.window.borrow().view_scale);
    let t8 = t7 / bh::ld(dw.second[2]);
    let t9 = t8 * bh::ld(dw.second[0]);
    let t10 = t8 * bh::ld(dw.second[1]);
    let t11 = -t10;
    let t12 = t11 + crate::math::x87_float::ld(front.window.borrow().view_centre[1]);
    dw.second[1] = bh::stf(t12);
    let t13 = t9 + crate::math::x87_float::ld(front.window.borrow().view_centre[0]);
    dw.second[0] = bh::stf(t13);
    dw.ends_scale[1] = bh::stf(t8);
    let t14 = bh::ld(dw.second[0]);
    let t15 = t14 - bh::ld(dw.turned[0]);
    vv.ribbon_way[0] = bh::stf(t15);
    let t16 = bh::ld(dw.second[1]);
    let t17 = t16 - bh::ld(dw.turned[1]);
    vv.ribbon_way[1] = bh::stf(t17);
    let t18 = bh::ld(vv.ribbon_way[0]);
    let t19 = t18 * t18;
    let t20 = bh::ld(vv.ribbon_way[1]);
    let t21 = t20 * t20;
    let t22 = t19 + t21;
    let t23 = t22.sqrt();
    let t24 = bh::ONE;
    let t25 = t24 / t23;
    let t26 = bh::ld(vv.ribbon_way[0]);
    let t27 = bh::ld(vv.ribbon_way[1]);
    let t28 = t27 * t25;
    vv.ribbon_way[0] = bh::stf(t28);
    let t29 = t25 * t26;
    let t30 = -t29;
    vv.ribbon_way[1] = bh::stf(t30);
    let t31 = bh::ld(vv.ribbon_way[0]);
    let t32 = t31 * bh::int(dw.half[0].to_bits() as i32);
    let t33 = t32 * bh::ld(dw.ends_scale[0]);
    vv.first_end_half_width[0] = bh::stf(t33);
    let t34 = bh::ld(vv.ribbon_way[1]);
    let t35 = t34 * bh::int(dw.half[0].to_bits() as i32);
    let t36 = t35 * bh::ld(dw.ends_scale[0]);
    vv.first_end_half_width[1] = bh::stf(t36);
    let t37 = bh::ld(vv.ribbon_way[0]);
    let t38 = t37 * bh::int(dw.half[0].to_bits() as i32);
    let t39 = t38 * bh::ld(dw.ends_scale[1]);
    vv.second_end_half_width[0] = bh::stf(t39);
    let t40 = bh::ld(vv.ribbon_way[1]);
    let t41 = t40 * bh::int(dw.half[0].to_bits() as i32);
    let t42 = t41 * bh::ld(dw.ends_scale[1]);
    vv.second_end_half_width[1] = bh::stf(t42);
    let t43 = bh::ld(dw.turned[0]);
    let t44 = t43 - bh::ld(vv.first_end_half_width[0]);
    st_cell(&buffer.vertices[0].x, t44);
    let t45 = bh::ld(dw.turned[1]);
    let t46 = t45 - bh::ld(vv.first_end_half_width[1]);
    st_cell(&buffer.vertices[0].y, t46);
    let t47 = bh::ld(dw.turned[0]);
    let t48 = t47 + bh::ld(vv.first_end_half_width[0]);
    st_cell(&buffer.vertices[1].x, t48);
    let t49 = bh::ld(dw.turned[1]);
    let t50 = t49 + bh::ld(vv.first_end_half_width[1]);
    st_cell(&buffer.vertices[1].y, t50);
    let t51 = bh::ld(dw.second[0]);
    let t52 = t51 + bh::ld(vv.second_end_half_width[0]);
    st_cell(&buffer.vertices[2].x, t52);
    let t53 = bh::ld(dw.second[1]);
    let t54 = t53 + bh::ld(vv.second_end_half_width[1]);
    st_cell(&buffer.vertices[2].y, t54);
    let t55 = bh::ld(dw.second[0]);
    let t56 = t55 - bh::ld(vv.second_end_half_width[0]);
    st_cell(&buffer.vertices[3].x, t56);
    let t57 = bh::ld(dw.second[1]);
    let t58 = t57 - bh::ld(vv.second_end_half_width[1]);
    st_cell(&buffer.vertices[3].y, t58);
    let t59 = bh::ld(dw.turned[2]);
    let t60 = t59 * bh::ld(dw.z_k);
    st_cell(&buffer.vertices[0].z, t60);
    st_cell(&buffer.vertices[1].z, t60);
    let t61 = bh::ld(dw.second[2]);
    let t62 = t61 * bh::ld(dw.z_k);
    st_cell(&buffer.vertices[2].z, t62);
    st_cell(&buffer.vertices[3].z, t62);
}

#[cfg(test)]
mod tests;
