//! The cars' and lamps' light on the lit place, and the light colour of the view.

use super::*;
use crate::errors::{BadInput, Refused};
use crate::net::store::NET;
use crate::physics::car_store::CAR;
use crate::physics::contact_store::CONTACT_WORK;
use crate::physics::ground_store::GROUND_WORK;
use crate::race::frame_store::FRAME_WORK;
use crate::race::level_grid_store::LEVEL_GRID;
use crate::race::matrices_store::MATRICES;
use crate::race::numbers_store::NUMBERS;
use crate::race::trap::Trap;
use crate::race::trap::car_at;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::race::view_vars_store::VIEW_VARS;
use crate::race::work_store::RACE_WORK;
use crate::scene::draw_store::DRAW;
use crate::scene::faces_store::FACES;
use crate::scene::light_store::LIGHT_STATE;
use crate::scene::light_tables_store::LIGHT_TABLES;
use crate::scene::light_work_store::LIGHT_WORK;
use crate::scene::object_store::OBJECT_WORK;
use crate::session::Dword;
use crate::session::MACHINE;

/// `race::car_light_modes` (0x455B70): `RACE_VIEW.shown_car` = the drawn view's car
/// (`RACE_VIEW.view_cars[g_drawnView]`); every car block's +0x6FC 0; then for each race car with
/// lights (+0x25C) that is drawn (any car in modes 0 and 2, else the shown car only; the
/// shown car not with +0x69C and +0x684 both 0, nor with `GROUND_WORK.view_mirrored` set): +0x6FC's
/// nibbles set step by step: 0x21; for the shown car with `VIEW_VARS.view_car_flag_b`, 0x12, then 0x14
/// unless `VIEW_VARS.view_car_stage` is 1, then 0x11 unless it is 2 or 3; then 0x100 in the third
/// nibble, 0x200 with `RACE_VARS.settings_words[3]` (1: the shown car only). Refuses (before anything) a null block among the eight or more than eight cars.
pub fn car_light_modes() -> Result<(), Refused> {
    let block = |n: u32| RACE_WORK.car_block(n as usize);
    let cars: u32 = RACE_VIEW.race_cars.bits();
    if (0..8).any(|n| block(n) == 0) {
        return Err(BadInput("a null car block").into());
    }
    if cars > 8 {
        return Err(BadInput("more than eight race cars").into());
    }
    let view: u32 = RACE_VIEW.drawn_view.bits();
    let shown = RACE_VIEW
        .view_cars
        .get(view as usize)
        .ok_or(BadInput("a drawn view past the two"))?
        .get();
    RACE_VIEW.shown_car.set_bits(shown);
    for n in 0..8 {
        car_at(block(n))?.state_bits.set(0);
    }
    // Converted: `rc2k_core::race::lights::{lights_drawn, light_word}`.
    use rc2k_core::race::lights::{light_word, lights_drawn};
    let mode: u32 = RACE_VARS.run_kind.bits();
    for n in 0..cars {
        let this_car = car_at(block(n))?;
        let at = &this_car.state_bits;
        let me = n == shown;
        let drawn = lights_drawn(
            this_car.driver_kind.bits() != 0,
            mode,
            me,
            || this_car.camera_switch_share.bits() == 0 && this_car.camera_view.bits() == 0,
            || GROUND_WORK.view_mirrored.get() != 0,
        );
        if !drawn {
            at.set(0);
            continue;
        }
        let word = light_word(
            at.get(),
            me,
            || VIEW_VARS.view_car_flag_b.get() != 0,
            || VIEW_VARS.view_car_stage.bits(),
            RACE_VARS.settings_words[3].bits(),
        );
        at.set(word);
    }
    Ok(())
}
/// Cell `n` of `g_courseGrid` (32 x 32). One past the grid refused (the original reads on).
pub(super) fn course_grid_cell(n: u32) -> Result<&'static Dword, BadInput> {
    LEVEL_GRID
        .course_grid
        .get(n as usize)
        .ok_or(BadInput("a course grid cell past the 32 x 32"))
}

/// Lamp `n` of `g_lamps` (16 of 0x10 bytes: +0 the point, +4 the strength, +8 the
/// colour). One past the 16 refused (the original reads on).
pub(super) fn lamp(n: u32) -> Result<&'static [Dword], BadInput> {
    let n = n as usize;
    LEVEL_GRID
        .lamps
        .get(4 * n..4 * n + 4)
        .ok_or(BadInput("a lamp past the sixteen"))
}

/// Depth entry `n` (8 bytes: the key, the second word). One past the 0x1000 refused.
pub(super) fn depth_entry(n: u32) -> Result<&'static [Dword], BadInput> {
    let n = n as usize;
    FACES
        .depth_entries
        .get(2 * n..2 * n + 2)
        .ok_or(BadInput("a depth entry past the table"))
}
/// `race::add_lamp_entry` (0x4573FA): the lamp's point (`g_lamps` +0) through
/// `g_viewMatrix` (the point into `OBJECT_WORK.place` .. `OBJECT_WORK.place[2]`, then x and y turned
/// there, z rounded into `g_faceDepthKey`, + 0x2000000 and shifted right 10), and a depth
/// entry of that key with `g_lampCount` + 0x3000000; `FRAME_WORK.lamp_entry_count` + 1. The caller checks
/// the room.
pub(super) fn add_lamp_entry(lamp: u32) -> Result<(), BadInput> {
    let point = self::lamp(lamp)?[0].get();
    let place = crate::race::scene_points::point_number(point)?;
    let turned = &OBJECT_WORK.place;
    for (cell, &v) in turned.iter().zip(&place) {
        cell.set(v);
    }
    let matrix: [u32; 16] = crate::session::read_bits(&MATRICES.view_matrix);
    let out = crate::math::x87_load::rotate_rounding_z(place.map(|v| v as i32), &matrix);
    turned[0].set(out[0]);
    turned[1].set(out[1]);
    let key = out[2].wrapping_add(0x200_0000) >> 10;
    RACE_VIEW.face_depth_key.set_bits(key);
    let second = RACE_VIEW.lamp_count.bits().wrapping_add(0x300_0000);
    let n: u32 = RACE_VIEW.depth_entry_count.bits();
    let at = depth_entry(n)?;
    at[0].set(key);
    at[1].set(second);
    RACE_VIEW.depth_entry_count.set_bits(n.wrapping_add(1));
    FRAME_WORK
        .lamp_entry_count
        .set_bits(FRAME_WORK.lamp_entry_count.bits().wrapping_add(1));
    Ok(())
}

/// `race::gather_lamps` (0x454E08): `g_lampCount` 0; each lit face (`g_litFaces`, the count
/// `g_litFaceCount`) whose object (+1) is 0x3B becomes a lamp while there are fewer than 16:
/// `g_lamps` entry (+0 the face's first point, +4 its +0x11 byte, +8 0xFFFFFF) and its depth
/// entry (`add_lamp_entry`. Refuses (before anything) more than 0x100 lit faces, or a depth
/// list without room for 16.
pub fn gather_lamps() -> Result<(), Refused> {
    let count: u32 = CONTACT_WORK.lit_face_count.bits();
    if count > 0x100 {
        return Err(BadInput("more than 0x100 lit faces").into());
    }
    if count != 0 && RACE_VIEW.depth_entry_count.bits() > SORT_MAX - 16 {
        return Err(BadInput("the depth list has no room for the lamps").into());
    }
    RACE_VIEW.lamp_count.set_bits(0u32);
    for k in 0..count {
        let face = CONTACT_WORK.lit_faces[k as usize].bits();
        // Converted: `rc2k_core::race::faces::takes_lamp`.
        let object = face_field::<u8>(face, 1)? as u32;
        if object != rc2k_core::race::faces::LAMP_OBJECT {
            continue;
        }
        let lamp: u32 = RACE_VIEW.lamp_count.bits();
        if !rc2k_core::race::faces::takes_lamp(object, lamp) {
            continue;
        }
        let at = self::lamp(lamp)?;
        at[0].set(face_field(face, 4)?);
        at[1].set(crate::race::course_places::face_number(face, "a lamp's face")?.mark() as u32);
        at[2].set(rc2k_core::race::faces::LAMP_COLOUR);
        add_lamp_entry(lamp)?;
        RACE_VIEW.lamp_count.set_bits(lamp + 1);
    }
    Ok(())
}

/// Whether the game uses MMX (`g_hasMmx`).
pub fn has_mmx() -> bool {
    MACHINE.has_mmx.get() != 0
}

/// Two colours added byte by byte (`rc2k_core::race::lights::add_colours`), MMX or not
/// as the game.
pub fn add_colours(a: u32, b: u32) -> u32 {
    rc2k_core::race::lights::add_colours(a, b, has_mmx())
}

/// `sub_44F691`: a car's (block `b`) headlights on the lit place, added to `g_lightColour` (see
/// `light_colour_at`). Called.
pub(super) fn car_light_on_point(b: u32) -> Result<(), BadInput> {
    use crate::math::x87_load::{int_square_sum_scaled_to_int, rotate_rounding};
    use rc2k_core::race::lights::headlight_index;
    let car = car_at(b)?;
    if car.lights_on.bits() == 0 {
        return Ok(());
    }
    let place = &LIGHT_STATE.lit_place;
    let d = [
        place[0].get().wrapping_sub(car.position[0].bits()),
        place[1].get().wrapping_sub(car.position[1].bits()),
        place[2].get().wrapping_sub(car.position[2].bits()),
    ];
    let t = rotate_rounding(
        d.map(|v| v as i32),
        &crate::session::read_bits(&car.car_inverse_matrix),
    );
    crate::session::write_bits(&LIGHT_STATE.turned_light, t);
    let k: u32 = NUMBERS.one_over_65536.bits();
    let s1 = int_square_sum_scaled_to_int(
        t[2].wrapping_sub(0x20_0000) as i32,
        t[0].wrapping_add(t[0]) as i32,
        k,
        2,
    );
    // The second spread is worked out and then not used (eax is loaded over it).
    let s2 = int_square_sum_scaled_to_int(t[2].wrapping_sub(0x10_0000) as i32, t[0] as i32, k, 1);
    DRAW.scratch.set_bits(s2);
    let index = headlight_index(s1, t[1], car.marks_count.bits());
    let colour = LIGHT_TABLES.car_light_table[index as usize].get();
    LIGHT_WORK
        .light_colour
        .set_bits(add_colours(colour, LIGHT_WORK.light_colour.bits()));
    Ok(())
}

/// `sub_44F601`: the cars' headlights on the lit place: with `NET.session` the chosen car
/// (`NET.player`) or all with `RACE_VARS.settings_words[4]` 1; else the players' cars (all with
/// `RACE_VARS.settings_words[4]` 1), only the drawn view's car outside one player and modes 0 and 2.
pub(super) fn car_lights_on_point() -> Result<(), BadInput> {
    let cars: u32 = RACE_VIEW.race_cars.bits();
    let block = |n: u32| RACE_WORK.car_block(n as usize);
    let all = RACE_VARS.settings_words[4].get() == 1;
    let one = NET.session.get() != 0;
    for n in 0..cars {
        let trap = Trap::new();
        let lit = rc2k_core::race::lights::headlights_light(
            n,
            one,
            || NET.player.get(),
            all,
            || RACE_VIEW.players.bits(),
            || RACE_VARS.run_kind.bits(),
            || {
                let n = RACE_VIEW.drawn_view.bits() as usize;
                crate::session::read_on_in(&RACE_VIEW, &RACE_VIEW.view_cars, n, "the view cars")
                    .unwrap_or_else(|e| {
                        trap.keep(e);
                        0
                    })
            },
        );
        trap.finish()?;
        if lit {
            car_light_on_point(block(n))?;
        }
    }
    Ok(())
}

/// `sub_44F863`: the lamps' light on the lit place: each lamp with a strength (+4,
/// `GROUND_WORK.point_light_strength`) gives ((0x200000 - its approximate distance) x strength) >> 17 (held to
/// 0 .. 0xFF) as a grey, times its colour (+8, `LIGHT_WORK.lamp_tint`) byte by byte, added to
/// `g_lightColour`.
pub(super) fn lamp_lights_on_point() -> Result<(), BadInput> {
    {
        let count: u32 = RACE_VIEW.lamp_count.bits();
        if count == 0 {
            return Ok(());
        }
        let place = &LIGHT_STATE.lit_place;
        for n in 0..count {
            let lamp = self::lamp(n)?;
            let strength = lamp[1].get();
            GROUND_WORK.point_light_strength.set(strength);
            if strength == 0 {
                continue;
            }
            let tint = lamp[2].get();
            LIGHT_WORK.lamp_tint.set(tint);
            let point = lamp[0].get();
            let coordinate = |k| crate::race::scene_points::point_word(point, k);
            let d = [
                coordinate(0)?.wrapping_sub(place[0].get()),
                coordinate(1)?.wrapping_sub(place[1].get()),
                coordinate(2)?.wrapping_sub(place[2].get()),
            ];
            let [distance, _, _] = crate::scene::approximate_distance::approximate_distance(d);
            DRAW.scratch.set_bits(distance);
            let (grey, _) = rc2k_core::race::lights::lamp_grey(
                distance,
                GROUND_WORK.point_light_strength.bits(),
            );
            let light = add_colours(
                rc2k_core::race::lights::multiply_colours(grey, tint),
                LIGHT_WORK.light_colour.bits(),
            );
            LIGHT_WORK.light_colour.set_bits(light);
        }
    }
    Ok(())
}

/// What `light_colour_at` refuses, before anything: no cars (its loop tests at the end),
/// more than eight, a null block it would read, more than 16 lamps.
pub fn check_light_colour() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::lights::check_light_colour`.
    Ok(rc2k_core::race::lights::check_light_colour(
        RACE_VIEW.race_cars.bits(),
        |n| RACE_WORK.car_block(n as usize),
        || RACE_VIEW.lamp_count.bits(),
    )?)
}

/// `race::light_colour_at` (0x4537CE): `g_lightColour` 0, then the cars' headlights
/// (`sub_44F601`) and the lamps (`sub_44F863`) on the lit place (`g_litPlace`,
/// `LIGHT_STATE.lit_place[1]`, `LIGHT_STATE.lit_place[2]`), added up. Refuses as
/// `check_light_colour`.
pub fn light_colour_at() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::lights::{check_light_colour, headlights_light,
    // headlight_index, lamp_grey, add_colours, multiply_colours}` (the turned place, the
    // spreads and the lamps' distances the x87's and the scene's).
    check_light_colour()?;
    LIGHT_WORK.light_colour.set_bits(0u32);
    car_lights_on_point()?;
    lamp_lights_on_point()?;
    Ok(())
}

/// `race::view_light` (0x455896): the lit place the work car's (`g_car` +0x134 ..);
/// `light_colour_at`; `FRAME_WORK.view_light` = `LIGHT_WORK.view_light_base` plus the light
/// (`add_colours`, through `DRAW.scratch`).
pub fn view_light() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::lights::add_colours` (through `light_colour_at`'s).
    check_light_colour()?;
    crate::session::write_bits(
        &LIGHT_STATE.lit_place,
        crate::session::read_bits(&CAR.position),
    );
    light_colour_at()?;
    let light: u32 = LIGHT_WORK.light_colour.bits();
    DRAW.scratch.set_bits(light);
    FRAME_WORK
        .view_light
        .set(add_colours(LIGHT_WORK.view_light_base.bits(), light));
    Ok(())
}
