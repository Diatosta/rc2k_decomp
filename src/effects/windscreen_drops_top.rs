//! `effects::windscreen_drops_top`: the windscreen drops' frame from the settings: the vertex ring
//! from its start, the ease and the rate, the place (the settings' +0x2720 ..) and two axes made
//! one long and square, then (only when the place's way faces away from the third axis) the 0x30
//! windscreen drop points scaled and turned onto the axes. Unless the rate is 0, the drops' frame
//! runs, the vertices left in the ring are drawn (`d3d::draw_marks_windscreen_drops`, the
//! platform's), and with the settings' +0x2744 and the drift on the drops step. Finally the marks'
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
use crate::effects::plain::{ld_cell, ldi_cell, st_cell};
use crate::effects::setup::MarksClock;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::Refused;
use crate::math::cell_x87::CellX87;
use crate::math::x87;
use crate::memory::session::PlainFloat;
use crate::race::effects_settings_store::EffectsSettings;
use crate::race::owner_frame::Frame;
use crate::scene::cross_product3::cross;
use crate::scene::vector3::normalise;

const POINTS: u32 = 0x30;

/// The ease (by the ticks), the rate, the place and the first axis from the settings, and
/// the first axis's length over its divisor into the first share.
fn frame_axes(
    drops: &WindscreenDrops,
    m: &EffectsSettings,
    effects: &crate::effects::owner::Effects,
) {
    frame_axes_x87(drops, m, &effects.setup.clock);
}

/// The second axis from the settings, its length over its divisor into the second share.
fn frame_up(drops: &WindscreenDrops, m: &EffectsSettings) {
    frame_up_x87(drops, m);
}

/// The place again as floats into the drops' `place`.
fn frame_place(drops: &WindscreenDrops, m: &EffectsSettings) {
    frame_place_x87(drops, m);
}

/// The place's way dotted with the third axis into `facing`.
fn frame_facing(drops: &WindscreenDrops) {
    frame_facing_x87(drops);
}

/// A point (x, y at p) turned onto the axes (x, y, z back at p).
fn point_turned(drops: &WindscreenDrops, p: &[PlainFloat; 3]) {
    point_turned_x87(drops, p);
}

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
    drops.hex.buffer.set(0);
    frame_axes(drops, m, effects);
    let w = &drops.work.axes;
    let (a0, a1, a2) = (&w[0..3], &w[3..6], &w[6..9]);
    normalise(a0);
    frame_up(drops, m);
    normalise(a1);
    cross(a1, a0, a2);
    normalise(a2);
    cross(a0, a2, a1);
    frame_place(drops, m);
    normalise(&drops.hex.place);
    frame_facing(drops);
    if !drops.state.facing.get().is_sign_negative() {
        return Ok(());
    }
    let [kx, ky] = drops
        .work
        .shares
        .each_ref()
        .map(crate::memory::session::PlainFloat::get);
    for n in 0..POINTS as usize {
        let (p, o) = (3 * n, 3 * n);
        drops.work.scaled[o].set(x87::mul(&drops.work.points[p], kx));
        drops.work.scaled[o + 1].set(x87::mul(&drops.work.points[p + 1], ky));
        drops.work.scaled[o + 2].set(0.0);
    }
    for p in drops.work.scaled.as_chunks::<3>().0 {
        point_turned(drops, p);
    }
    let rate = drops.state.rate.get();
    // Any rate but +0 (the bits tested).
    if rate != 0.0 || rate.is_sign_negative() {
        crate::effects::windscreen_drops_frame::windscreen_drops_frame(
            buffer, frame, masks, drops, scene, gfx, effects,
        )?;
        let at_now: u32 = drops.hex.buffer.get();
        if at_now != 0 {
            let quads = x87::int_over_int_to_int(at_now as i32, drops.hex.per_vertex.get()) as u32;
            drops.hex.buffer.set(0);
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
        if marks.windscreen_wet.get() != 0 && effects.setup.kind_floats.running.get() != 0 {
            crate::effects::windscreen_drops::windscreen_drops_step(masks, drops)?;
        }
    }
    crate::scene::set_texture::set_texture(
        scene.scratch.game_scratch(),
        scene,
        scene.textures.kinds.get().kind_first_slots[202].get(),
    );
    scene
        .scratch
        .front_end_polygon()
        .polygon_vertex_count
        .set(4);
    Ok(())
}

/// `frame_axes`'s arithmetic as plain Rust on the x87 model, in the same order.
fn frame_axes_x87(drops: &WindscreenDrops, m: &EffectsSettings, c: &MarksClock) {
    let t0 = ld_cell(&drops.hex.top_k[0]);
    let t1 = t0 * crate::math::x87_float::int(c.frame_ticks.get() as i32);
    st_cell(&drops.hex.ease, t1);
    let t2 = ldi_cell(&m.windscreen_drops_rate_factor);
    let t3 = t2 * ld_cell(&drops.hex.top_k[1]);
    drops.state.rate.st(t3);
    let t4 = ldi_cell(&m.light_point_b_negated[0]);
    let t5 = ldi_cell(&m.light_point_b_negated[1]);
    let t6 = ldi_cell(&m.light_point_b_negated[2]);
    drops.work.place[2].st(t6);
    drops.work.place[1].st(t5);
    drops.work.place[0].st(t4);
    let t7 = ldi_cell(&m.light_point_b[0]);
    let t8 = t7 - ldi_cell(&m.light_point_b_negated[0]);
    drops.work.axes[0].st(t8);
    let t9 = ldi_cell(&m.light_point_b[1]);
    let t10 = t9 - ldi_cell(&m.light_point_b_negated[1]);
    drops.work.axes[1].st(t10);
    let t11 = ldi_cell(&m.light_point_b[2]);
    let t12 = t11 - ldi_cell(&m.light_point_b_negated[2]);
    drops.work.axes[2].st(t12);
    let t13 = drops.work.axes[0].ld();
    let t14 = t13 * t13;
    let t15 = drops.work.axes[1].ld();
    let t16 = t15 * t15;
    let t17 = drops.work.axes[2].ld();
    let t18 = t17 * t17;
    let t19 = t16 + t18;
    let t20 = t14 + t19;
    let t21 = t20.sqrt();
    let t22 = t21 / ld_cell(&drops.hex.top_k[2]);
    drops.work.shares[0].st(t22);
}

/// `frame_up`'s arithmetic as plain Rust on the x87 model, in the same order.
fn frame_up_x87(drops: &WindscreenDrops, m: &EffectsSettings) {
    let t0 = ldi_cell(&m.light_point_a_negated[0]);
    let t1 = t0 - ldi_cell(&m.light_point_b_negated[0]);
    drops.work.axes[3].st(t1);
    let t2 = ldi_cell(&m.light_point_a_negated[1]);
    let t3 = t2 - ldi_cell(&m.light_point_b_negated[1]);
    drops.work.axes[4].st(t3);
    let t4 = ldi_cell(&m.light_point_a_negated[2]);
    let t5 = t4 - ldi_cell(&m.light_point_b_negated[2]);
    drops.work.axes[5].st(t5);
    let t6 = drops.work.axes[3].ld();
    let t7 = t6 * t6;
    let t8 = drops.work.axes[4].ld();
    let t9 = t8 * t8;
    let t10 = drops.work.axes[5].ld();
    let t11 = t10 * t10;
    let t12 = t9 + t11;
    let t13 = t7 + t12;
    let t14 = t13.sqrt();
    let t15 = t14 / ld_cell(&drops.hex.top_k[3]);
    drops.work.shares[1].st(t15);
}

/// `frame_place`'s arithmetic as plain Rust on the x87 model, in the same order.
fn frame_place_x87(drops: &WindscreenDrops, m: &EffectsSettings) {
    let t0 = ldi_cell(&m.light_point_b_negated[0]);
    st_cell(&drops.hex.place[0], t0);
    let t1 = ldi_cell(&m.light_point_b_negated[1]);
    st_cell(&drops.hex.place[1], t1);
    let t2 = ldi_cell(&m.light_point_b_negated[2]);
    st_cell(&drops.hex.place[2], t2);
}

/// `frame_facing`'s arithmetic as plain Rust on the x87 model, in the same order.
fn frame_facing_x87(drops: &WindscreenDrops) {
    let t0 = ld_cell(&drops.hex.place[0]);
    let t1 = t0 * drops.work.axes[6].ld();
    let t2 = ld_cell(&drops.hex.place[1]);
    let t3 = t2 * drops.work.axes[7].ld();
    let t4 = ld_cell(&drops.hex.place[2]);
    let t5 = t4 * drops.work.axes[8].ld();
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    drops.state.facing.st(t7);
}

/// `point_turned`'s arithmetic as plain Rust on the x87 model, in the same order.
fn point_turned_x87(drops: &WindscreenDrops, p: &[PlainFloat; 3]) {
    let t0 = ld_cell(&p[0]);
    let t1 = ld_cell(&p[1]);
    let t2 = t0 * drops.work.axes[0].ld();
    let t3 = t1 * drops.work.axes[3].ld();
    let t4 = t2 + t3;
    st_cell(&p[0], t4);
    let t5 = t0 * drops.work.axes[1].ld();
    let t6 = t1 * drops.work.axes[4].ld();
    let t7 = t5 + t6;
    st_cell(&p[1], t7);
    let t8 = t1 * drops.work.axes[5].ld();
    let t9 = t0 * drops.work.axes[2].ld();
    let t10 = t8 + t9;
    st_cell(&p[2], t10);
}

#[cfg(test)]
mod tests;
