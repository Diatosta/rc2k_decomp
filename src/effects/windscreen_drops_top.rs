//! `effects::windscreen_drops_top`: the windscreen drops' frame from the settings: the vertex ring
//! from its start, the ease and the rate, the place (the settings' `light_point_b_negated`) and two axes made
//! one long and square, then (only when the place's way faces away from the third axis) the 0x30
//! windscreen drop points scaled and turned onto the axes. Unless the rate is 0, the drops' frame
//! runs, the vertices left in the ring are drawn (`d3d::draw_marks_windscreen_drops`, the
//! platform's), and with the settings' `windscreen_wet` and the drift on the drops step. Finally the marks'
//! texture and a polygon vertex count of 4 are set.
//!
//! Screen: the rain drops on the windscreen in the in-car view (confirmed: gone with
//! `d3d::draw_marks_windscreen_drops` cut). Drawn as hexagons.
//!
//! Works on the effects settings.
//!
//! The x87 runs are plain Rust on the x87 model (`math::x87_float`). Refuses null settings, before
//! anything.

use crate::effects::mask_store::Masks;
use crate::effects::plain::ldi_cell;
use crate::effects::setup_store::MarksClock;
use crate::effects::windscreen_drop_state_store::WindscreenDropWork;
use crate::effects::windscreen_drop_vars_store::{WindscreenDropVars, WindscreenDrops};
use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87_float::{self as bh, stf};
use crate::race::effects_settings_store::EffectsSettings;
use crate::race::owner_frame::Frame;
use crate::scene::cross_product3::cross_plain;
use crate::scene::vector3::normalise_plain;

const POINTS: u32 = 0x30;

pub fn windscreen_drops_top(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    frame: &Frame,
    masks: &Masks,
    drops: &WindscreenDrops,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let m = effects.settings()?;
    {
        let mut hex = drops.hex.borrow_mut();
        let mut work = drops.work.borrow_mut();
        hex.buffer = 0;
        // The ease (by the ticks), the rate, the place and the first axis from the settings,
        // and the first axis's length over its divisor into the first share.
        frame_axes_x87(&mut hex, &mut work, drops, m, &effects.setup.clock.borrow());
        normalise_axis(&mut work.axes, 0);
        // The second axis, its length over its divisor into the second share.
        frame_up_x87(&hex, &mut work, m);
        let axes = &mut work.axes;
        normalise_axis(axes, 1);
        put_axis(axes, 2, cross_plain(&axis_of(axes, 1), &axis_of(axes, 0)));
        normalise_axis(axes, 2);
        put_axis(axes, 1, cross_plain(&axis_of(axes, 0), &axis_of(axes, 2)));
        // The place again as floats into the drops' `place`, and its way dotted with the third
        // axis into `facing`.
        frame_place_x87(&mut hex, m);
        normalise_plain(&mut hex.place);
        frame_facing_x87(&hex, &work, drops);
    }
    if !drops.state.borrow().facing.is_sign_negative() {
        return Ok(());
    }
    {
        let mut work = drops.work.borrow_mut();
        let [kx, ky] = work.shares;
        for n in 0..POINTS as usize {
            let (p, o) = (3 * n, 3 * n);
            work.scaled[o] = x87::mul(work.points[p], kx);
            work.scaled[o + 1] = x87::mul(work.points[p + 1], ky);
            work.scaled[o + 2] = 0.0;
        }
        let axes = work.axes;
        for p in work.scaled.as_chunks_mut::<3>().0 {
            point_turned_x87(&axes, p);
        }
    }
    let rate = drops.state.borrow().rate;
    // Any rate but positive zero (the bits tested).
    if rate != 0.0 || rate.is_sign_negative() {
        crate::effects::windscreen_drops_frame::windscreen_drops_frame(
            buffer, frame, masks, drops, scene, gfx, effects,
        )?;
        let at_now: u32 = drops.hex.borrow().buffer;
        if at_now != 0 {
            let quads =
                x87::int_over_int_to_int(at_now as i32, drops.hex.borrow().per_vertex) as u32;
            drops.hex.borrow_mut().buffer = 0;
            gfx.card
                .windscreen_drop_batch
                .index_count
                .set(quads.wrapping_mul(12));
            gfx.card
                .windscreen_drop_batch
                .vertex_count
                .set(quads.wrapping_mul(6));
            crate::effects::windscreen_drops_frame::draw_batch(scene, gfx)?;
        }
        let marks = effects.settings()?;
        if marks.windscreen_wet.get() != 0 && effects.setup.kind_floats.borrow().running != 0 {
            crate::effects::windscreen_drops::windscreen_drops_step(masks, drops)?;
        }
    }
    crate::scene::set_texture::set_texture(
        scene.scratch.game_scratch(),
        scene,
        scene.textures.kinds.borrow().kind_first_slots[202],
    );
    scene
        .scratch
        .front_end_polygon()
        .polygon_vertex_count
        .set(4);
    Ok(())
}

/// Axis `n` (three floats) of the three axes.
fn axis_of(axes: &[f32; 9], n: usize) -> [f32; 3] {
    [axes[3 * n], axes[3 * n + 1], axes[3 * n + 2]]
}

/// `v` into axis `n` of the three axes.
fn put_axis(axes: &mut [f32; 9], n: usize, v: [f32; 3]) {
    axes[3 * n..3 * n + 3].copy_from_slice(&v);
}

/// Axis `n` normalised in place.
fn normalise_axis(axes: &mut [f32; 9], n: usize) {
    let mut v = axis_of(axes, n);
    normalise_plain(&mut v);
    put_axis(axes, n, v);
}

/// `frame_axes`'s arithmetic as plain Rust on the x87 model, in the same order.
fn frame_axes_x87(
    hex: &mut WindscreenDropVars,
    work: &mut WindscreenDropWork,
    drops: &WindscreenDrops,
    m: &EffectsSettings,
    c: &MarksClock,
) {
    let t0 = bh::ld(hex.top_k[0]);
    let t1 = t0 * crate::math::x87_float::int(c.frame_ticks as i32);
    hex.ease = stf(t1);
    let t2 = ldi_cell(&m.windscreen_drops_rate_factor);
    let t3 = t2 * bh::ld(hex.top_k[1]);
    drops.state.borrow_mut().rate = stf(t3);
    let t4 = ldi_cell(&m.light_point_b_negated[0]);
    let t5 = ldi_cell(&m.light_point_b_negated[1]);
    let t6 = ldi_cell(&m.light_point_b_negated[2]);
    work.place[2] = stf(t6);
    work.place[1] = stf(t5);
    work.place[0] = stf(t4);
    let t7 = ldi_cell(&m.light_point_b[0]);
    let t8 = t7 - ldi_cell(&m.light_point_b_negated[0]);
    work.axes[0] = stf(t8);
    let t9 = ldi_cell(&m.light_point_b[1]);
    let t10 = t9 - ldi_cell(&m.light_point_b_negated[1]);
    work.axes[1] = stf(t10);
    let t11 = ldi_cell(&m.light_point_b[2]);
    let t12 = t11 - ldi_cell(&m.light_point_b_negated[2]);
    work.axes[2] = stf(t12);
    let t13 = bh::ld(work.axes[0]);
    let t14 = t13 * t13;
    let t15 = bh::ld(work.axes[1]);
    let t16 = t15 * t15;
    let t17 = bh::ld(work.axes[2]);
    let t18 = t17 * t17;
    let t19 = t16 + t18;
    let t20 = t14 + t19;
    let t21 = t20.sqrt();
    let t22 = t21 / bh::ld(hex.top_k[2]);
    work.shares[0] = stf(t22);
}

/// `frame_up`'s arithmetic as plain Rust on the x87 model, in the same order.
fn frame_up_x87(hex: &WindscreenDropVars, work: &mut WindscreenDropWork, m: &EffectsSettings) {
    let t0 = ldi_cell(&m.light_point_a_negated[0]);
    let t1 = t0 - ldi_cell(&m.light_point_b_negated[0]);
    work.axes[3] = stf(t1);
    let t2 = ldi_cell(&m.light_point_a_negated[1]);
    let t3 = t2 - ldi_cell(&m.light_point_b_negated[1]);
    work.axes[4] = stf(t3);
    let t4 = ldi_cell(&m.light_point_a_negated[2]);
    let t5 = t4 - ldi_cell(&m.light_point_b_negated[2]);
    work.axes[5] = stf(t5);
    let t6 = bh::ld(work.axes[3]);
    let t7 = t6 * t6;
    let t8 = bh::ld(work.axes[4]);
    let t9 = t8 * t8;
    let t10 = bh::ld(work.axes[5]);
    let t11 = t10 * t10;
    let t12 = t9 + t11;
    let t13 = t7 + t12;
    let t14 = t13.sqrt();
    let t15 = t14 / bh::ld(hex.top_k[3]);
    work.shares[1] = stf(t15);
}

/// `frame_place`'s arithmetic as plain Rust on the x87 model, in the same order.
fn frame_place_x87(hex: &mut WindscreenDropVars, m: &EffectsSettings) {
    let t0 = ldi_cell(&m.light_point_b_negated[0]);
    hex.place[0] = stf(t0);
    let t1 = ldi_cell(&m.light_point_b_negated[1]);
    hex.place[1] = stf(t1);
    let t2 = ldi_cell(&m.light_point_b_negated[2]);
    hex.place[2] = stf(t2);
}

/// `frame_facing`'s arithmetic as plain Rust on the x87 model, in the same order.
fn frame_facing_x87(hex: &WindscreenDropVars, work: &WindscreenDropWork, drops: &WindscreenDrops) {
    let t0 = bh::ld(hex.place[0]);
    let t1 = t0 * bh::ld(work.axes[6]);
    let t2 = bh::ld(hex.place[1]);
    let t3 = t2 * bh::ld(work.axes[7]);
    let t4 = bh::ld(hex.place[2]);
    let t5 = t4 * bh::ld(work.axes[8]);
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    drops.state.borrow_mut().facing = stf(t7);
}

/// `point_turned`'s arithmetic as plain Rust on the x87 model, in the same order.
fn point_turned_x87(axes: &[f32; 9], p: &mut [f32; 3]) {
    let t0 = bh::ld(p[0]);
    let t1 = bh::ld(p[1]);
    let t2 = t0 * bh::ld(axes[0]);
    let t3 = t1 * bh::ld(axes[3]);
    let t4 = t2 + t3;
    p[0] = stf(t4);
    let t5 = t0 * bh::ld(axes[1]);
    let t6 = t1 * bh::ld(axes[4]);
    let t7 = t5 + t6;
    p[1] = stf(t7);
    let t8 = t1 * bh::ld(axes[5]);
    let t9 = t0 * bh::ld(axes[2]);
    let t10 = t8 + t9;
    p[2] = stf(t10);
}

#[cfg(test)]
mod tests;
