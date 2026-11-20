//! `effects::draw_pong_box`: one box of the pong game (ball, bat or a score cell) drawn on the
//! squares' frame: its height along the second axis times the flakes' second length into the sides
//! (and times a constant into the second pair), its width along the first times the first length
//! (likewise); the texture places from the pong's constants; its place (along, across) on the frame
//! into the corners; unless the first corner's depth is below the nearest depth: projected, depth
//! times the depth scale, colour -1, `scene::draw_fan`.
//!
//! The x87 runs are plain Rust on the x87 model (`math::x87_float`).

use crate::effects::pong_store::{Pong, PongState};
use crate::effects::tuning_store::MarksFloats;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::math::cell_x87::CellX87;
use crate::math::x87;

/// The sides, from the four values; the place's two values (left untouched) popped after.
fn sides(
    pong: &Pong,
    fl: &WindscreenFlakes,
    v: &[[u8; 10]; 4],
    effects: &crate::effects::owner::Effects,
) {
    sides_x87(
        v,
        &mut pong.state.borrow_mut(),
        &effects.setup.floats,
        &mut fl.tables.borrow_mut(),
    );
}

/// The corners, from the place.
fn corners(pong: &Pong, fl: &WindscreenFlakes, v: &[[u8; 10]; 4]) {
    corners_x87(v, &pong.state.borrow(), &mut fl.tables.borrow_mut());
}

/// The corners projected, and their depth.
fn projected(
    front: &crate::scene::polygon_work_store::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    fl: &WindscreenFlakes,
) {
    projected_x87(front, buffer, &mut fl.tables.borrow_mut());
}

pub(super) fn draw_pong_box(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    pong: &Pong,
    fl: &WindscreenFlakes,
    v: [[u8; 10]; 4],
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) {
    sides(pong, fl, &v, effects);
    let [pa, pb, pc, pd] = pong.rules.box_places;
    for (n, (u, w)) in [(pa, pb), (pc, pb), (pc, pd), (pa, pd)]
        .into_iter()
        .enumerate()
    {
        buffer.vertices[n].u.set(u);
        buffer.vertices[n].v.set(w);
    }
    corners(pong, fl, &v);
    if x87::compare_status(fl.tables.borrow().corners[0][2], pong.rules.near) == 0x100 {
        return;
    }
    projected(scene.scratch.front_end_polygon(), buffer, fl);
    for item in buffer.vertices.iter().take(4) {
        item.colour.set(0xFFFF_FFFFu32);
    }
    crate::effects::clip_quad::draw_fan(buffer, scene);
}

use crate::effects::plain::st_cell;
use crate::effects::windscreen_flakes_store::WindscreenFlakeTables;
use crate::math::x87_float as bh;

/// `sides`'s arithmetic as plain Rust on the x87 model, in the same order.
fn sides_x87(
    v: &[[u8; 10]; 4],
    p: &mut PongState,
    f: &MarksFloats,
    sq: &mut WindscreenFlakeTables,
) {
    let _t0 = bh::from_f80(v[3]);
    let _t1 = bh::from_f80(v[2]);
    let t2 = bh::from_f80(v[1]);
    let t3 = bh::from_f80(v[0]);
    let t4 = bh::ld(sq.axes[1][0]);
    let t5 = t4 * t3;
    let t6 = t5 * bh::ld(sq.lengths[1]);
    p.box_sides[3] = f32::from_bits(bh::st(t6));
    let t7 = t6 * bh::ld(f.half);
    p.box_sides[9] = f32::from_bits(bh::st(t7));
    let t8 = bh::ld(sq.axes[1][1]);
    let t9 = t8 * t3;
    let t10 = t9 * bh::ld(sq.lengths[1]);
    p.box_sides[4] = f32::from_bits(bh::st(t10));
    let t11 = t10 * bh::ld(f.half);
    p.box_sides[10] = f32::from_bits(bh::st(t11));
    let t12 = bh::ld(sq.axes[1][2]);
    let t13 = t3 * t12;
    let t14 = t13 * bh::ld(sq.lengths[1]);
    p.box_sides[5] = f32::from_bits(bh::st(t14));
    let t15 = t14 * bh::ld(f.half);
    p.box_sides[11] = f32::from_bits(bh::st(t15));
    let t16 = bh::ld(sq.axes[0][0]);
    let t17 = t16 * t2;
    let t18 = t17 * bh::ld(sq.lengths[0]);
    p.box_sides[0] = f32::from_bits(bh::st(t18));
    let t19 = t18 * bh::ld(f.half);
    p.box_sides[6] = f32::from_bits(bh::st(t19));
    let t20 = bh::ld(sq.axes[0][1]);
    let t21 = t20 * t2;
    let t22 = t21 * bh::ld(sq.lengths[0]);
    p.box_sides[1] = f32::from_bits(bh::st(t22));
    let t23 = t22 * bh::ld(f.half);
    p.box_sides[7] = f32::from_bits(bh::st(t23));
    let t24 = bh::ld(sq.axes[0][2]);
    let t25 = t2 * t24;
    let t26 = t25 * bh::ld(sq.lengths[0]);
    p.box_sides[2] = f32::from_bits(bh::st(t26));
    let t27 = t26 * bh::ld(f.half);
    p.box_sides[8] = f32::from_bits(bh::st(t27));
}

/// `corners`'s arithmetic as plain Rust on the x87 model, in the same order.
fn corners_x87(v: &[[u8; 10]; 4], p: &PongState, sq: &mut WindscreenFlakeTables) {
    let t0 = bh::from_f80(v[3]);
    let t1 = bh::from_f80(v[2]);
    let t2 = t1 * bh::ld(sq.lengths[1]);
    let t3 = t0 * bh::ld(sq.lengths[0]);
    let t4 = t3 * bh::ld(sq.axes[0][0]);
    let t5 = t2 * bh::ld(sq.axes[1][0]);
    let t6 = t4 + t5;
    let t7 = t6 + bh::ld(sq.place[0]);
    sq.corners[0][0] = f32::from_bits(bh::st(t7));
    let t8 = t3 * bh::ld(sq.axes[0][1]);
    let t9 = t2 * bh::ld(sq.axes[1][1]);
    let t10 = t8 + t9;
    let t11 = t10 + bh::ld(sq.place[1]);
    sq.corners[0][1] = f32::from_bits(bh::st(t11));
    let t12 = t2 * bh::ld(sq.axes[1][2]);
    let t13 = t3 * bh::ld(sq.axes[0][2]);
    let t14 = t12 + t13;
    let t15 = t14 + bh::ld(sq.place[2]);
    sq.corners[0][2] = f32::from_bits(bh::st(t15));
    let t16 = bh::ld(sq.corners[0][0]);
    sq.corners[0][0] = f32::from_bits(bh::st(t16));
    let t17 = t16 + bh::ld(p.box_sides[3]);
    sq.corners[1][0] = f32::from_bits(bh::st(t17));
    let t18 = t17 + bh::ld(p.box_sides[0]);
    sq.corners[2][0] = f32::from_bits(bh::st(t18));
    let t19 = t18 - bh::ld(p.box_sides[3]);
    sq.corners[3][0] = f32::from_bits(bh::st(t19));
    let t20 = bh::ld(sq.corners[0][1]);
    sq.corners[0][1] = f32::from_bits(bh::st(t20));
    let t21 = t20 + bh::ld(p.box_sides[4]);
    sq.corners[1][1] = f32::from_bits(bh::st(t21));
    let t22 = t21 + bh::ld(p.box_sides[1]);
    sq.corners[2][1] = f32::from_bits(bh::st(t22));
    let t23 = t22 - bh::ld(p.box_sides[4]);
    sq.corners[3][1] = f32::from_bits(bh::st(t23));
    let t24 = bh::ld(sq.corners[0][2]);
    sq.corners[0][2] = f32::from_bits(bh::st(t24));
    let t25 = t24 + bh::ld(p.box_sides[5]);
    sq.corners[1][2] = f32::from_bits(bh::st(t25));
    let t26 = t25 + bh::ld(p.box_sides[2]);
    sq.corners[2][2] = f32::from_bits(bh::st(t26));
    let t27 = t26 - bh::ld(p.box_sides[5]);
    sq.corners[3][2] = f32::from_bits(bh::st(t27));
}

/// `projected`'s arithmetic as plain Rust on the x87 model, in the same order.
fn projected_x87(
    front: &crate::scene::polygon_work_store::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    sq: &mut WindscreenFlakeTables,
) {
    let t0 = bh::ld(front.window.view_scale.get());
    let t1 = t0 / bh::ld(sq.corners[3][2]);
    let t2 = bh::ld(front.window.view_scale.get());
    let t3 = t2 / bh::ld(sq.corners[2][2]);
    let t4 = bh::ld(front.window.view_scale.get());
    let t5 = t4 / bh::ld(sq.corners[1][2]);
    let t6 = bh::ld(front.window.view_scale.get());
    let t7 = t6 / bh::ld(sq.corners[0][2]);
    let t8 = bh::ld(sq.corners[0][1]);
    let t9 = t8 * t7;
    let t10 = t7 * bh::ld(sq.corners[0][0]);
    let t11 = -t9;
    let t12 = t11 + front.window.view_centre[1].ld();
    st_cell(&buffer.vertices[0].y, t12);
    let t13 = t10 + front.window.view_centre[0].ld();
    st_cell(&buffer.vertices[0].x, t13);
    let t14 = bh::ld(sq.corners[1][1]);
    let t15 = t14 * t5;
    let t16 = t5 * bh::ld(sq.corners[1][0]);
    let t17 = -t15;
    let t18 = t17 + front.window.view_centre[1].ld();
    st_cell(&buffer.vertices[1].y, t18);
    let t19 = t16 + front.window.view_centre[0].ld();
    st_cell(&buffer.vertices[1].x, t19);
    let t20 = bh::ld(sq.corners[2][1]);
    let t21 = t20 * t3;
    let t22 = t3 * bh::ld(sq.corners[2][0]);
    let t23 = -t21;
    let t24 = t23 + front.window.view_centre[1].ld();
    st_cell(&buffer.vertices[2].y, t24);
    let t25 = t22 + front.window.view_centre[0].ld();
    st_cell(&buffer.vertices[2].x, t25);
    let t26 = bh::ld(sq.corners[3][1]);
    let t27 = t26 * t1;
    let t28 = t1 * bh::ld(sq.corners[3][0]);
    let t29 = -t27;
    let t30 = t29 + front.window.view_centre[1].ld();
    st_cell(&buffer.vertices[3].y, t30);
    let t31 = t28 + front.window.view_centre[0].ld();
    st_cell(&buffer.vertices[3].x, t31);
    let t32 = bh::ld(sq.corners[0][2]);
    let t33 = t32 * bh::ld(sq.inverse);
    let t34 = bh::ld(sq.corners[1][2]);
    let t35 = t34 * bh::ld(sq.inverse);
    let t36 = bh::ld(sq.corners[2][2]);
    let t37 = t36 * bh::ld(sq.inverse);
    let t38 = bh::ld(sq.corners[3][2]);
    let t39 = t38 * bh::ld(sq.inverse);
    st_cell(&buffer.vertices[3].z, t39);
    st_cell(&buffer.vertices[2].z, t37);
    st_cell(&buffer.vertices[1].z, t35);
    st_cell(&buffer.vertices[0].z, t33);
}

#[cfg(test)]
mod tests;
