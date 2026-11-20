//! `effects::draw_plane`: a quad laid on the windscreen drops' frame (a reading: the ground glare).
//! The rate, the place and two axes come from the settings (+0x2720 .., +0x272C .., +0x2708 ..,
//! +0x2740); the axes' lengths take their shares and the axes are made one long and square
//! (`scene::normalise_vector3`, `scene::cross_product3`) and scaled (`effects::scale_axes`).
//! Nothing more happens unless the place's way faces the third axis away, or a frame switch is set.
//! Then, with the depth test off (`scene::depth_test_off`), render states 0x11 = 2, 0x12 = 2 and
//! the effects' texture, the four corners are laid out; their texture places come from the quad's
//! corner in the mark corners table, then from the plane's `places`; the colour is the view's
//! light. Unless a corner's depth is at or below the nearest depth (signed compares of the bits)
//! the corners are projected (depth plus the kind float and the plane's `lifts`, times the depth
//! scale; rhw 1 over the depth times `rhw_k`), drawn with `scene::draw_fan`, and the depth test is
//! switched on again.
//!
//! The x87 runs are plain Rust on the x87 model (`math::x87_float`). Refuses null settings and a
//! corner past the mark corners table, before anything.

use crate::effects::kind_store::QuadWork;
use crate::effects::plain::ldi_cell;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::{BadInput, Refused};
use crate::math::cell_x87::CellX87;
use crate::math::x87;
use crate::race::effects_settings_store::EffectsSettings;
use crate::scene::cross_product3::cross;
use crate::scene::vector3::normalise;

/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0xC, 0xE), (8, 0xA), (0, 2), (4, 6)];

/// The plane's constants. The offsets are from its start.
struct PlaneTuning {
    /// +0x0: what the settings' rate is multiplied by into the first axis' scale
    pub rate_k: f32,
    /// +0x4: what the two axes' lengths are divided by
    pub shares: [f32; 2],
    /// +0xC: the four corners' texture places (u, v)
    pub places: [[f32; 2]; 4],
    /// +0x2C: what each corner's depth is raised by
    pub lifts: [f32; 4],
    /// +0x3C: what one over each corner's depth is multiplied by
    pub rhw_k: [f32; 4],
}

/// The plane's constants.
static PLANE_TUNING: PlaneTuning = PlaneTuning {
    rate_k: 1.0_f32,
    shares: [2048.0_f32, 1024.0_f32],
    places: [
        [0.0_f32, 0.5019531_f32],
        [0.9999_f32, 0.5019531_f32],
        [0.9999_f32; 2],
        [0.0_f32, 0.9999_f32],
    ],
    lifts: [500.0_f32; 4],
    rhw_k: [256.0_f32; 4],
};

/// The rate, the place and the first axis from the settings, and the first axis's length over
/// its divisor into the first of the flakes' `lengths`.
fn plane_axes(fl: &WindscreenFlakes, m: &EffectsSettings) {
    plane_axes_x87(m, &fl.timing, &fl.tables, &PLANE_TUNING);
}

/// The second axis, its length over its divisor into the second of the flakes' `lengths`.
fn plane_up(fl: &WindscreenFlakes, m: &EffectsSettings) {
    plane_up_x87(m, &fl.tables, &PLANE_TUNING);
}

/// The place as floats into the drops' `place`.
fn plane_place(drops: &WindscreenDrops, m: &EffectsSettings) {
    plane_place_x87(drops, m);
}

/// The place's way dotted with the third axis into the drops' `facing`.
fn plane_facing(drops: &WindscreenDrops, fl: &WindscreenFlakes) {
    plane_facing_x87(drops, &fl.tables);
}

/// The plane's centre and corners from the axes.
fn plane_corners(fl: &WindscreenFlakes, effects: &crate::effects::owner::Effects) {
    plane_corners_x87(&fl.tables, &effects.kinds.plane_quad);
}

/// The four corners projected.
fn plane_projected(
    front: &crate::scene::polygon_work::PolygonWork,
    effects: &crate::effects::owner::Effects,
) {
    plane_projected_x87(front, &effects.kinds.plane_quad);
}

pub fn draw_plane(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    drops: &WindscreenDrops,
    fl: &WindscreenFlakes,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let buffer = &ctx.part.vertices;
    let (race, scene) = (ctx.race, ctx.scene);
    use crate::scene::set_render_state::set_render_state;
    let m = effects.settings()?;
    let corner: u32 = effects.kinds.plane_quad.corner.get();
    plane_axes(fl, m);
    let [a0, a1, a2] = [0, 1, 2].map(|n| &fl.tables.axes[n]);
    normalise(a0);
    plane_up(fl, m);
    normalise(a1);
    cross(a1, a0, a2);
    normalise(a2);
    cross(a0, a2, a1);
    crate::effects::scale_axes::scale_axes(fl);
    plane_place(drops, m);
    normalise(&drops.hex.place);
    plane_facing(drops, fl);
    if !drops.state.facing.get().is_sign_negative() {
        return Ok(());
    }
    if race.frame.frame_work.get().lead_kept[7].get() == 0 {
        return Ok(());
    }
    if corner >= 0x100 {
        return Err(BadInput("a plane corner past g_marksCorners").into());
    }
    crate::scene::depth_test_off::depth_test_off(ctx.part);
    for state in [0x11, 0x12] {
        set_render_state(ctx.part, 2, state);
    }
    crate::scene::set_texture::set_texture(
        ctx.part,
        scene,
        scene.textures.kinds.get().kind_first_slots[202].get(),
    );
    plane_corners(fl, effects);
    let entry = &effects.blocks.corners.corners[corner as usize];
    let word = |off: u32| u32::from(entry.words[off as usize / 2].get());
    let uv_k = effects.frame.draw_work.uv_k.get();
    for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
        let vertex = &buffer.vertices[n];
        vertex.u.set(x87::int_times(word(u) as i32, uv_k));
        vertex.v.set(x87::int_times(word(v) as i32, uv_k));
    }
    for n in 0..4 {
        let vertex = &buffer.vertices[n];
        vertex.u.set(x87::loaded(PLANE_TUNING.places[n][0]));
        vertex.v.set(x87::loaded(PLANE_TUNING.places[n][1]));
    }
    let light: u32 = ctx.view.light.view_light.get();
    for item in buffer.vertices.iter().take(4) {
        item.colour.set(light);
    }
    for item in effects.kinds.plane_quad.corners.iter().take(4) {
        if x87::bits_at_or_below(item[2].get(), race.course.numbers.get().k4096_a.get()) {
            return Ok(());
        }
    }
    plane_projected(ctx.scene.scratch.front_end_polygon(), effects);
    let [kind, z_k] = [
        effects.setup.kind_floats.kind_float.get(),
        effects.frame.draw_work.z_k.get(),
    ];
    for n in 0..4 {
        let c = &effects.kinds.plane_quad.corners[n];
        let vertex = &buffer.vertices[n];
        vertex.x.set(c[0].get());
        vertex.y.set(c[1].get());
        let depth = c[2].get();
        vertex.z.set(bh::stf(
            (bh::ld(depth) + bh::ld(kind) + bh::ld(PLANE_TUNING.lifts[n])) * bh::ld(z_k),
        ));
    }
    for n in 0..4 {
        let depth = effects.kinds.plane_quad.corners[n][2].get();
        buffer.vertices[n]
            .rhw
            .set(x87::reciprocal_times(depth, PLANE_TUNING.rhw_k[n]));
    }
    crate::effects::clip_quad::draw_fan(buffer, scene);
    crate::scene::depth_test_on::depth_test_on(ctx.part);
    Ok(())
}

use crate::effects::plain::{ld_cell, st_cell};
use crate::effects::windscreen_flakes_store::{WindscreenFlakeTables, WindscreenFlakeTiming};
use crate::math::x87_float as bh;

/// `plane_axes`'s arithmetic as plain Rust on the x87 model, in the same order.
fn plane_axes_x87(
    m: &EffectsSettings,
    sti: &WindscreenFlakeTiming,
    sq: &WindscreenFlakeTables,
    pt: &PlaneTuning,
) {
    let t0 = ldi_cell(&m.windscreen_flakes_rate_factor);
    let t1 = t0 * bh::ld(pt.rate_k);
    sti.rate.st(t1);
    let t2 = ldi_cell(&m.light_point_b_negated[0]);
    let t3 = ldi_cell(&m.light_point_b_negated[1]);
    let t4 = ldi_cell(&m.light_point_b_negated[2]);
    sq.place[2].st(t4);
    sq.place[1].st(t3);
    sq.place[0].st(t2);
    let t5 = ldi_cell(&m.light_point_b[0]);
    let t6 = t5 - ldi_cell(&m.light_point_b_negated[0]);
    sq.axes[0][0].st(t6);
    let t7 = ldi_cell(&m.light_point_b[1]);
    let t8 = t7 - ldi_cell(&m.light_point_b_negated[1]);
    sq.axes[0][1].st(t8);
    let t9 = ldi_cell(&m.light_point_b[2]);
    let t10 = t9 - ldi_cell(&m.light_point_b_negated[2]);
    sq.axes[0][2].st(t10);
    let t11 = sq.axes[0][0].ld();
    let t12 = t11 * t11;
    let t13 = sq.axes[0][1].ld();
    let t14 = t13 * t13;
    let t15 = sq.axes[0][2].ld();
    let t16 = t15 * t15;
    let t17 = t14 + t16;
    let t18 = t12 + t17;
    let t19 = t18.sqrt();
    let t20 = t19 / bh::ld(pt.shares[0]);
    sq.lengths[0].st(t20);
}

/// `plane_up`'s arithmetic as plain Rust on the x87 model, in the same order.
fn plane_up_x87(m: &EffectsSettings, sq: &WindscreenFlakeTables, pt: &PlaneTuning) {
    let t0 = ldi_cell(&m.light_point_a_negated[0]);
    let t1 = t0 - ldi_cell(&m.light_point_b_negated[0]);
    sq.axes[1][0].st(t1);
    let t2 = ldi_cell(&m.light_point_a_negated[1]);
    let t3 = t2 - ldi_cell(&m.light_point_b_negated[1]);
    sq.axes[1][1].st(t3);
    let t4 = ldi_cell(&m.light_point_a_negated[2]);
    let t5 = t4 - ldi_cell(&m.light_point_b_negated[2]);
    sq.axes[1][2].st(t5);
    let t6 = sq.axes[1][0].ld();
    let t7 = t6 * t6;
    let t8 = sq.axes[1][1].ld();
    let t9 = t8 * t8;
    let t10 = sq.axes[1][2].ld();
    let t11 = t10 * t10;
    let t12 = t9 + t11;
    let t13 = t7 + t12;
    let t14 = t13.sqrt();
    let t15 = t14 / bh::ld(pt.shares[1]);
    sq.lengths[1].st(t15);
}

/// `plane_place`'s arithmetic as plain Rust on the x87 model, in the same order.
fn plane_place_x87(drops: &WindscreenDrops, m: &EffectsSettings) {
    let t0 = ldi_cell(&m.light_point_b_negated[0]);
    st_cell(&drops.hex.place[0], t0);
    let t1 = ldi_cell(&m.light_point_b_negated[1]);
    st_cell(&drops.hex.place[1], t1);
    let t2 = ldi_cell(&m.light_point_b_negated[2]);
    st_cell(&drops.hex.place[2], t2);
}

/// `plane_facing`'s arithmetic as plain Rust on the x87 model, in the same order.
fn plane_facing_x87(drops: &WindscreenDrops, sq: &WindscreenFlakeTables) {
    let t0 = ld_cell(&drops.hex.place[0]);
    let t1 = t0 * sq.axes[2][0].ld();
    let t2 = ld_cell(&drops.hex.place[1]);
    let t3 = t2 * sq.axes[2][1].ld();
    let t4 = ld_cell(&drops.hex.place[2]);
    let t5 = t4 * sq.axes[2][2].ld();
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    drops.state.facing.st(t7);
}

/// `plane_corners`'s arithmetic as plain Rust on the x87 model, in the same order.
fn plane_corners_x87(sq: &WindscreenFlakeTables, q: &QuadWork) {
    let t0 = bh::X(0.0);
    let t1 = bh::X(0.0);
    let t2 = t1 * sq.lengths[1].ld();
    let t3 = t0 * sq.lengths[0].ld();
    let t4 = t3 * sq.axes[0][0].ld();
    let t5 = t2 * sq.axes[1][0].ld();
    let t6 = t4 + t5;
    let t7 = t6 + sq.place[0].ld();
    q.corners[3][0].st(t7);
    let t8 = t3 * sq.axes[0][1].ld();
    let t9 = t2 * sq.axes[1][1].ld();
    let t10 = t8 + t9;
    let t11 = t10 + sq.place[1].ld();
    q.corners[3][1].st(t11);
    let t12 = t2 * sq.axes[1][2].ld();
    let t13 = t3 * sq.axes[0][2].ld();
    let t14 = t12 + t13;
    let t15 = t14 + sq.place[2].ld();
    q.corners[3][2].st(t15);
    let t16 = q.corners[3][0].ld();
    let t17 = t16 + sq.scaled_axes[1][0].ld();
    q.corners[0][0].st(t17);
    let t18 = t17 + sq.scaled_axes[0][0].ld();
    q.corners[1][0].st(t18);
    let t19 = t18 - sq.scaled_axes[1][0].ld();
    q.corners[2][0].st(t19);
    let t20 = q.corners[3][1].ld();
    let t21 = t20 + sq.scaled_axes[1][1].ld();
    q.corners[0][1].st(t21);
    let t22 = t21 + sq.scaled_axes[0][1].ld();
    q.corners[1][1].st(t22);
    let t23 = t22 - sq.scaled_axes[1][1].ld();
    q.corners[2][1].st(t23);
    let t24 = q.corners[3][2].ld();
    let t25 = t24 + sq.scaled_axes[1][2].ld();
    q.corners[0][2].st(t25);
    let t26 = t25 + sq.scaled_axes[0][2].ld();
    q.corners[1][2].st(t26);
    let t27 = t26 - sq.scaled_axes[1][2].ld();
    q.corners[2][2].st(t27);
}

/// `plane_projected`'s arithmetic as plain Rust on the x87 model, in the same order.
fn plane_projected_x87(front: &crate::scene::polygon_work::PolygonWork, q: &QuadWork) {
    let t0 = crate::math::x87_float::ld(front.window.view_scale.get());
    let t1 = t0 / q.corners[0][2].ld();
    let t2 = t1 * q.corners[0][0].ld();
    let t3 = t2 + crate::math::x87_float::ld(front.window.view_centre[0].get());
    q.corners[0][0].st(t3);
    let t4 = t1 * q.corners[0][1].ld();
    let t5 = -t4;
    let t6 = t5 + crate::math::x87_float::ld(front.window.view_centre[1].get());
    q.corners[0][1].st(t6);
    let t7 = crate::math::x87_float::ld(front.window.view_scale.get());
    let t8 = t7 / q.corners[1][2].ld();
    let t9 = t8 * q.corners[1][0].ld();
    let t10 = t9 + crate::math::x87_float::ld(front.window.view_centre[0].get());
    q.corners[1][0].st(t10);
    let t11 = t8 * q.corners[1][1].ld();
    let t12 = -t11;
    let t13 = t12 + crate::math::x87_float::ld(front.window.view_centre[1].get());
    q.corners[1][1].st(t13);
    let t14 = crate::math::x87_float::ld(front.window.view_scale.get());
    let t15 = t14 / q.corners[2][2].ld();
    let t16 = t15 * q.corners[2][0].ld();
    let t17 = t16 + crate::math::x87_float::ld(front.window.view_centre[0].get());
    q.corners[2][0].st(t17);
    let t18 = t15 * q.corners[2][1].ld();
    let t19 = -t18;
    let t20 = t19 + crate::math::x87_float::ld(front.window.view_centre[1].get());
    q.corners[2][1].st(t20);
    let t21 = crate::math::x87_float::ld(front.window.view_scale.get());
    let t22 = t21 / q.corners[3][2].ld();
    let t23 = t22 * q.corners[3][0].ld();
    let t24 = t23 + crate::math::x87_float::ld(front.window.view_centre[0].get());
    q.corners[3][0].st(t24);
    let t25 = t22 * q.corners[3][1].ld();
    let t26 = -t25;
    let t27 = t26 + crate::math::x87_float::ld(front.window.view_centre[1].get());
    q.corners[3][1].st(t27);
}

#[cfg(test)]
mod tests;
