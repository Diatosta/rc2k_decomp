//! `effects::draw_windscreen_flake`: one windscreen flake item drawn on the windscreen flakes'
//! frame. The four vertices' texture places from `places` (u a, c, c, a; v b, b, d, d) plus the
//! item's `texture_u` / `texture_v`; its place (the across share times `lengths[0]`, the along
//! share times `lengths[1]`) along the two axes from the frame's place into `corners`; less
//! `half_axes`; the four corners (plus, less the scaled axes) into `corners`; unless the first
//! corner's depth is at or below 4096.0 (a signed compare of the bits): projected (the view scale
//! over each depth, the view's centre), depth (plus the kind float, times `inverse`), colour,
//! `scene::draw_fan`.
//!
//! The x87 runs are plain Rust on the x87 model (`math::x87_float`).

use crate::effects::tuning_store::MarksKindFloats;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::math::cell_x87::CellX87;
use crate::memory::session::PlainFloat;

const FAR_ENOUGH: f32 = 4096.0;

/// The item's texture shift added to the places.
fn shifted_places(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    item: &WindscreenFlakeItem,
) {
    shifted_places_x87(buffer, item);
}

/// The corners, from the two values on the x87 stack.
fn corners(fl: &WindscreenFlakes, v: &[[u8; 10]; 2]) {
    corners_x87(v, &fl.tables);
}

/// The corners projected, and their depth.
fn projected(
    front: &crate::scene::polygon_work_store::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    fl: &WindscreenFlakes,
    effects: &crate::effects::owner::Effects,
) {
    projected_x87(front, buffer, &effects.setup.kind_floats, &fl.tables);
}

/// Draws `item` in `colour`.
pub(super) fn draw_windscreen_flake(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    fl: &WindscreenFlakes,
    item: &WindscreenFlakeItem,
    colour: u32,
    shares: [[u8; 10]; 2],
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) {
    let [a, pb, c, d] = fl.state.places.each_ref().map(PlainFloat::get);
    for (n, (u, v)) in [(a, pb), (c, pb), (c, d), (a, d)].into_iter().enumerate() {
        buffer.vertices[n].u.set(u);
        buffer.vertices[n].v.set(v);
    }
    shifted_places(buffer, item);
    corners(fl, &shares);
    if crate::math::x87::bits_at_or_below(fl.tables.corners[0][2].get(), FAR_ENOUGH) {
        return;
    }
    projected(scene.scratch.front_end_polygon(), buffer, fl, effects);
    for item in buffer.vertices.iter().take(4) {
        item.colour.set(colour);
    }
    crate::effects::clip_quad::draw_fan(buffer, scene);
}

use crate::effects::plain::{ld_cell, st_cell};
use crate::effects::windscreen_flakes_store::{WindscreenFlakeItem, WindscreenFlakeTables};
use crate::math::x87_float as bh;

/// `shifted_places`'s arithmetic as plain Rust on the x87 model, in the same order.
fn shifted_places_x87(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    item: &WindscreenFlakeItem,
) {
    let t0 = ld_cell(&item.texture_u);
    let t1 = t0 + ld_cell(&buffer.vertices[0].u);
    st_cell(&buffer.vertices[0].u, t1);
    let t2 = ld_cell(&item.texture_u);
    let t3 = t2 + ld_cell(&buffer.vertices[1].u);
    st_cell(&buffer.vertices[1].u, t3);
    let t4 = ld_cell(&item.texture_u);
    let t5 = t4 + ld_cell(&buffer.vertices[2].u);
    st_cell(&buffer.vertices[2].u, t5);
    let t6 = ld_cell(&item.texture_u);
    let t7 = t6 + ld_cell(&buffer.vertices[3].u);
    st_cell(&buffer.vertices[3].u, t7);
    let t8 = ld_cell(&item.texture_v);
    let t9 = t8 + ld_cell(&buffer.vertices[0].v);
    st_cell(&buffer.vertices[0].v, t9);
    let t10 = ld_cell(&item.texture_v);
    let t11 = t10 + ld_cell(&buffer.vertices[1].v);
    st_cell(&buffer.vertices[1].v, t11);
    let t12 = ld_cell(&item.texture_v);
    let t13 = t12 + ld_cell(&buffer.vertices[2].v);
    st_cell(&buffer.vertices[2].v, t13);
    let t14 = ld_cell(&item.texture_v);
    let t15 = t14 + ld_cell(&buffer.vertices[3].v);
    st_cell(&buffer.vertices[3].v, t15);
}

/// `corners`'s arithmetic as plain Rust on the x87 model, in the same order.
fn corners_x87(v: &[[u8; 10]; 2], sq: &WindscreenFlakeTables) {
    let t0 = bh::from_f80(v[1]);
    let t1 = bh::from_f80(v[0]);
    let t2 = t1 * sq.lengths[1].ld();
    let t3 = t0 * sq.lengths[0].ld();
    let t4 = t3 * sq.axes[0][0].ld();
    let t5 = t2 * sq.axes[1][0].ld();
    let t6 = t4 + t5;
    let t7 = t6 + sq.place[0].ld();
    sq.corners[0][0].st(t7);
    let t8 = t3 * sq.axes[0][1].ld();
    let t9 = t2 * sq.axes[1][1].ld();
    let t10 = t8 + t9;
    let t11 = t10 + sq.place[1].ld();
    sq.corners[0][1].st(t11);
    let t12 = t2 * sq.axes[1][2].ld();
    let t13 = t3 * sq.axes[0][2].ld();
    let t14 = t12 + t13;
    let t15 = t14 + sq.place[2].ld();
    sq.corners[0][2].st(t15);
    let t16 = sq.corners[0][0].ld();
    let t17 = t16 - sq.half_axes[0][0].ld();
    let t18 = t17 - sq.half_axes[1][0].ld();
    sq.corners[0][0].st(t18);
    let t19 = t18 + sq.scaled_by_radius[1][0].ld();
    sq.corners[1][0].st(t19);
    let t20 = t19 + sq.scaled_by_radius[0][0].ld();
    sq.corners[2][0].st(t20);
    let t21 = t20 - sq.scaled_by_radius[1][0].ld();
    sq.corners[3][0].st(t21);
    let t22 = sq.corners[0][1].ld();
    let t23 = t22 - sq.half_axes[0][1].ld();
    let t24 = t23 - sq.half_axes[1][1].ld();
    sq.corners[0][1].st(t24);
    let t25 = t24 + sq.scaled_by_radius[1][1].ld();
    sq.corners[1][1].st(t25);
    let t26 = t25 + sq.scaled_by_radius[0][1].ld();
    sq.corners[2][1].st(t26);
    let t27 = t26 - sq.scaled_by_radius[1][1].ld();
    sq.corners[3][1].st(t27);
    let t28 = sq.corners[0][2].ld();
    let t29 = t28 - sq.half_axes[0][2].ld();
    let t30 = t29 - sq.half_axes[1][2].ld();
    sq.corners[0][2].st(t30);
    let t31 = t30 + sq.scaled_by_radius[1][2].ld();
    sq.corners[1][2].st(t31);
    let t32 = t31 + sq.scaled_by_radius[0][2].ld();
    sq.corners[2][2].st(t32);
    let t33 = t32 - sq.scaled_by_radius[1][2].ld();
    sq.corners[3][2].st(t33);
}

/// `projected`'s arithmetic as plain Rust on the x87 model, in the same order.
fn projected_x87(
    front: &crate::scene::polygon_work_store::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    k: &MarksKindFloats,
    sq: &WindscreenFlakeTables,
) {
    let t0 = crate::math::x87_float::ld(front.window.view_scale.get());
    let t1 = t0 / sq.corners[3][2].ld();
    let t2 = crate::math::x87_float::ld(front.window.view_scale.get());
    let t3 = t2 / sq.corners[2][2].ld();
    let t4 = crate::math::x87_float::ld(front.window.view_scale.get());
    let t5 = t4 / sq.corners[1][2].ld();
    let t6 = crate::math::x87_float::ld(front.window.view_scale.get());
    let t7 = t6 / sq.corners[0][2].ld();
    let t8 = sq.corners[0][1].ld();
    let t9 = t8 * t7;
    let t10 = t7 * sq.corners[0][0].ld();
    let t11 = -t9;
    let t12 = t11 + crate::math::x87_float::ld(front.window.view_centre[1].get());
    st_cell(&buffer.vertices[0].y, t12);
    let t13 = t10 + crate::math::x87_float::ld(front.window.view_centre[0].get());
    st_cell(&buffer.vertices[0].x, t13);
    let t14 = sq.corners[1][1].ld();
    let t15 = t14 * t5;
    let t16 = t5 * sq.corners[1][0].ld();
    let t17 = -t15;
    let t18 = t17 + crate::math::x87_float::ld(front.window.view_centre[1].get());
    st_cell(&buffer.vertices[1].y, t18);
    let t19 = t16 + crate::math::x87_float::ld(front.window.view_centre[0].get());
    st_cell(&buffer.vertices[1].x, t19);
    let t20 = sq.corners[2][1].ld();
    let t21 = t20 * t3;
    let t22 = t3 * sq.corners[2][0].ld();
    let t23 = -t21;
    let t24 = t23 + crate::math::x87_float::ld(front.window.view_centre[1].get());
    st_cell(&buffer.vertices[2].y, t24);
    let t25 = t22 + crate::math::x87_float::ld(front.window.view_centre[0].get());
    st_cell(&buffer.vertices[2].x, t25);
    let t26 = sq.corners[3][1].ld();
    let t27 = t26 * t1;
    let t28 = t1 * sq.corners[3][0].ld();
    let t29 = -t27;
    let t30 = t29 + crate::math::x87_float::ld(front.window.view_centre[1].get());
    st_cell(&buffer.vertices[3].y, t30);
    let t31 = t28 + crate::math::x87_float::ld(front.window.view_centre[0].get());
    st_cell(&buffer.vertices[3].x, t31);
    let t32 = sq.corners[0][2].ld();
    let t33 = t32 + k.kind_float.ld();
    let t34 = t33 * sq.inverse.ld();
    let t35 = sq.corners[1][2].ld();
    let t36 = t35 + k.kind_float.ld();
    let t37 = t36 * sq.inverse.ld();
    let t38 = sq.corners[2][2].ld();
    let t39 = t38 + k.kind_float.ld();
    let t40 = t39 * sq.inverse.ld();
    let t41 = sq.corners[3][2].ld();
    let t42 = t41 + k.kind_float.ld();
    let t43 = t42 * sq.inverse.ld();
    st_cell(&buffer.vertices[3].z, t43);
    st_cell(&buffer.vertices[2].z, t40);
    st_cell(&buffer.vertices[1].z, t37);
    st_cell(&buffer.vertices[0].z, t34);
}

#[cfg(test)]
mod tests;
