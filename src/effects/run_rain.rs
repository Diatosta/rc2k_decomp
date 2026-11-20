//! `effects::run_rain`: the rain's frame. The drops (the settings' count, held to 0x100; none
//! leaves `drawn` 0) are drawn as streaks by `effects::draw_rain_streak` from their noise places
//! plus the view's place and the drift (wrapped to 0x40000 and centred). Each frame the view's move
//! since the last frame is scaled and held to a length, the view's place is kept per view entry,
//! and the drift follows the clock when the drift switch is set; the drops' turned places are kept
//! when the settings' `other_marks_shown` is set. The quad vertices' rhw is the float 256.0.
//!
//! Screen: the falling rain, seen from inside and outside the car (confirmed: gone with
//! `draw_rain_streak`'s draw cut).
//!
//! Works on the effects settings.
//!
//! The x87 blocks are plain Rust on the x87 model (`math::x87_float`), over the game's variables.
//! Refuses null settings and a view entry past the five (the settings' `marks_layout`), before anything.

use crate::effects::rain::{RainRules, RainState, Weather};
use crate::errors::{BadInput, Refused};
use crate::math::cell_x87::CellX87;
use crate::math::x87;
use crate::race::owner::Race;

const MOST_DROPS: i32 = 0x100;
const ENTRIES: u32 = 5;
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];
const COLOURS: [u32; 4] = [0xFF_FFFF, 0, 0x2800_0000, 0x28FF_FFFF];

/// The view's move since the last frame, scaled, into its entry.
fn box_moved(weather: &Weather, i: usize) {
    box_moved_x87(i, &weather.rain, &weather.rain_rules);
}

/// The move's length into `depth`, compared with `move_limit` (the compare's status word
/// goes to the shared FPU status variable).
fn move_length(weather: &Weather, work: &crate::effects::work_store::MarksWork) {
    move_length_x87(&weather.rain, &weather.rain_rules, work);
}

/// The move scaled to `move_held` over its length, times its three factors.
fn move_held(weather: &Weather) {
    move_held_x87(&weather.rain, &weather.rain_rules);
}

/// The view's y plus the drift (rounded) kept in its entry.
fn keep_y(weather: &Weather, i: usize) {
    keep_y_x87(i, &weather.rain);
}

pub fn run_rain(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    race: &Race,
    weather: &Weather,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let part = scene.scratch.game_scratch();
    let setup = &effects.setup;
    use crate::scene::set_render_state::set_render_state;
    let r = &weather.rain;
    {
        let marks = effects.settings()?;
        if marks.marks_layout.get() >= ENTRIES {
            return Err(BadInput("a rain view entry past the five").into());
        }
        let mut drops = marks.rain_drops.get() as i32;
        r.drops.set(drops as u32);
        if drops == 0 {
            r.drawn.set(0);
            return Ok(());
        }
        if drops > MOST_DROPS {
            drops = MOST_DROPS;
            r.drops.set(drops as u32);
        }
        for n in 0..9 {
            r.view_matrix[n].set(marks.listener_turn[n].get());
        }
        for n in 0..3 {
            r.view_place[n].set(marks.listener_place[n].get());
        }
        crate::effects::place_rain_view::place_rain_view(weather)?;
        crate::scene::depth_buffer_on::depth_buffer(part, false);
        for (value, state) in STATES {
            set_render_state(part, value, state);
        }
        // What `clear_texture` returns is read by nothing here.
        crate::scene::clear_texture::clear_texture(part, scene);
        // The rhw is 256.0 as a float: its bits taken as a whole number made 1132462080.0,
        // which the handheld's GL clipped the streaks away by.
        let rhw = weather.rain_rules.rhw;
        for (n, colour) in COLOURS.into_iter().enumerate() {
            let v = &buffer.vertices[n];
            v.rhw.set(rhw);
            v.colour.set(colour);
            v.specular.set(0);
            v.u.set(0.0);
            v.v.set(0.0);
        }
        scene
            .scratch
            .front_end_polygon()
            .polygon_vertex_count
            .set(4);
        if setup.kind_floats.running.get() != 0 {
            crate::effects::drift_offsets::drift_offsets(race, weather, effects);
        }
        let marks = effects.settings()?;
        let e = marks.marks_layout.get() as usize;
        if marks.own_marks_shown.get() != 0 || setup.kind_floats.running.get() != 0 {
            box_moved(weather, e);
        }
        for n in 0..3 {
            r.motion[n].set(r.moves[e][n].get());
        }
        move_length(weather, &setup.work);
        let status = setup.work.fpu_status.get() & 0x700;
        setup.work.fpu_status.set(status);
        if status != 0x100 {
            move_held(weather);
        }
        r.last_views[e][0].set(r.view[0].get());
        keep_y(weather, e);
        r.last_views[e][2].set(r.view[2].get());
        if setup.kind_floats.running.get() != 0 {
            let clock = race.shared.physics_tables.get().scene_clock.get() & 0xF_FFFF;
            r.drift
                .set(x87::int_times_to_int(clock as i32, weather.rain_rules.drift_k) as u32);
        }
        for k in (0..drops.max(0) as usize).rev() {
            let drift = r.drift.get();
            let spread = r.drift_halves[0].get();
            let view = |n: usize| r.view[n].get();
            let noise = |n: usize| r.noise[n][k].get();
            let x = noise(0).wrapping_add(view(0)).wrapping_add(spread);
            let y = noise(1).wrapping_add(drift).wrapping_add(view(1));
            let z = noise(2).wrapping_add(view(2)).wrapping_add(spread);
            let [x, y, z] = [x, y, z].map(|v| (v & 0x3_FFFF).wrapping_sub(0x1_FFFF));
            let place = [
                x87::int_80(z as i32, false),
                x87::int_80(y as i32, false),
                x87::int_80(x as i32, false),
            ];
            crate::effects::draw_rain_streak::draw_rain_streak(
                buffer, weather, place, scene, effects,
            )?;
            let marks = effects.settings()?;
            if marks.other_marks_shown.get() != 0 {
                for (n, kept) in r.kept.iter().enumerate() {
                    kept[k].set(x87::to_int(&r.turned[n]) as u32);
                }
            }
        }
        r.drawn.set(1);
    }
    Ok(())
}

use crate::math::x87_float as bh;

/// `box_moved`'s arithmetic as plain Rust on the x87 model, in the same order.
fn box_moved_x87(i: usize, r: &RainState, k: &RainRules) {
    {
        let t0 = bh::int(r.view[0].get() as i32);
        let t1 = t0 - bh::int(r.last_views[i][0].get() as i32);
        let t2 = t1 * bh::ld(k.move_scale[0]);
        r.moves[i][0].st(t2);
        let t3 = bh::int(r.view[1].get() as i32);
        let t4 = t3 + bh::int(r.drift.get() as i32);
        let t5 = t4 - bh::int(r.last_views[i][1].get() as i32);
        let t6 = t5 * bh::ld(k.move_scale[1]);
        r.moves[i][1].st(t6);
        let t7 = bh::int(r.view[2].get() as i32);
        let t8 = t7 - bh::int(r.last_views[i][2].get() as i32);
        let t9 = t8 * bh::ld(k.move_scale[2]);
        r.moves[i][2].st(t9);
    }
}

/// `move_length`'s arithmetic as plain Rust on the x87 model, in the same order.
fn move_length_x87(r: &RainState, k: &RainRules, work: &crate::effects::work_store::MarksWork) {
    let t0 = r.motion[0].ld();
    let t1 = t0 * t0;
    let t2 = r.motion[1].ld();
    let t3 = t2 * t2;
    let t4 = r.motion[2].ld();
    let t5 = t4 * t4;
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    let t8 = t7.sqrt();
    r.depth.st(t8);
    crate::effects::plain::set_status(work, bh::compare(t8, bh::ld(k.move_limit)));
}

/// `move_held`'s arithmetic as plain Rust on the x87 model, in the same order.
fn move_held_x87(r: &RainState, k: &RainRules) {
    {
        let t0 = r.depth.ld();
        let t1 = bh::ld(k.move_held) / t0;
        let t2 = t1 * r.motion[0].ld();
        let t3 = t2 * bh::ld(k.move_factors[0]);
        r.motion[0].st(t3);
        let t4 = t1 * r.motion[1].ld();
        let t5 = t4 * bh::ld(k.move_factors[1]);
        r.motion[1].st(t5);
        let t6 = t1 * r.motion[2].ld();
        let t7 = t6 * bh::ld(k.move_factors[2]);
        r.motion[2].st(t7);
    }
}

/// `keep_y`'s arithmetic as plain Rust on the x87 model, in the same order.
fn keep_y_x87(i: usize, r: &RainState) {
    {
        let t0 = bh::int(r.view[1].get() as i32);
        let t1 = t0 + bh::int(r.drift.get() as i32);
        r.last_views[i][1].set(bh::to_int(t1) as u32);
    }
}

#[cfg(test)]
mod tests;
