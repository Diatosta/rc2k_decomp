//! The cars' and lamps' light on the lit place, and the light colour of the view.

use super::*;
use crate::errors::{BadInput, Refused};
use crate::race::owner_course::CourseReads;
use crate::race::trap::Trap;
use crate::race::view_scratch::DrawCtx;
use crate::session::Dword;

/// `race::car_light_modes` (0x455B70): the shown car (`g_shownCar` in the original) = the drawn
/// view's car (`crate::race::owner::race().shared.view.view_cars[g_drawnView]`); every car block's
/// +0x6FC 0; then for each race car with lights (+0x25C) that is drawn (any car in modes 0 and 2,
/// else the shown car only; the shown car not with +0x69C and +0x684 both 0, nor in the mirror,
/// `mirrored`): +0x6FC's nibbles set step by step: 0x21; for the shown car with `view_car_flag_b`,
/// 0x12, then 0x14 unless `view_car_stage` is 1, then 0x11 unless it is 2 or 3; then 0x100 in the
/// third nibble, 0x200 with `crate::race::owner::race().shared.vars.settings_words[3]` (1: the
/// shown car only). Refuses (before anything) a null block among the eight or more than eight cars.
/// Not the original's: the camera flag and stage (`view_car_flag_b`, `view_car_stage`) are the
/// shown car's own (`camera_mode_is_2_or_3`, +0x684 `camera_view`), where the original read what
/// the view drawn before had set from them: one frame late after a camera change (`notes/bugs.md`).
pub fn car_light_modes(
    states: &crate::race::car_draw_state::CarDrawState,
    work: &crate::race::work_store::RaceWork,
    vars: &crate::race::vars_store::RaceVars,
    race_view: &crate::race::view_store::RaceView,
    mirrored: bool,
    view: u32,
) -> Result<(), Refused> {
    let block = |n: u32| work.block_number(n as usize);
    let cars: u32 = race_view.race_cars.get();
    if (0..8).any(|n| !work.has_car_block(n as usize)) {
        return Err(BadInput("a null car block").into());
    }
    if cars > 8 {
        return Err(BadInput("more than eight race cars").into());
    }
    let shown = race_view
        .view_cars
        .get(view as usize)
        .ok_or(BadInput("a drawn view past the two"))?
        .get();
    // (The original kept it in `g_shownCar`, read only here.)
    for n in 0..8 {
        states.state_of(block(n)?)?.set(0);
    }
    // Converted: `rc2k_core::race::lights::{lights_drawn, light_word}`.
    use rc2k_core::race::lights::{light_word, lights_drawn};
    let mode: u32 = vars.run_kind.get();
    for n in 0..cars {
        // The state bits are the view's own; what decides them, the physics'.
        let this_car = crate::race::frame_view::car(block(n)?)?;
        let at = states.state_of(block(n)?)?;
        let me = n == shown;
        let drawn = lights_drawn(
            this_car.driver_kind.get() != 0,
            mode,
            me,
            || this_car.camera_switch_share.bits() == 0 && this_car.camera_view.get() == 0,
            || mirrored,
        );
        if !drawn {
            at.set(0);
            continue;
        }
        let word = light_word(
            at.get(),
            me,
            || this_car.camera_mode_is_2_or_3.get() != 0,
            || this_car.camera_view.get(),
            vars.settings_words[3].get(),
        );
        at.set(word);
    }
    Ok(())
}

/// A view's own lamp `n` of the 16 (0x10 bytes: +0 the point, +4 the strength, +8 the colour;
/// `g_lamps` in the original). One past the 16 refused (the original reads on).
pub(super) fn own_lamp(
    lamps: &crate::race::lamp_work::LampWork,
    n: u32,
) -> Result<&[Dword], BadInput> {
    let n = n as usize;
    lamps
        .lamps
        .get(4 * n..4 * n + 4)
        .ok_or(BadInput("a lamp past the sixteen"))
}

/// `race::add_lamp_entry` (0x4573FA): the lamp's point (`g_lamps` +0) through `g_viewMatrix` (the
/// point into `OBJECT_WORK.place` .. `OBJECT_WORK.place[2]`, then x and y turned there, z rounded
/// into `g_faceDepthKey`, + 0x2000000 and shifted right 10), and a depth entry of that key with
/// `g_lampCount` + 0x3000000 (the original also counted them in `g_lampEntryCount`, read by
/// nothing). The caller checks the room.
pub(super) fn add_lamp_entry(ctx: &DrawCtx, lamp: u32) -> Result<(), BadInput> {
    let DrawCtx { view, .. } = *ctx;
    let point = own_lamp(&view.lamps, lamp)?[0].get();
    let place =
        crate::race::scene_points::point_number(ctx.reads.course(), &ctx.part.points, point)?;
    let turned = &view.object.place;
    for (cell, &v) in turned.iter().zip(&place) {
        cell.set(v);
    }
    let matrix: [u32; 16] = crate::session::read_bits(&view.matrices.view_matrix);
    let out = crate::math::x87::rotate_rounding_z(place.map(|v| v as i32), &matrix);
    turned[0].set(out[0]);
    turned[1].set(out[1]);
    let key = out[2].wrapping_add(0x200_0000) >> 10;
    view.face.depth_key.set(key);
    let second = view.lamps.lamp_count.get().wrapping_add(0x300_0000);
    let n: u32 = view.lists.depth_entry_count.get();
    let at = view.lists.depth_entry(n)?;
    at[0].set(key);
    at[1].set(second);
    view.lists.depth_entry_count.set(n.wrapping_add(1));
    Ok(())
}

/// `race::gather_lamps` (0x454E08): `g_lampCount` 0; each lit face (`g_litFaces`, the count
/// `g_litFaceCount`) whose object (+1) is 0x3B becomes a lamp while there are fewer than 16:
/// `g_lamps` entry (+0 the face's first point, +4 its +0x11 byte, +8 0xFFFFFF) and its depth entry
/// (`add_lamp_entry`. Refuses (before anything) more than 0x100 lit faces, or a depth list without
/// room for 16.
pub fn gather_lamps(ctx: &DrawCtx) -> Result<(), Refused> {
    let contact = ctx.reads.contact;
    let count: u32 = contact.lit_face_count.get();
    if count > 0x100 {
        return Err(BadInput("more than 0x100 lit faces").into());
    }
    if count != 0 && ctx.view.lists.depth_entry_count.get() > SORT_MAX - 16 {
        return Err(BadInput("the depth list has no room for the lamps").into());
    }
    ctx.view.lamps.lamp_count.set(0u32);
    for k in 0..count {
        let face = contact.lit_faces[k as usize].get();
        // Converted: `rc2k_core::race::faces::takes_lamp`.
        let object = face_field::<u8>(ctx.reads.course(), face, 1)? as u32;
        if object != rc2k_core::race::faces::LAMP_OBJECT {
            continue;
        }
        let lamp: u32 = ctx.view.lamps.lamp_count.get();
        if !rc2k_core::race::faces::takes_lamp(object, lamp) {
            continue;
        }
        let at = own_lamp(&ctx.view.lamps, lamp)?;
        at[0].set(face_field(ctx.reads.course(), face, 4)?);
        at[1].set(crate::race::course_places::face_number(ctx.reads.course(), face)?.mark() as u32);
        at[2].set(rc2k_core::race::faces::LAMP_COLOUR);
        add_lamp_entry(ctx, lamp)?;
        ctx.view.lamps.lamp_count.set(lamp + 1);
    }
    Ok(())
}

/// Two colours added byte by byte (`rc2k_core::race::lights::add_colours`), MMX (`mmx`) or
/// not as the game.
pub fn add_colours(a: u32, b: u32, mmx: bool) -> u32 {
    rc2k_core::race::lights::add_colours(a, b, mmx)
}

/// 0x44F691: a car's headlights on the lit place, added to `g_lightColour` (see
/// `light_colour_at`). Called.
pub(super) fn car_light_on_point(
    course: CourseReads<'_>,
    car: &crate::race::frame_view::CarView,
    marks_count: u32,
    light_tables: &crate::scene::light_tables_store::LightTables,
    object_light: &crate::scene::object_light::ObjectLight,
    mmx: bool,
) -> Result<(), BadInput> {
    use crate::math::x87::{int_square_sum_scaled_to_int, rotate_rounding};
    use rc2k_core::race::lights::headlight_index;
    if car.lights_on.get() == 0 {
        return Ok(());
    }
    let place = &object_light.lit_place;
    let d = [
        place[0].get().wrapping_sub(car.position[0].get()),
        place[1].get().wrapping_sub(car.position[1].get()),
        place[2].get().wrapping_sub(car.position[2].get()),
    ];
    let t = rotate_rounding(
        d.map(|v| v as i32),
        &crate::session::read_bits(&car.car_inverse_matrix),
    );
    crate::session::write_bits(&object_light.turned_light, t);
    let k = course.numbers.one_over_65536.get();
    let s1 = int_square_sum_scaled_to_int(
        t[2].wrapping_sub(0x20_0000) as i32,
        t[0].wrapping_add(t[0]) as i32,
        k,
        2,
    );
    // The original worked out a second spread and stored it in `g_scratch`; nothing read it, so it is not kept.
    let index = headlight_index(s1, t[1], marks_count);
    let colour = light_tables.car_light_table[index as usize].get();
    object_light
        .light_colour
        .set(add_colours(colour, object_light.light_colour.get(), mmx));
    Ok(())
}

/// 0x44F601: the cars' headlights on the lit place: with `crate::net::owner::net().vars.session`
/// the chosen car
/// (`crate::net::owner::net().vars.player`) or all with `crate::race::owner::race().shared.vars.settings_words[4]` 1; else the players' cars (all with
/// `crate::race::owner::race().shared.vars.settings_words[4]` 1), only the drawn view's car outside one player and modes 0 and 2.
#[allow(
    clippy::too_many_arguments,
    reason = "the drawing's stores the part reads, as they are lent"
)]
pub(super) fn car_lights_on_point(
    reads: crate::workers::DrawReads<'_>,
    net: &crate::net::owner::Net,
    object_light: &crate::scene::object_light::ObjectLight,
    drawn_view: u32,
    mmx: bool,
) -> Result<(), BadInput> {
    let light_tables = reads.light_tables;
    let blocks = &reads.car_blocks;
    let (race_work, race_vars, race_view) = (reads.work, reads.vars, reads.view);
    let vars = &net.vars;
    let cars: u32 = race_view.race_cars.get();
    let all = race_vars.settings_words[4].get() == 1;
    let one = vars.session.get() != 0;
    for n in 0..cars {
        let trap = Trap::new();
        let lit = rc2k_core::race::lights::headlights_light(
            n,
            one,
            || vars.player.get(),
            all,
            || race_view.players.get(),
            || race_vars.run_kind.get(),
            || {
                race_view.drawn_view_car(drawn_view).unwrap_or_else(|e| {
                    trap.keep(e);
                    0
                })
            },
        );
        trap.finish()?;
        if lit {
            // The marks count is the drawing's own (`race::car_parts`), in the block.
            car_light_on_point(
                reads.course(),
                crate::race::frame_view::car(n as usize)?,
                crate::race::owner_shared::RaceShared::race_car_in(race_work, blocks, n as usize)?
                    .marks_count
                    .get(),
                light_tables,
                object_light,
                mmx,
            )?;
        }
    }
    Ok(())
}

/// 0x44F863: the lamps' light on the lit place: each lamp with a strength (+4) gives ((0x200000 - its approximate distance) x strength) >> 17 (held to
/// 0 .. 0xFF) as a grey, times its colour (+8, `LIGHT_WORK.lamp_tint`) byte by byte, added to
/// `g_lightColour`.
pub(super) fn lamp_lights_on_point(
    course: CourseReads<'_>,
    points: &crate::race::scene_points::OwnPoints,
    lamps: &crate::race::lamp_work::LampWork,
    object_light: &crate::scene::object_light::ObjectLight,
    mmx: bool,
) -> Result<(), BadInput> {
    {
        let count: u32 = lamps.lamp_count.get();
        if count == 0 {
            return Ok(());
        }
        let place = &object_light.lit_place;
        for n in 0..count {
            let lamp = own_lamp(lamps, n)?;
            let strength = lamp[1].get();
            if strength == 0 {
                continue;
            }
            let tint = lamp[2].get();
            object_light.lamp_tint.set(tint);
            let point = lamp[0].get();
            let at = crate::race::scene_points::point_number(course, points, point)?;
            let d: [u32; 3] = std::array::from_fn(|k| at[k].wrapping_sub(place[k].get()));
            let [distance, _, _] = crate::scene::approximate_distance::approximate_distance(d);
            let (grey, _) = rc2k_core::race::lights::lamp_grey(distance, strength);
            let light = add_colours(
                rc2k_core::race::lights::multiply_colours(grey, tint),
                object_light.light_colour.get(),
                mmx,
            );
            object_light.light_colour.set(light);
        }
    }
    Ok(())
}

/// What `light_colour_at` refuses, before anything: no cars (its loop tests at the end),
/// more than eight, a null block it would read, more than 16 lamps.
pub fn check_light_colour(
    work: &crate::race::work_store::RaceWork,
    race_view: &crate::race::view_store::RaceView,
    lamps: &crate::race::lamp_work::LampWork,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::lights::check_light_colour`.
    Ok(rc2k_core::race::lights::check_light_colour(
        race_view.race_cars.get(),
        |n| u32::from(work.has_car_block(n as usize)),
        || lamps.lamp_count.get(),
    )?)
}

/// `race::light_colour_at` (0x4537CE): `g_lightColour` 0, then the cars' headlights (0x44F601) and
/// the lamps (0x44F863) on the lit place (`g_litPlace`, `LIGHT_STATE.lit_place[1]`,
/// `LIGHT_STATE.lit_place[2]`), added up. Refuses as `check_light_colour`.
/// `reads`: the drawing's light tables and cars' blocks.
pub fn light_colour_at(
    reads: crate::workers::DrawReads<'_>,
    net: &crate::net::owner::Net,
    points: &crate::race::scene_points::OwnPoints,
    lamps: &crate::race::lamp_work::LampWork,
    object_light: &crate::scene::object_light::ObjectLight,
    drawn_view: u32,
    mmx: bool,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::lights::{check_light_colour, headlights_light,
    // headlight_index, lamp_grey, add_colours, multiply_colours}` (the turned place, the
    // spreads and the lamps' distances the x87's and the scene's).
    check_light_colour(reads.work, reads.view, lamps)?;
    object_light.light_colour.set(0u32);
    car_lights_on_point(reads, net, object_light, drawn_view, mmx)?;
    lamp_lights_on_point(reads.course(), points, lamps, object_light, mmx)?;
    Ok(())
}

/// `race::view_light` (0x455896): the lit place the view's car's (`ViewScratch::car`, the
/// original's `g_car` +0x134 ..); `light_colour_at`; the view's `ObjectLight::view_light` = the
/// view's car's light colour plus the light (`add_colours`). The original added
/// `LIGHT_WORK.view_light_base`, the light colour of the last car the previous view drew (a slip
/// fixed: the views' order showed in the snowflakes' tint; `wiki/Fixed-bugs.md`).
pub fn view_light(ctx: &DrawCtx<'_>, mmx: bool) -> Result<(), Refused> {
    let (race, view) = (ctx.race, ctx.view);
    let (lamps, object_light, car) = (&view.lamps, &view.light, &view.car);
    let drawn_view = view.drawn_view(race);
    // Converted: `rc2k_core::race::lights::add_colours` (through `light_colour_at`'s).
    check_light_colour(ctx.reads.work, ctx.reads.view, lamps)?;
    crate::session::write_bits(
        &object_light.lit_place,
        crate::session::read_bits(&car.position),
    );
    light_colour_at(
        ctx.reads,
        ctx.net,
        &ctx.part.points,
        lamps,
        object_light,
        drawn_view,
        mmx,
    )?;
    let light: u32 = object_light.light_colour.get();
    object_light
        .view_light
        .set(add_colours(car.light_colour.get(), light, mmx));
    Ok(())
}
