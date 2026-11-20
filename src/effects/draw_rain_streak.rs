//! `effects::draw_rain_streak`: a streak (a point and where it was) drawn as a quad facing the
//! viewer. The point (its three values, scaled by `point_scale` into `second`) turned by
//! `matrix_copy` plus the whole numbers in `point` into `turned` (its depth also `depth`); the
//! point less `motion` turned the same into `second`. Unless the first depth is at or below 0x20000
//! or the second at or below 0x48000000 (signed compares of the bits), or `drawn` is 0: both
//! projected (the view scale over the depth; scales `view_scale`, the view's centre) into `ends`,
//! the half width over the screen length across the line, the four vertices' x, y and depth (times
//! `depth_scale`), `effects::clip_quad`, and unless all is outside `scene::draw_fan` and the
//! polygon's vertex count 4.
//!
//! The arithmetic is plain Rust on the x87 model (`math::x87_float`). Refuses null marks settings
//! (`clip_quad`'s refusal), before anything.

use crate::effects::clip_quad::{clip_quad, draw_fan};
use crate::effects::rain_store::{RainRules, RainState, Weather};
use crate::errors::{BadInput, Refused};
use crate::math::cell_x87::CellX87;
use crate::math::x87;

/// The point scaled and turned, and the last place.
fn turn(weather: &Weather, p: &[[u8; 10]; 3]) {
    turn_x87(p, &mut weather.rain.borrow_mut());
}

/// Both ends projected and the quad's x, y and depth.
fn project(
    front: &crate::scene::polygon_work_store::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    weather: &Weather,
) {
    project_x87(
        front,
        buffer,
        &mut weather.rain.borrow_mut(),
        &weather.rain_rules,
    );
}

pub(super) fn draw_rain_streak(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    weather: &Weather,
    place: [[u8; 10]; 3],
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        if effects.setup.settings.is_set.get() == 0 {
            return Err(BadInput("no marks settings").into());
        }
        turn(weather, &place);
        // The depths compared by their bits: 0x20000 is a tiny subnormal (the whole
        // number as a float), 0x48000000 is 131072.
        let (depth, drawn, second_depth) = {
            let rain = weather.rain.borrow();
            (rain.depth, rain.drawn, rain.second[2])
        };
        if x87::bits_at_or_below(depth, f32::from_bits(0x2_0000))
            || drawn == 0
            || x87::bits_at_or_below(second_depth, 131_072.0)
        {
            return Ok(());
        }
        project(scene.scratch.front_end_polygon(), buffer, weather);
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

use crate::effects::plain::st_cell;
use crate::math::x87_float as bh;

/// `turn`'s arithmetic as plain Rust on the x87 model, in the same order.
fn turn_x87(p: &[[u8; 10]; 3], r: &mut RainState) {
    let t0 = bh::from_f80(p[2]);
    let t1 = bh::from_f80(p[1]);
    let t2 = bh::from_f80(p[0]);
    let t3 = t0 * bh::ld(r.point_scale[0]);
    r.second[0] = f32::from_bits(bh::st(t3));
    let t4 = t1 * bh::ld(r.point_scale[1]);
    r.second[1] = f32::from_bits(bh::st(t4));
    let t5 = t2 * bh::ld(r.point_scale[2]);
    r.second[2] = f32::from_bits(bh::st(t5));
    let t6 = t3 * bh::ld(r.matrix_copy[0]);
    let t7 = t4 * bh::ld(r.matrix_copy[3]);
    let t8 = t5 * bh::ld(r.matrix_copy[6]);
    let t9 = t7 + t8;
    let t10 = t6 + t9;
    let t11 = t10 + bh::int(r.point[0] as i32);
    r.turned[0] = f32::from_bits(bh::st(t11));
    let t12 = t3 * bh::ld(r.matrix_copy[1]);
    let t13 = t4 * bh::ld(r.matrix_copy[4]);
    let t14 = t5 * bh::ld(r.matrix_copy[7]);
    let t15 = t13 + t14;
    let t16 = t12 + t15;
    let t17 = t16 + bh::int(r.point[1] as i32);
    r.turned[1] = f32::from_bits(bh::st(t17));
    let t18 = t3 * bh::ld(r.matrix_copy[2]);
    let t19 = t4 * bh::ld(r.matrix_copy[5]);
    let t20 = t5 * bh::ld(r.matrix_copy[8]);
    let t21 = t19 + t20;
    let t22 = t18 + t21;
    let t23 = t22 + bh::int(r.point[2] as i32);
    r.depth = f32::from_bits(bh::st(t23));
    r.turned[2] = f32::from_bits(bh::st(t23));
    let t24 = bh::ld(r.second[1]);
    let t25 = t24 - bh::ld(r.motion[1]);
    let t26 = bh::ld(r.second[0]);
    let t27 = t26 - bh::ld(r.motion[0]);
    let t28 = bh::ld(r.second[2]);
    let t29 = t28 - bh::ld(r.motion[2]);
    let t30 = t27 * bh::ld(r.matrix_copy[0]);
    let t31 = t25 * bh::ld(r.matrix_copy[3]);
    let t32 = t29 * bh::ld(r.matrix_copy[6]);
    let t33 = t31 + t32;
    let t34 = t30 + t33;
    let t35 = t34 + bh::int(r.point[0] as i32);
    r.second[0] = f32::from_bits(bh::st(t35));
    let t36 = t27 * bh::ld(r.matrix_copy[1]);
    let t37 = t25 * bh::ld(r.matrix_copy[4]);
    let t38 = t29 * bh::ld(r.matrix_copy[7]);
    let t39 = t37 + t38;
    let t40 = t36 + t39;
    let t41 = t40 + bh::int(r.point[1] as i32);
    r.second[1] = f32::from_bits(bh::st(t41));
    let t42 = t27 * bh::ld(r.matrix_copy[2]);
    let t43 = t25 * bh::ld(r.matrix_copy[5]);
    let t44 = t29 * bh::ld(r.matrix_copy[8]);
    let t45 = t43 + t44;
    let t46 = t42 + t45;
    let t47 = t46 + bh::int(r.point[2] as i32);
    r.second[2] = f32::from_bits(bh::st(t47));
}

/// `project`'s arithmetic as plain Rust on the x87 model, in the same order.
fn project_x87(
    front: &crate::scene::polygon_work_store::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    r: &mut RainState,
    k: &RainRules,
) {
    let t0 = bh::ld(r.second[2]);
    let t1 = bh::ld(front.window.view_scale.get()) / t0;
    let t2 = bh::ld(r.turned[2]);
    let t3 = bh::ld(front.window.view_scale.get()) / t2;
    let t4 = bh::ld(r.second[0]);
    let t5 = bh::ld(r.second[1]);
    let t6 = bh::ld(r.turned[0]);
    let t7 = bh::ld(r.turned[1]);
    let t8 = t4 * t1;
    let t9 = t5 * t1;
    let t10 = t6 * t3;
    let t11 = t7 * t3;
    let t12 = t1 * bh::ld(k.view_scale[0]);
    let t13 = t3 * bh::ld(k.view_scale[1]);
    let t14 = t8 + front.window.view_centre[0].ld();
    r.ends[0][0] = f32::from_bits(bh::st(t14));
    let t15 = -t9;
    let t16 = t15 + front.window.view_centre[1].ld();
    r.ends[0][1] = f32::from_bits(bh::st(t16));
    let t17 = t10 + front.window.view_centre[0].ld();
    r.ends[1][0] = f32::from_bits(bh::st(t17));
    let t18 = -t11;
    let t19 = t18 + front.window.view_centre[1].ld();
    r.ends[1][1] = f32::from_bits(bh::st(t19));
    r.ends[0][2] = f32::from_bits(bh::st(t12));
    r.ends[1][2] = f32::from_bits(bh::st(t13));
    let t20 = bh::ONE;
    let t21 = t20 / bh::ld(front.window.farthest.get());
    let t22 = bh::ld(r.ends[1][0]);
    let t23 = t22 - bh::ld(r.ends[0][0]);
    r.edge_a[1] = f32::from_bits(bh::st(t23));
    let t24 = t23 * t23;
    let t25 = bh::ld(r.ends[1][1]);
    let t26 = t25 - bh::ld(r.ends[0][1]);
    let t27 = -t26;
    r.edge_a[0] = f32::from_bits(bh::st(t27));
    let t28 = t27 * t27;
    let t29 = t24 + t28;
    let t30 = t29.sqrt();
    let t31 = bh::ld(k.half_width) / t30;
    let t32 = t31 * bh::ld(r.edge_a[0]);
    let t33 = t32 * bh::ld(r.ends[0][2]);
    r.edge_a[0] = f32::from_bits(bh::st(t33));
    let t34 = t32 * bh::ld(r.ends[1][2]);
    r.edge_b[0] = f32::from_bits(bh::st(t34));
    let t35 = t31 * bh::ld(r.edge_a[1]);
    let t36 = t35 * bh::ld(r.ends[0][2]);
    r.edge_a[1] = f32::from_bits(bh::st(t36));
    let t37 = t35 * bh::ld(r.ends[1][2]);
    r.edge_b[1] = f32::from_bits(bh::st(t37));
    r.depth_scale = f32::from_bits(bh::st(t21));
    let t38 = bh::ld(r.second[2]);
    let t39 = t38 * bh::ld(r.depth_scale);
    st_cell(&buffer.vertices[0].z, t39);
    st_cell(&buffer.vertices[1].z, t39);
    st_cell(&buffer.vertices[2].z, t39);
    st_cell(&buffer.vertices[3].z, t39);
    let t40 = bh::ld(r.ends[0][0]);
    st_cell(&buffer.vertices[0].x, t40);
    let t41 = t40 - bh::ld(r.edge_a[0]);
    st_cell(&buffer.vertices[1].x, t41);
    let t42 = bh::ld(r.ends[1][0]);
    st_cell(&buffer.vertices[3].x, t42);
    let t43 = t42 - bh::ld(r.edge_b[0]);
    st_cell(&buffer.vertices[2].x, t43);
    let t44 = bh::ld(r.ends[0][1]);
    st_cell(&buffer.vertices[0].y, t44);
    let t45 = t44 - bh::ld(r.edge_a[1]);
    st_cell(&buffer.vertices[1].y, t45);
    let t46 = bh::ld(r.ends[1][1]);
    st_cell(&buffer.vertices[3].y, t46);
    let t47 = t46 - bh::ld(r.edge_b[1]);
    st_cell(&buffer.vertices[2].y, t47);
}

#[cfg(test)]
mod tests;
