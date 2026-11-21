//! `effects::draw_plane`: a quad laid on the windscreen drops' frame (a reading: the ground glare).
//! The rate, the place and two axes come from the settings (`windscreen_flakes_rate_factor` and the
//! `light_point_a` / `light_point_b` points); the axes' lengths take their shares and the axes are
//! made one long and square (`scene::normalise_vector3`, `scene::cross_product3`) and scaled
//! (`effects::scale_axes`). Nothing more happens unless the place's way faces the third axis away,
//! or a frame switch is set. Then, with the depth test off (`scene::depth_test_off`), render states
//! 0x11 = 2, 0x12 = 2 and the effects' texture, the four corners are laid out; their texture places
//! come from the quad's corner in the mark corners table, then from the plane's `places`; the
//! colour is the view's light. Unless a corner's depth is at or below the nearest depth (signed
//! compares of the bits) the corners are projected (depth plus the kind float and the plane's
//! `lifts`, times the depth scale; rhw 1 over the depth times `rhw_k`), drawn with
//! `scene::draw_fan`, and the depth test is switched on again.
//!
//! The x87 runs are plain Rust on the x87 model (`math::x87_float`). Refuses null settings and a
//! corner past the mark corners table, before anything.

use crate::effects::kind_store::QuadWork;
use crate::effects::plain::ldi_cell;
use crate::effects::windscreen_drop_vars_store::WindscreenDrops;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::effects_settings_store::EffectsSettings;
use crate::scene::cross_product3::cross_plain;
use crate::scene::vector3::normalise_plain;

/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0xC, 0xE), (8, 0xA), (0, 2), (4, 6)];

/// The plane's constants. The offsets are from its start.
struct PlaneTuning {
    /// What the settings' rate is multiplied by into the first axis' scale
    pub rate_k: f32,
    /// What the two axes' lengths are divided by
    pub shares: [f32; 2],
    /// The four corners' texture places (u, v)
    pub places: [[f32; 2]; 4],
    /// What each corner's depth is raised by
    pub lifts: [f32; 4],
    /// What one over each corner's depth is multiplied by
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
    plane_axes_x87(
        m,
        &mut fl.timing.borrow_mut(),
        &mut fl.tables.borrow_mut(),
        &PLANE_TUNING,
    );
}

/// The second axis, its length over its divisor into the second of the flakes' `lengths`.
fn plane_up(fl: &WindscreenFlakes, m: &EffectsSettings) {
    plane_up_x87(m, &mut fl.tables.borrow_mut(), &PLANE_TUNING);
}

/// The place as floats into the drops' `place`.
fn plane_place(drops: &WindscreenDrops, m: &EffectsSettings) {
    plane_place_x87(drops, m);
}

/// The place's way dotted with the third axis into the drops' `facing`.
fn plane_facing(drops: &WindscreenDrops, fl: &WindscreenFlakes) {
    plane_facing_x87(drops, &mut fl.tables.borrow_mut());
}

/// The plane's centre and corners from the axes.
fn plane_corners(fl: &WindscreenFlakes, effects: &crate::effects::owner::Effects) {
    plane_corners_x87(
        &mut fl.tables.borrow_mut(),
        &mut effects.kinds.plane_quad.borrow_mut(),
    );
}

/// The four corners projected.
fn plane_projected(
    front: &crate::scene::polygon_work_store::PolygonWork,
    effects: &crate::effects::owner::Effects,
) {
    plane_projected_x87(front, &mut effects.kinds.plane_quad.borrow_mut());
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
    let corner: u32 = effects.kinds.plane_quad.borrow().corner;
    plane_axes(fl, m);
    {
        let mut tables = fl.tables.borrow_mut();
        normalise_plain(&mut tables.axes[0]);
    }
    plane_up(fl, m);
    {
        let mut tables = fl.tables.borrow_mut();
        let axes = &mut tables.axes;
        normalise_plain(&mut axes[1]);
        axes[2] = cross_plain(&axes[1], &axes[0]);
        normalise_plain(&mut axes[2]);
        axes[1] = cross_plain(&axes[0], &axes[2]);
    }
    crate::effects::scale_axes::scale_axes(fl);
    plane_place(drops, m);
    normalise_plain(&mut drops.hex.borrow_mut().place);
    plane_facing(drops, fl);
    if !drops.state.borrow().facing.is_sign_negative() {
        return Ok(());
    }
    if race.frame.frame_work.borrow().lead_kept[7] == 0 {
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
        scene.textures.kinds.borrow().kind_first_slots[202],
    );
    plane_corners(fl, effects);
    let entry = &effects.blocks.corners.corners[corner as usize];
    let word = |off: u32| u32::from(entry.words[off as usize / 2].get());
    let uv_k = effects.frame.draw_work.borrow().uv_k;
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
    let light: u32 = ctx.view.light.borrow().view_light;
    for item in buffer.vertices.iter().take(4) {
        item.colour.set(light);
    }
    for item in effects.kinds.plane_quad.borrow().corners.iter().take(4) {
        if x87::bits_at_or_below(item[2], race.course.numbers.k4096_a.get()) {
            return Ok(());
        }
    }
    plane_projected(ctx.scene.scratch.front_end_polygon(), effects);
    let [kind, z_k] = [
        effects.setup.kind_floats.borrow().kind_float,
        effects.frame.draw_work.borrow().z_k,
    ];
    for n in 0..4 {
        let c = effects.kinds.plane_quad.borrow().corners[n];
        let vertex = &buffer.vertices[n];
        vertex.x.set(c[0]);
        vertex.y.set(c[1]);
        let depth = c[2];
        vertex.z.set(bh::stf(
            (bh::ld(depth) + bh::ld(kind) + bh::ld(PLANE_TUNING.lifts[n])) * bh::ld(z_k),
        ));
    }
    for n in 0..4 {
        let depth = effects.kinds.plane_quad.borrow().corners[n][2];
        buffer.vertices[n]
            .rhw
            .set(x87::reciprocal_times(depth, PLANE_TUNING.rhw_k[n]));
    }
    crate::effects::clip_quad::draw_fan(buffer, scene);
    crate::scene::depth_test_on::depth_test_on(ctx.part);
    Ok(())
}

use crate::effects::windscreen_flakes_store::{WindscreenFlakeTables, WindscreenFlakeTiming};
use crate::math::x87_float as bh;

/// `plane_axes`'s arithmetic as plain Rust on the x87 model, in the same order.
fn plane_axes_x87(
    m: &EffectsSettings,
    sti: &mut WindscreenFlakeTiming,
    sq: &mut WindscreenFlakeTables,
    pt: &PlaneTuning,
) {
    let t0 = ldi_cell(&m.windscreen_flakes_rate_factor);
    let t1 = t0 * bh::ld(pt.rate_k);
    sti.rate = f32::from_bits(bh::st(t1));
    let t2 = ldi_cell(&m.light_point_b_negated[0]);
    let t3 = ldi_cell(&m.light_point_b_negated[1]);
    let t4 = ldi_cell(&m.light_point_b_negated[2]);
    sq.place[2] = f32::from_bits(bh::st(t4));
    sq.place[1] = f32::from_bits(bh::st(t3));
    sq.place[0] = f32::from_bits(bh::st(t2));
    let t5 = ldi_cell(&m.light_point_b[0]);
    let t6 = t5 - ldi_cell(&m.light_point_b_negated[0]);
    sq.axes[0][0] = f32::from_bits(bh::st(t6));
    let t7 = ldi_cell(&m.light_point_b[1]);
    let t8 = t7 - ldi_cell(&m.light_point_b_negated[1]);
    sq.axes[0][1] = f32::from_bits(bh::st(t8));
    let t9 = ldi_cell(&m.light_point_b[2]);
    let t10 = t9 - ldi_cell(&m.light_point_b_negated[2]);
    sq.axes[0][2] = f32::from_bits(bh::st(t10));
    let t11 = bh::ld(sq.axes[0][0]);
    let t12 = t11 * t11;
    let t13 = bh::ld(sq.axes[0][1]);
    let t14 = t13 * t13;
    let t15 = bh::ld(sq.axes[0][2]);
    let t16 = t15 * t15;
    let t17 = t14 + t16;
    let t18 = t12 + t17;
    let t19 = t18.sqrt();
    let t20 = t19 / bh::ld(pt.shares[0]);
    sq.lengths[0] = f32::from_bits(bh::st(t20));
}

/// `plane_up`'s arithmetic as plain Rust on the x87 model, in the same order.
fn plane_up_x87(m: &EffectsSettings, sq: &mut WindscreenFlakeTables, pt: &PlaneTuning) {
    let t0 = ldi_cell(&m.light_point_a_negated[0]);
    let t1 = t0 - ldi_cell(&m.light_point_b_negated[0]);
    sq.axes[1][0] = f32::from_bits(bh::st(t1));
    let t2 = ldi_cell(&m.light_point_a_negated[1]);
    let t3 = t2 - ldi_cell(&m.light_point_b_negated[1]);
    sq.axes[1][1] = f32::from_bits(bh::st(t3));
    let t4 = ldi_cell(&m.light_point_a_negated[2]);
    let t5 = t4 - ldi_cell(&m.light_point_b_negated[2]);
    sq.axes[1][2] = f32::from_bits(bh::st(t5));
    let t6 = bh::ld(sq.axes[1][0]);
    let t7 = t6 * t6;
    let t8 = bh::ld(sq.axes[1][1]);
    let t9 = t8 * t8;
    let t10 = bh::ld(sq.axes[1][2]);
    let t11 = t10 * t10;
    let t12 = t9 + t11;
    let t13 = t7 + t12;
    let t14 = t13.sqrt();
    let t15 = t14 / bh::ld(pt.shares[1]);
    sq.lengths[1] = f32::from_bits(bh::st(t15));
}

/// `plane_place`'s arithmetic as plain Rust on the x87 model, in the same order.
fn plane_place_x87(drops: &WindscreenDrops, m: &EffectsSettings) {
    let t0 = ldi_cell(&m.light_point_b_negated[0]);
    drops.hex.borrow_mut().place[0] = bh::stf(t0);
    let t1 = ldi_cell(&m.light_point_b_negated[1]);
    drops.hex.borrow_mut().place[1] = bh::stf(t1);
    let t2 = ldi_cell(&m.light_point_b_negated[2]);
    drops.hex.borrow_mut().place[2] = bh::stf(t2);
}

/// `plane_facing`'s arithmetic as plain Rust on the x87 model, in the same order.
fn plane_facing_x87(drops: &WindscreenDrops, sq: &mut WindscreenFlakeTables) {
    let t0 = bh::ld(drops.hex.borrow().place[0]);
    let t1 = t0 * bh::ld(sq.axes[2][0]);
    let t2 = bh::ld(drops.hex.borrow().place[1]);
    let t3 = t2 * bh::ld(sq.axes[2][1]);
    let t4 = bh::ld(drops.hex.borrow().place[2]);
    let t5 = t4 * bh::ld(sq.axes[2][2]);
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    drops.state.borrow_mut().facing = f32::from_bits(bh::st(t7));
}

/// `plane_corners`'s arithmetic as plain Rust on the x87 model, in the same order.
fn plane_corners_x87(sq: &mut WindscreenFlakeTables, q: &mut QuadWork) {
    let t0 = bh::X(0.0);
    let t1 = bh::X(0.0);
    let t2 = t1 * bh::ld(sq.lengths[1]);
    let t3 = t0 * bh::ld(sq.lengths[0]);
    let t4 = t3 * bh::ld(sq.axes[0][0]);
    let t5 = t2 * bh::ld(sq.axes[1][0]);
    let t6 = t4 + t5;
    let t7 = t6 + bh::ld(sq.place[0]);
    q.corners[3][0] = bh::stf(t7);
    let t8 = t3 * bh::ld(sq.axes[0][1]);
    let t9 = t2 * bh::ld(sq.axes[1][1]);
    let t10 = t8 + t9;
    let t11 = t10 + bh::ld(sq.place[1]);
    q.corners[3][1] = bh::stf(t11);
    let t12 = t2 * bh::ld(sq.axes[1][2]);
    let t13 = t3 * bh::ld(sq.axes[0][2]);
    let t14 = t12 + t13;
    let t15 = t14 + bh::ld(sq.place[2]);
    q.corners[3][2] = bh::stf(t15);
    let t16 = bh::ld(q.corners[3][0]);
    let t17 = t16 + bh::ld(sq.scaled_axes[1][0]);
    q.corners[0][0] = bh::stf(t17);
    let t18 = t17 + bh::ld(sq.scaled_axes[0][0]);
    q.corners[1][0] = bh::stf(t18);
    let t19 = t18 - bh::ld(sq.scaled_axes[1][0]);
    q.corners[2][0] = bh::stf(t19);
    let t20 = bh::ld(q.corners[3][1]);
    let t21 = t20 + bh::ld(sq.scaled_axes[1][1]);
    q.corners[0][1] = bh::stf(t21);
    let t22 = t21 + bh::ld(sq.scaled_axes[0][1]);
    q.corners[1][1] = bh::stf(t22);
    let t23 = t22 - bh::ld(sq.scaled_axes[1][1]);
    q.corners[2][1] = bh::stf(t23);
    let t24 = bh::ld(q.corners[3][2]);
    let t25 = t24 + bh::ld(sq.scaled_axes[1][2]);
    q.corners[0][2] = bh::stf(t25);
    let t26 = t25 + bh::ld(sq.scaled_axes[0][2]);
    q.corners[1][2] = bh::stf(t26);
    let t27 = t26 - bh::ld(sq.scaled_axes[1][2]);
    q.corners[2][2] = bh::stf(t27);
}

/// `plane_projected`'s arithmetic as plain Rust on the x87 model, in the same order.
fn plane_projected_x87(front: &crate::scene::polygon_work_store::PolygonWork, q: &mut QuadWork) {
    let t0 = crate::math::x87_float::ld(front.window.borrow().view_scale);
    let t1 = t0 / bh::ld(q.corners[0][2]);
    let t2 = t1 * bh::ld(q.corners[0][0]);
    let t3 = t2 + crate::math::x87_float::ld(front.window.borrow().view_centre[0]);
    q.corners[0][0] = bh::stf(t3);
    let t4 = t1 * bh::ld(q.corners[0][1]);
    let t5 = -t4;
    let t6 = t5 + crate::math::x87_float::ld(front.window.borrow().view_centre[1]);
    q.corners[0][1] = bh::stf(t6);
    let t7 = crate::math::x87_float::ld(front.window.borrow().view_scale);
    let t8 = t7 / bh::ld(q.corners[1][2]);
    let t9 = t8 * bh::ld(q.corners[1][0]);
    let t10 = t9 + crate::math::x87_float::ld(front.window.borrow().view_centre[0]);
    q.corners[1][0] = bh::stf(t10);
    let t11 = t8 * bh::ld(q.corners[1][1]);
    let t12 = -t11;
    let t13 = t12 + crate::math::x87_float::ld(front.window.borrow().view_centre[1]);
    q.corners[1][1] = bh::stf(t13);
    let t14 = crate::math::x87_float::ld(front.window.borrow().view_scale);
    let t15 = t14 / bh::ld(q.corners[2][2]);
    let t16 = t15 * bh::ld(q.corners[2][0]);
    let t17 = t16 + crate::math::x87_float::ld(front.window.borrow().view_centre[0]);
    q.corners[2][0] = bh::stf(t17);
    let t18 = t15 * bh::ld(q.corners[2][1]);
    let t19 = -t18;
    let t20 = t19 + crate::math::x87_float::ld(front.window.borrow().view_centre[1]);
    q.corners[2][1] = bh::stf(t20);
    let t21 = crate::math::x87_float::ld(front.window.borrow().view_scale);
    let t22 = t21 / bh::ld(q.corners[3][2]);
    let t23 = t22 * bh::ld(q.corners[3][0]);
    let t24 = t23 + crate::math::x87_float::ld(front.window.borrow().view_centre[0]);
    q.corners[3][0] = bh::stf(t24);
    let t25 = t22 * bh::ld(q.corners[3][1]);
    let t26 = -t25;
    let t27 = t26 + crate::math::x87_float::ld(front.window.borrow().view_centre[1]);
    q.corners[3][1] = bh::stf(t27);
}

#[cfg(test)]
mod tests;
