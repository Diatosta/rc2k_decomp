//! The cars' and lamps' light on the lit place, and the light colour of the view.

use super::*;
use crate::errors::{BadInput, Refused};
use crate::net::owner::net;
use crate::race::owner::Race;
use crate::race::owner::race;
use crate::race::owner_course::Course;
use crate::race::owner_shared::Shared;
use crate::race::trap::Trap;
use crate::race::trap::car_at;
use crate::race::view_scratch::DrawCtx;
use crate::session::Dword;

/// `race::car_light_modes` (0x455B70): `crate::race::owner::race().shared.view.shown_car` = the drawn view's car
/// (`crate::race::owner::race().shared.view.view_cars[g_drawnView]`); every car block's +0x6FC 0; then for each race car with
/// lights (+0x25C) that is drawn (any car in modes 0 and 2, else the shown car only; the
/// shown car not with +0x69C and +0x684 both 0, nor with `GROUND_WORK.view_mirrored` set): +0x6FC's
/// nibbles set step by step: 0x21; for the shown car with `race().frame.view_and_records.view_vars.view_car_flag_b`, 0x12, then 0x14
/// unless `race().frame.view_and_records.view_vars.view_car_stage` is 1, then 0x11 unless it is 2 or 3; then 0x100 in the third
/// nibble, 0x200 with `crate::race::owner::race().shared.vars.settings_words[3]` (1: the shown car only). Refuses (before anything) a null block among the eight or more than eight cars.
pub fn car_light_modes(race: &Race) -> Result<(), Refused> {
    let block = |n: u32| race.shared.work.car_block(n as usize);
    let cars: u32 = race.shared.view.race_cars.bits();
    if (0..8).any(|n| block(n) == 0) {
        return Err(BadInput("a null car block").into());
    }
    if cars > 8 {
        return Err(BadInput("more than eight race cars").into());
    }
    let view: u32 = race.shared.view.drawn_view.bits();
    let shown = race
        .shared
        .view
        .view_cars
        .get(view as usize)
        .ok_or(BadInput("a drawn view past the two"))?
        .get();
    race.shared.view.shown_car.set_bits(shown);
    for n in 0..8 {
        car_at(block(n))?.state_bits.set(0);
    }
    // Converted: `rc2k_core::race::lights::{lights_drawn, light_word}`.
    use rc2k_core::race::lights::{light_word, lights_drawn};
    let mode: u32 = race.shared.vars.run_kind.bits();
    for n in 0..cars {
        // The state bits are the drawing's own (in the block); what decides them, the physics'.
        let this_car = crate::race::frame_view::car_at(block(n))?;
        let at = &car_at(block(n))?.state_bits;
        let me = n == shown;
        let drawn = lights_drawn(
            this_car.driver_kind.bits() != 0,
            mode,
            me,
            || this_car.camera_switch_share.bits() == 0 && this_car.camera_view.bits() == 0,
            || race.physics.ground.view_mirrored.get() != 0,
        );
        if !drawn {
            at.set(0);
            continue;
        }
        let word = light_word(
            at.get(),
            me,
            || race.frame.view_and_records.view_vars.view_car_flag_b.get() != 0,
            || race.frame.view_and_records.view_vars.view_car_stage.bits(),
            race.shared.vars.settings_words[3].bits(),
        );
        at.set(word);
    }
    Ok(())
}
/// Cell `n` of `g_courseGrid` (32 x 32). One past the grid refused (the original reads on).
pub(super) fn course_grid_cell(n: u32) -> Result<&'static Dword, BadInput> {
    race()
        .course
        .level_grid
        .course_grid
        .get(n as usize)
        .ok_or(BadInput("a course grid cell past the 32 x 32"))
}

/// Lamp `n` of `g_lamps` (16 of 0x10 bytes: +0 the point, +4 the strength, +8 the
/// colour). One past the 16 refused (the original reads on).
pub(super) fn lamp(course: &Course, n: u32) -> Result<&'_ [Dword], BadInput> {
    let n = n as usize;
    course
        .level_grid
        .lamps
        .get(4 * n..4 * n + 4)
        .ok_or(BadInput("a lamp past the sixteen"))
}

/// `race::add_lamp_entry` (0x4573FA): the lamp's point (`g_lamps` +0) through
/// `g_viewMatrix` (the point into `OBJECT_WORK.place` .. `OBJECT_WORK.place[2]`, then x and y turned
/// there, z rounded into `g_faceDepthKey`, + 0x2000000 and shifted right 10), and a depth
/// entry of that key with `g_lampCount` + 0x3000000; `race().frame.frame_work.lamp_entry_count` + 1. The caller checks
/// the room.
pub(super) fn add_lamp_entry(ctx: &DrawCtx, lamp: u32) -> Result<(), BadInput> {
    let DrawCtx { race, scene, view } = *ctx;
    let frame = &race.frame;
    let point = self::lamp(&race.course, lamp)?[0].get();
    let place = crate::race::scene_points::point_number(point)?;
    let turned = &scene.drawing.object_work.place;
    for (cell, &v) in turned.iter().zip(&place) {
        cell.set(v);
    }
    let matrix: [u32; 16] = crate::session::read_bits(&frame.matrices.view_matrix);
    let out = crate::math::x87_load::rotate_rounding_z(place.map(|v| v as i32), &matrix);
    turned[0].set(out[0]);
    turned[1].set(out[1]);
    let key = out[2].wrapping_add(0x200_0000) >> 10;
    race.shared.view.face_depth_key.set_bits(key);
    let second = race.shared.view.lamp_count.bits().wrapping_add(0x300_0000);
    let n: u32 = view.lists.depth_entry_count.bits();
    let at = view.lists.depth_entry(n)?;
    at[0].set(key);
    at[1].set(second);
    view.lists.depth_entry_count.set_bits(n.wrapping_add(1));
    frame
        .frame_work
        .lamp_entry_count
        .set_bits(frame.frame_work.lamp_entry_count.bits().wrapping_add(1));
    Ok(())
}

/// `race::gather_lamps` (0x454E08): `g_lampCount` 0; each lit face (`g_litFaces`, the count
/// `g_litFaceCount`) whose object (+1) is 0x3B becomes a lamp while there are fewer than 16:
/// `g_lamps` entry (+0 the face's first point, +4 its +0x11 byte, +8 0xFFFFFF) and its depth
/// entry (`add_lamp_entry`. Refuses (before anything) more than 0x100 lit faces, or a depth
/// list without room for 16.
pub fn gather_lamps(ctx: &DrawCtx) -> Result<(), Refused> {
    let race = ctx.race;
    let course = &race.course;
    let physics = &race.physics;
    let count: u32 = physics.contact.lit_face_count.bits();
    if count > 0x100 {
        return Err(BadInput("more than 0x100 lit faces").into());
    }
    if count != 0 && ctx.view.lists.depth_entry_count.bits() > SORT_MAX - 16 {
        return Err(BadInput("the depth list has no room for the lamps").into());
    }
    race.shared.view.lamp_count.set_bits(0u32);
    for k in 0..count {
        let face = physics.contact.lit_faces[k as usize].bits();
        // Converted: `rc2k_core::race::faces::takes_lamp`.
        let object = face_field::<u8>(face, 1)? as u32;
        if object != rc2k_core::race::faces::LAMP_OBJECT {
            continue;
        }
        let lamp: u32 = race.shared.view.lamp_count.bits();
        if !rc2k_core::race::faces::takes_lamp(object, lamp) {
            continue;
        }
        let at = self::lamp(course, lamp)?;
        at[0].set(face_field(face, 4)?);
        at[1].set(crate::race::course_places::face_number(face, "a lamp's face")?.mark() as u32);
        at[2].set(rc2k_core::race::faces::LAMP_COLOUR);
        add_lamp_entry(ctx, lamp)?;
        race.shared.view.lamp_count.set_bits(lamp + 1);
    }
    Ok(())
}

/// Whether the game uses MMX (`g_hasMmx`).
pub fn has_mmx(system: &crate::system::owner::System) -> bool {
    system.session.machine.has_mmx.get() != 0
}

/// Two colours added byte by byte (`rc2k_core::race::lights::add_colours`), MMX or not
/// as the game.
pub fn add_colours(a: u32, b: u32) -> u32 {
    let system = crate::system::owner::system();
    rc2k_core::race::lights::add_colours(a, b, has_mmx(system))
}

/// 0x44F691: a car's headlights on the lit place, added to `g_lightColour` (see
/// `light_colour_at`). Called.
pub(super) fn car_light_on_point(
    course: &Course,
    car: &crate::race::frame_view::CarView,
    marks_count: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    use crate::math::x87_load::{int_square_sum_scaled_to_int, rotate_rounding};
    use rc2k_core::race::lights::headlight_index;
    if car.lights_on.bits() == 0 {
        return Ok(());
    }
    let place = &scene.lights.state.lit_place;
    let d = [
        place[0].get().wrapping_sub(car.position[0].bits()),
        place[1].get().wrapping_sub(car.position[1].bits()),
        place[2].get().wrapping_sub(car.position[2].bits()),
    ];
    let t = rotate_rounding(
        d.map(|v| v as i32),
        &crate::session::read_bits(&car.car_inverse_matrix),
    );
    crate::session::write_bits(&scene.lights.state.turned_light, t);
    let k: u32 = course.numbers.one_over_65536.bits();
    let s1 = int_square_sum_scaled_to_int(
        t[2].wrapping_sub(0x20_0000) as i32,
        t[0].wrapping_add(t[0]) as i32,
        k,
        2,
    );
    // The original worked out a second spread and stored it in `g_scratch`; nothing read it, so it is not kept.
    let index = headlight_index(s1, t[1], marks_count);
    let colour = scene.light_tables.car_light_table[index as usize].get();
    scene
        .lights
        .work
        .light_colour
        .set_bits(add_colours(colour, scene.lights.work.light_colour.bits()));
    Ok(())
}

/// 0x44F601: the cars' headlights on the lit place: with `crate::net::owner::net().vars.session`
/// the chosen car
/// (`crate::net::owner::net().vars.player`) or all with `crate::race::owner::race().shared.vars.settings_words[4]` 1; else the players' cars (all with
/// `crate::race::owner::race().shared.vars.settings_words[4]` 1), only the drawn view's car outside one player and modes 0 and 2.
pub(super) fn car_lights_on_point(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    let course = &race.course;
    let vars = &net().vars;
    let shared = &race.shared;
    let cars: u32 = shared.view.race_cars.bits();
    let all = shared.vars.settings_words[4].get() == 1;
    let one = vars.session.get() != 0;
    for n in 0..cars {
        let trap = Trap::new();
        let lit = rc2k_core::race::lights::headlights_light(
            n,
            one,
            || vars.player.get(),
            all,
            || shared.view.players.bits(),
            || shared.vars.run_kind.bits(),
            || {
                let n = shared.view.drawn_view.bits() as usize;
                crate::session::read_on_in(
                    &*shared.view,
                    &shared.view.view_cars,
                    n,
                    "the view cars",
                )
                .unwrap_or_else(|e| {
                    trap.keep(e);
                    0
                })
            },
        );
        trap.finish()?;
        if lit {
            // The marks count is the drawing's own (`race::car_parts`), in the block.
            car_light_on_point(
                course,
                crate::race::frame_view::car(n as usize)?,
                shared.work.car(n as usize)?.marks_count.bits(),
                scene,
            )?;
        }
    }
    Ok(())
}

/// 0x44F863: the lamps' light on the lit place: each lamp with a strength (+4,
/// `GROUND_WORK.point_light_strength`) gives ((0x200000 - its approximate distance) x strength) >> 17 (held to
/// 0 .. 0xFF) as a grey, times its colour (+8, `LIGHT_WORK.lamp_tint`) byte by byte, added to
/// `g_lightColour`.
pub(super) fn lamp_lights_on_point(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    let physics = &race.physics;
    {
        let count: u32 = race.shared.view.lamp_count.bits();
        if count == 0 {
            return Ok(());
        }
        let place = &scene.lights.state.lit_place;
        for n in 0..count {
            let lamp = self::lamp(&race.course, n)?;
            let strength = lamp[1].get();
            physics.ground.point_light_strength.set(strength);
            if strength == 0 {
                continue;
            }
            let tint = lamp[2].get();
            scene.lights.work.lamp_tint.set(tint);
            let point = lamp[0].get();
            let at = crate::race::scene_points::point_number(point)?;
            let d: [u32; 3] = std::array::from_fn(|k| at[k].wrapping_sub(place[k].get()));
            let [distance, _, _] = crate::scene::approximate_distance::approximate_distance(d);
            let (grey, _) = rc2k_core::race::lights::lamp_grey(
                distance,
                physics.ground.point_light_strength.bits(),
            );
            let light = add_colours(
                rc2k_core::race::lights::multiply_colours(grey, tint),
                scene.lights.work.light_colour.bits(),
            );
            scene.lights.work.light_colour.set_bits(light);
        }
    }
    Ok(())
}

/// What `light_colour_at` refuses, before anything: no cars (its loop tests at the end),
/// more than eight, a null block it would read, more than 16 lamps.
pub fn check_light_colour(shared: &Shared) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::lights::check_light_colour`.
    Ok(rc2k_core::race::lights::check_light_colour(
        shared.view.race_cars.bits(),
        |n| u32::from(shared.work.has_car_block(n as usize)),
        || shared.view.lamp_count.bits(),
    )?)
}

/// `race::light_colour_at` (0x4537CE): `g_lightColour` 0, then the cars' headlights
/// (0x44F601) and the lamps (0x44F863) on the lit place (`g_litPlace`,
/// `LIGHT_STATE.lit_place[1]`, `LIGHT_STATE.lit_place[2]`), added up. Refuses as
/// `check_light_colour`.
pub fn light_colour_at(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::lights::{check_light_colour, headlights_light,
    // headlight_index, lamp_grey, add_colours, multiply_colours}` (the turned place, the
    // spreads and the lamps' distances the x87's and the scene's).
    check_light_colour(&race.shared)?;
    scene.lights.work.light_colour.set_bits(0u32);
    car_lights_on_point(race, scene)?;
    lamp_lights_on_point(race, scene)?;
    Ok(())
}

/// `race::view_light` (0x455896): the lit place the view's car's (`race::frame::drawn_car`, the original's `g_car` +0x134 ..);
/// `light_colour_at`; `race().frame.frame_work.view_light` = `LIGHT_WORK.view_light_base` plus the light
/// (`add_colours`).
pub fn view_light(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::lights::add_colours` (through `light_colour_at`'s).
    check_light_colour(&race.shared)?;
    crate::session::write_bits(
        &scene.lights.state.lit_place,
        crate::session::read_bits(&crate::race::frame::drawn_car().position),
    );
    light_colour_at(race, scene)?;
    let light: u32 = scene.lights.work.light_colour.bits();
    race.frame
        .frame_work
        .view_light
        .set(add_colours(scene.lights.work.view_light_base.bits(), light));
    Ok(())
}
