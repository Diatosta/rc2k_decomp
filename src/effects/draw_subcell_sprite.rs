//! `effects::draw_subcell_sprite`: a ring entry's square sprite from a subcell: the place
//! (x, y, z) turned by the view matrix into the draw work's place (the depth also whole in
//! the ring's `whole`); unless the depth is at or below the nearest depth (a signed compare
//! of the bits): the four vertices' specular `scene::fog_at` of the depth, colour the
//! entry's car tag | 0xFF000000, texture places from the subcell's entry in the subcell
//! places, depth times the depth scale; half the side (the size's bits as a float, times
//! the view scale / depth); the corners about the projected place; `effects::clip_quad`,
//! and unless all is outside `scene::draw_fan` and the polygon's vertex count 4.
//!
//! The x87 blocks are plain Rust on the x87 model (`math::x87_float`). Refuses, before anything, a
//! subcell past the table's 0x400, a null entry, and the fog's and the clip's refusals.

use crate::effects::clip_quad::{clip_quad, draw_fan};
use crate::effects::mask_store::Masks;
use crate::effects::ring_store::Ring;
use crate::effects::ring_store::RingWork;
use crate::errors::{BadInput, Refused};
use crate::math::cell_x87::CellX87;
use crate::scene::fog_at::fog_at;

/// The sub-cell places in the surface's table. The bursts, the only entries drawn this way,
/// number theirs below 0x3F7.
const SUBCELLS: u32 = 0x400;

/// The place turned by the view.
fn turn(
    view_matrix: &[crate::memory::session::PlainFloat; 16],
    ring: &Ring,
    p: &[[u8; 10]; 3],
    effects: &crate::effects::owner::Effects,
) {
    turn_x87(view_matrix, p, &ring.work, &effects.frame.draw_work);
}

/// The square about the projected place.
fn square(
    front: &crate::scene::polygon_work::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    effects: &crate::effects::owner::Effects,
) {
    square_x87(front, buffer, &effects.frame.draw_work);
}

/// `subcell` is the subcell's number, `size` a float's bits, `entry` the ring entry.
#[allow(
    clippy::too_many_arguments,
    reason = "the original's registers, the effects' groups and the owners"
)]
pub(super) fn draw_subcell_sprite(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    masks: &Masks,
    ring: &Ring,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    subcell: u32,
    size: u32,
    entry: u32,
    place: [[u8; 10]; 3],
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<(), Refused> {
    if subcell >= SUBCELLS {
        return Err(BadInput("a mark subcell past g_marksSubcellPlaces").into());
    }
    if entry == 0 {
        return Err(BadInput("a null ring entry").into());
    }
    fog_at(view.fog, 0)?;
    if effects.setup.settings.is_set.get() == 0 {
        return Err(BadInput("no marks settings").into());
    }
    turn(view.view_matrix, ring, &place, effects);
    if crate::math::x87::bits_at_or_below(
        effects.frame.draw_work.turned[2].get(),
        effects.frame.draw_work.near.get(),
    ) {
        return Ok(());
    }
    let fogged = fog_at(view.fog, ring.work.whole.get())?;
    let colour = crate::effects::ring_entries::entry(ring, entry)?.car_tag | 0xFF00_0000;
    for item in buffer.vertices.iter().take(4) {
        item.specular.set(fogged);
        item.colour.set(colour);
    }
    effects.frame.draw_work.half[0].set(f32::from_bits(size));
    let at = subcell << 4;
    let place =
        |n: usize| f32::from_bits(masks.surface.places.get_u32(0x400 + at as usize + 4 * n));
    let [u0, v0, u1, v1] = [place(0), place(1), place(2), place(3)];
    for (n, (u, v)) in [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
        .into_iter()
        .enumerate()
    {
        buffer.vertices[n].u.set(u);
        buffer.vertices[n].v.set(v);
    }
    square(scene.scratch.front_end_polygon(), buffer, effects);
    if clip_quad(buffer, scene, effects)? {
        return Ok(());
    }
    draw_fan(buffer, scene);
    scene
        .scratch
        .front_end_polygon()
        .polygon_vertex_count
        .set(4);
    Ok(())
}

use crate::effects::frame_state::DrawWork;
use crate::effects::plain::st_cell;
use crate::math::x87_float as bh;
use crate::math::x87_float::ld;

/// `turn`'s arithmetic as plain Rust on the x87 model, in the same order.
fn turn_x87(
    view_matrix: &[crate::memory::session::PlainFloat; 16],
    p: &[[u8; 10]; 3],
    w: &RingWork,
    dw: &DrawWork,
) {
    let t0 = bh::from_f80(p[2]);
    let t1 = bh::from_f80(p[1]);
    let t2 = bh::from_f80(p[0]);
    let t3 = view_matrix[0].ld();
    let t4 = t3 * t2;
    let t5 = view_matrix[4].ld();
    let t6 = t5 * t1;
    let t7 = view_matrix[8].ld();
    let t8 = t7 * t0;
    let t9 = t6 + t8;
    let t10 = t4 + t9;
    let t11 = t10 + view_matrix[12].ld();
    let t12 = view_matrix[1].ld();
    let t13 = t12 * t2;
    let t14 = view_matrix[5].ld();
    let t15 = t14 * t1;
    let t16 = view_matrix[9].ld();
    let t17 = t16 * t0;
    let t18 = t15 + t17;
    let t19 = t13 + t18;
    let t20 = t19 + view_matrix[13].ld();
    let t21 = view_matrix[2].ld();
    let t22 = t2 * t21;
    let t23 = view_matrix[6].ld();
    let t24 = t1 * t23;
    let t25 = view_matrix[10].ld();
    let t26 = t0 * t25;
    let t27 = t26 + t24;
    let t28 = t22 + t27;
    let t29 = t28 + view_matrix[14].ld();
    dw.turned[0].st(t11);
    dw.turned[1].st(t20);
    w.whole.set(bh::to_int(t29) as u32);
    dw.turned[2].st(t29);
}

/// `square`'s arithmetic as plain Rust on the x87 model, in the same order.
fn square_x87(
    front: &crate::scene::polygon_work::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    dw: &DrawWork,
) {
    let t0 = ld(front.window.view_scale.get());
    let t1 = t0 / dw.turned[2].ld();
    let t2 = dw.turned[2].ld();
    let t3 = t2 * dw.z_k.ld();
    st_cell(&buffer.vertices[0].z, t3);
    st_cell(&buffer.vertices[1].z, t3);
    st_cell(&buffer.vertices[2].z, t3);
    st_cell(&buffer.vertices[3].z, t3);
    let t4 = dw.half[0].ld();
    let t5 = t4 * t1;
    dw.half[0].st(t5);
    dw.half[1].st(t5);
    let t6 = t1 * dw.turned[0].ld();
    let t7 = t1 * dw.turned[1].ld();
    let t8 = -t7;
    let t9 = t8 + front.window.view_centre[1].ld();
    let t10 = t9 - dw.half[1].ld();
    st_cell(&buffer.vertices[0].y, t10);
    let t11 = t9 - dw.half[1].ld();
    st_cell(&buffer.vertices[1].y, t11);
    let t12 = t9 + dw.half[1].ld();
    st_cell(&buffer.vertices[2].y, t12);
    let t13 = t9 + dw.half[1].ld();
    st_cell(&buffer.vertices[3].y, t13);
    let t14 = t6 + front.window.view_centre[0].ld();
    let t15 = t14 - dw.half[0].ld();
    st_cell(&buffer.vertices[0].x, t15);
    let t16 = t14 + dw.half[0].ld();
    st_cell(&buffer.vertices[1].x, t16);
    let t17 = t14 + dw.half[0].ld();
    st_cell(&buffer.vertices[2].x, t17);
    let t18 = t14 - dw.half[0].ld();
    st_cell(&buffer.vertices[3].x, t18);
}

#[cfg(test)]
mod tests;
