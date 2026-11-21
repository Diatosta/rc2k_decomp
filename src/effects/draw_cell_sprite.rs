//! `effects::draw_cell_sprite`: a ring entry's sprite: the place (x, y, z) turned by the view
//! matrix into the draw work's place (the depth also whole in the ring's `whole`); unless the depth
//! is at or below the nearest depth (a signed compare of the bits): the four vertices' specular
//! `scene::fog_at` of the depth, colour (the alpha's top byte | the entry's car tag), texture
//! places from the cell's entry in the cell places, depth times the depth scale; half the side (the
//! size times the view scale / depth, and that times the cell's corner slope) turned by the sine
//! and cosine tables at the angle; the corners about the projected place; `effects::clip_quad`, and
//! unless all is outside `scene::draw_fan` and the polygon's vertex count 4.
//!
//! The x87 blocks are plain Rust on the x87 model (`math::x87_float`). Refuses, before anything, a
//! cell past 0x40, an angle of 0x1000 or more, a null entry, and the fog's and the clip's refusals.

use crate::effects::clip_quad::{clip_quad, draw_fan};
use crate::effects::mask_store::Masks;
use crate::effects::ring_store::Ring;
use crate::effects::ring_store::RingWork;
use crate::errors::{BadInput, Refused};
use crate::math::cell_x87::CellX87;
use crate::scene::fog_at::fog_at;

const CELLS: u32 = 0x41;
const ANGLES: u32 = 0x1000;

/// The place turned by the view.
fn turn(
    view_matrix: &[crate::memory::session::PlainFloat; 16],
    ring: &Ring,
    p: &[[u8; 10]; 3],
    effects: &crate::effects::owner::Effects,
) {
    turn_x87(
        view_matrix,
        p,
        &mut ring.work.borrow_mut(),
        &mut effects.frame.draw_work.borrow_mut(),
    );
}

/// The square, turned, about the projected place. Gives a value kept in the draw work's
/// `DrawWork.sprite_half[1]` cell between its two stores.
fn square(
    front: &crate::scene::polygon_work_store::PolygonWork,
    engine_tables: &crate::math::engine_tables::EngineTables,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    k: u32,
    angle: u32,
    effects: &crate::effects::owner::Effects,
) -> u32 {
    square_x87(
        front,
        engine_tables,
        buffer,
        k as usize,
        angle as usize,
        &mut effects.frame.draw_work.borrow_mut(),
        &effects.blocks.corner_slopes.borrow(),
    )
}

/// `cell` is the cell's number, `angle` the turn, `alpha` the colour's top byte, `size` the
/// side, `entry` the ring entry.
#[allow(
    clippy::too_many_arguments,
    reason = "the original's five registers and the cell places, and now the two owners"
)]
pub(super) fn draw_cell_sprite(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    masks: &Masks,
    ring: &Ring,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    cell: u32,
    angle: u32,
    alpha: u32,
    size: u32,
    entry: u32,
    place: [[u8; 10]; 3],
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<(), Refused> {
    if cell >= CELLS {
        return Err(BadInput("a mark cell past g_marksCellPlaces").into());
    }
    if angle >= ANGLES {
        return Err(BadInput("an angle past the sine tables").into());
    }
    if entry == 0 {
        return Err(BadInput("a null ring entry").into());
    }
    // The fog's and the clip's refusals, before anything.
    fog_at(view.fog, 0)?;
    if effects.setup.settings.is_set.get() == 0 {
        return Err(BadInput("no marks settings").into());
    }
    turn(view.view_matrix, ring, &place, effects);
    if crate::math::x87::bits_at_or_below(
        effects.frame.draw_work.borrow().turned[2],
        effects.frame.draw_work.borrow().near,
    ) {
        return Ok(());
    }
    let fogged = fog_at(view.fog, ring.work.borrow().whole)?;
    let colour = (alpha & 0xFF00_0000) | crate::effects::ring_entries::entry(ring, entry)?.car_tag;
    for item in buffer.vertices.iter().take(4) {
        item.specular.set(fogged);
        item.colour.set(colour);
    }
    effects.frame.draw_work.borrow_mut().sprite_half[0] = size;
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
    square(
        scene.scratch.front_end_polygon(),
        view.engine_tables,
        buffer,
        at,
        angle,
        effects,
    );
    if clip_quad(buffer, scene, effects)? {
        return Ok(());
    }
    draw_fan(buffer, scene);
    scene
        .scratch
        .front_end_polygon()
        .scalars
        .borrow_mut()
        .polygon_vertex_count = 4;
    Ok(())
}

use crate::effects::blocks_store::CornerSlopes;
use crate::effects::frame_state_store::DrawWork;
use crate::effects::plain::{ld_cell, st_cell};
use crate::math::engine_tables::{cosine, sine};
use crate::math::x87_float as bh;

/// `turn`'s arithmetic as plain Rust on the x87 model, in the same order.
fn turn_x87(
    view_matrix: &[crate::memory::session::PlainFloat; 16],
    p: &[[u8; 10]; 3],
    w: &mut RingWork,
    dw: &mut DrawWork,
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
    dw.turned[0] = bh::stf(t11);
    dw.turned[1] = bh::stf(t20);
    w.whole = bh::to_int(t29) as u32;
    dw.turned[2] = bh::stf(t29);
}

/// The float at byte `k` of the slope records, whatever field of a record it falls on.
fn slope_at(cs: &CornerSlopes, k: usize) -> f32 {
    let record = &cs.slopes[k / 0x24];
    match (k % 0x24) / 4 {
        0 => record.slope,
        n => record.values[n - 1],
    }
}

/// `square`'s arithmetic as plain Rust on the x87 model, in the same order. `k` is the
/// cell's offset in the cell table (16 bytes a cell), which is also added to the corner
/// slopes table (records of 0x24: the float at that byte is read, whatever field it falls
/// on; a slip kept as is), so that read goes through `slope_at`. `angle` is the sine
/// table's cell (the cosine is the cell 0x400 on).
fn square_x87(
    front: &crate::scene::polygon_work_store::PolygonWork,
    engine_tables: &crate::math::engine_tables::EngineTables,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    k: usize,
    angle: usize,
    dw: &mut DrawWork,
    cs: &CornerSlopes,
) -> u32 {
    let t0 = crate::math::x87_float::ld(front.window.borrow().view_scale);
    let t1 = t0 / bh::ld(dw.turned[2]);
    let t2 = bh::ld(dw.turned[2]);
    let t3 = t2 * bh::ld(dw.z_k);
    st_cell(&buffer.vertices[0].z, t3);
    st_cell(&buffer.vertices[1].z, t3);
    st_cell(&buffer.vertices[2].z, t3);
    st_cell(&buffer.vertices[3].z, t3);
    let t4 = bh::int(dw.sprite_half[0] as i32);
    let t5 = t4 * t1;
    dw.sprite_half[0] = bh::st(t5);
    let t6 = t5 * bh::ld(slope_at(cs, k));
    dw.sprite_half[1] = bh::st(t6);
    let t7 = dw.sprite_half[0];
    let t8 = t7 ^ 0x80000000;
    dw.sprite_half[3] = t8;
    let t9 = dw.sprite_half[1];
    dw.sprite_half[2] = t9;
    let t10 = bh::ld(dw.sprite_half[0]);
    let t11 = bh::ld(dw.sprite_half[1]);
    let t12 = t11 * ld_cell(cosine(engine_tables, angle));
    let t13 = t10 * ld_cell(sine(engine_tables, angle));
    let t14 = t12 + t13;
    dw.sprite_half[0] = bh::st(t14);
    let t15 = -t11;
    let t16 = t15 * ld_cell(sine(engine_tables, angle));
    let t17 = t10 * ld_cell(cosine(engine_tables, angle));
    let t18 = t16 + t17;
    dw.sprite_half[1] = bh::st(t18);
    let t19 = bh::ld(dw.sprite_half[2]);
    let t20 = bh::ld(dw.sprite_half[3]);
    let t21 = t20 * ld_cell(cosine(engine_tables, angle));
    let t22 = t19 * ld_cell(sine(engine_tables, angle));
    let t23 = t21 + t22;
    dw.sprite_half[2] = bh::st(t23);
    let t24 = -t20;
    let t25 = t24 * ld_cell(sine(engine_tables, angle));
    let t26 = t19 * ld_cell(cosine(engine_tables, angle));
    let t27 = t25 + t26;
    dw.sprite_half[3] = bh::st(t27);
    let t28 = t1 * bh::ld(dw.turned[0]);
    let t29 = t1 * bh::ld(dw.turned[1]);
    let t30 = -t29;
    let t31 = t30 + crate::math::x87_float::ld(front.window.borrow().view_centre[1]);
    let t32 = t31 - bh::ld(dw.sprite_half[1]);
    st_cell(&buffer.vertices[0].y, t32);
    let t33 = t31 - bh::ld(dw.sprite_half[3]);
    st_cell(&buffer.vertices[1].y, t33);
    let t34 = t31 + bh::ld(dw.sprite_half[1]);
    st_cell(&buffer.vertices[2].y, t34);
    let t35 = t31 + bh::ld(dw.sprite_half[3]);
    st_cell(&buffer.vertices[3].y, t35);
    let t36 = t28 + crate::math::x87_float::ld(front.window.borrow().view_centre[0]);
    let t37 = t36 - bh::ld(dw.sprite_half[0]);
    st_cell(&buffer.vertices[0].x, t37);
    let t38 = t36 - bh::ld(dw.sprite_half[2]);
    st_cell(&buffer.vertices[1].x, t38);
    let t39 = t36 + bh::ld(dw.sprite_half[0]);
    st_cell(&buffer.vertices[2].x, t39);
    let t40 = t36 + bh::ld(dw.sprite_half[2]);
    st_cell(&buffer.vertices[3].x, t40);
    t9
}

#[cfg(test)]
mod tests;
