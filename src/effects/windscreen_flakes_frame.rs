//! `effects::windscreen_flakes_frame`: the windscreen flakes' frame (as `effects::draw_plane`'s,
//! with the rate times one constant and the lengths over two more): the rate, the place and two
//! axes from the settings, made one long and square, scaled (`effects::scale_axes`). Nothing more
//! happens unless the place's way faces away from the third axis; then `effects::run_pong` runs;
//! unless the rate times a constant comes to 0 (rounded as the x87 is set),
//! `effects::run_windscreen_flakes` runs, and with the settings' `windscreen_wet` and the drift switch
//! `effects::windscreen_flakes_step`.
//!
//! Screen: the snow settling on the windscreen in the in-car view (confirmed: gone with
//! `draw_windscreen_flake` cut). Drawn as squares.
//!
//! Works on the effects settings. The x87 runs are plain Rust on the x87 model
//! (`math::x87_float`). Refuses null settings and `windscreen_flakes_step`'s refusals.

use crate::effects::mask_store::Masks;
use crate::effects::plain::ldi_cell;
use crate::effects::windscreen_drop_vars_store::WindscreenDrops;
use crate::effects::windscreen_flakes::windscreen_flakes_step;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::Refused;
use crate::math::cell_x87::CellX87;
use crate::math::x87;
use crate::race::effects_settings_store::EffectsSettings;
use crate::scene::cross_product3::cross;
use crate::scene::vector3::normalise;

/// The rate (`WindscreenFlakeTiming.rate`), the place and the first axis from the settings, its
/// length over `WindscreenFlakeState.length_divisors[0]` into `WindscreenFlakeTables.lengths[0]`.
fn frame_axes(fl: &WindscreenFlakes, m: &EffectsSettings) {
    frame_axes_x87(m, &fl.timing, &fl.tables, &fl.state);
}

/// The second axis, its length over `WindscreenFlakeState.length_divisors[1]` into
/// `WindscreenFlakeTables.lengths[1]`.
fn frame_up(fl: &WindscreenFlakes, m: &EffectsSettings) {
    frame_up_x87(m, &fl.tables, &fl.state);
}

/// The place as floats into `WindscreenDropVars.place[0]` ...
fn frame_place(drops: &WindscreenDrops, m: &EffectsSettings) {
    frame_place_x87(drops, m);
}

/// The place's way dotted with the third axis into `WindscreenDropState.facing`.
fn frame_facing(drops: &WindscreenDrops, fl: &WindscreenFlakes) {
    frame_facing_x87(drops, &fl.tables);
}

/// The flakes are tinted by `ctx`'s view's light.
pub fn windscreen_flakes_frame(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    masks: &Masks,
    drops: &WindscreenDrops,
    pong: &crate::effects::pong_store::Pong,
    fl: &WindscreenFlakes,
    cheats: &crate::menu::cheats_store::Cheats,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let buffer = &ctx.part.vertices;
    let scene = ctx.scene;
    let m = effects.settings()?;
    frame_axes(fl, m);
    let [a0, a1, a2] = [0, 1, 2].map(|n| &fl.tables.axes[n]);
    normalise(a0);
    frame_up(fl, m);
    normalise(a1);
    cross(a1, a0, a2);
    normalise(a2);
    cross(a0, a2, a1);
    crate::effects::scale_axes::scale_axes(fl);
    frame_place(drops, m);
    normalise(&drops.hex.place);
    frame_facing(drops, fl);
    if !drops.state.facing.get().is_sign_negative() {
        return Ok(());
    }
    crate::effects::run_pong::run_pong(buffer, ctx.race, pong, fl, scene, cheats, effects)?;
    if x87::mul_to_int(&fl.timing.rate, &fl.state.share) == 0 {
        return Ok(());
    }
    crate::effects::run_windscreen_flakes::run_windscreen_flakes(
        buffer,
        masks,
        drops,
        fl,
        ctx.view.light.view_light.get(),
        effects,
        scene,
        &ctx.race.shared,
    )?;
    let marks = effects.settings()?;
    if marks.windscreen_wet.get() == 0 {
        return Ok(());
    }
    if effects.setup.kind_floats.running.get() == 0 {
        return Ok(());
    }
    windscreen_flakes_step(masks, drops, fl)?;
    Ok(())
}

use crate::effects::plain::{ld_cell, st_cell};
use crate::effects::windscreen_flakes_store::{
    WindscreenFlakeState, WindscreenFlakeTables, WindscreenFlakeTiming,
};

/// `frame_axes`'s arithmetic as plain Rust on the x87 model, in the same order.
fn frame_axes_x87(
    m: &EffectsSettings,
    sti: &WindscreenFlakeTiming,
    sq: &WindscreenFlakeTables,
    ss: &WindscreenFlakeState,
) {
    let t0 = ldi_cell(&m.windscreen_flakes_rate_factor);
    let t1 = t0 * ss.rate_share.ld();
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
    let t20 = t19 / ss.length_divisors[0].ld();
    sq.lengths[0].st(t20);
}

/// `frame_up`'s arithmetic as plain Rust on the x87 model, in the same order.
fn frame_up_x87(m: &EffectsSettings, sq: &WindscreenFlakeTables, ss: &WindscreenFlakeState) {
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
    let t15 = t14 / ss.length_divisors[1].ld();
    sq.lengths[1].st(t15);
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
fn frame_facing_x87(drops: &WindscreenDrops, sq: &WindscreenFlakeTables) {
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

#[cfg(test)]
mod tests;
