//! The cars' and lamps' light on the lit place, and the light colour of the view.

use super::*;
use crate::errors::{BadInput, Refused};
use crate::race::owner_course::CourseReads;
use crate::race::trap::Trap;
use crate::race::view_scratch::DrawCtx;

/// Sets the cars' light modes: the shown car (`shown_car`) is the drawn view's car
/// (`crate::test_support::own::race().shared.view.view_cars[drawn_view]`); every car's state bits
/// (the view's `CarDrawState`) are set to 0; then for each race car with lights (`driver_kind`)
/// that is drawn (any car in modes 0 and 2, else the shown car only; the shown car not with
/// `camera_switch_share` and `camera_view` both 0, nor in the mirror, `mirrored`) the state bits'
/// nibbles are set step by step: 0x21; for the shown car with `view_car_flag_b`, 0x12, then 0x14
/// unless `view_car_stage` is 1, then 0x11 unless it is 2 or 3; then 0x100 in the third nibble,
/// 0x200 with `crate::test_support::own::race().shared.vars.settings_words[3]` (1: the shown car
/// only). Refused (before anything): a null block among the eight, or more than eight cars. The
/// camera flag and stage (`view_car_flag_b`, `view_car_stage`) are the shown car's own
/// (`camera_mode_is_2_or_3`, `camera_view`), not what the view drawn before had set from them
/// (which was one frame late after a camera change).
pub fn car_light_modes(
    frame_cars: &crate::race::frame_view::FrameCars,
    states: &crate::race::car_draw_state::CarDrawState,
    work: &crate::race::work_store::RaceWork,
    vars: crate::race::vars_store::DrawnVarsRef<'_>,
    race_view: crate::race::view_store::DrawnViewRef<'_>,
    mirrored: bool,
    view: u32,
) -> Result<(), Refused> {
    let block = |n: u32| work.block_number(n as usize);
    let cars: u32 = race_view.race_cars();
    if (0..8).any(|n| !work.has_car_block(n as usize)) {
        return Err(BadInput("a null car block").into());
    }
    if cars > 8 {
        return Err(BadInput("more than eight race cars").into());
    }
    let shown = race_view.view_car_number(view as usize)?;
    for n in 0..8 {
        let car_block = block(n)?;
        states.borrow_mut().set_state(car_block, 0)?;
    }
    use rc2k_core::race::lights::{light_word, lights_drawn};
    let mode: u32 = vars.run_kind();
    for n in 0..cars {
        // The state bits are the view's own; what decides them, the physics'.
        let this_car = frame_cars.car(block(n)?)?;
        let car_block = block(n)?;
        let at = states.borrow().state_of(car_block)?;
        let me = n == shown;
        let drawn = lights_drawn(
            this_car.driver_kind.get() != 0,
            mode,
            me,
            || this_car.camera_switch_share.get().to_bits() == 0 && this_car.camera_view.get() == 0,
            || mirrored,
        );
        if !drawn {
            states.borrow_mut().set_state(car_block, 0)?;
            continue;
        }
        let word = light_word(
            at,
            me,
            || this_car.camera_mode_is_2_or_3.get() != 0,
            || this_car.camera_view.get(),
            vars.settings_word(3),
        );
        states.borrow_mut().set_state(car_block, word)?;
    }
    Ok(())
}

/// A view's own lamp `n` of the 16 (the point, the strength, the colour, and a word never used;
/// `lamps`). One past the 16 is refused.
pub(super) fn own_lamp(
    lamps: &crate::race::lamp_work::LampWork,
    n: u32,
) -> Result<[u32; 4], BadInput> {
    lamps
        .borrow()
        .lamps
        .get(n as usize)
        .copied()
        .ok_or(BadInput("a lamp past the sixteen"))
}

/// Adds the lamp entry: the lamp's point (the first of its `lamps` entry) through `view_matrix`
/// (the point into `place` .. `place[2]`, then x and y turned there, z rounded into
/// `face_depth_key`, + 0x2000000 and shifted right 10), and a depth entry of that key with
/// `lamp_count` + 0x3000000. The caller checks the room.
pub(super) fn add_lamp_entry(ctx: &DrawCtx, lamp: u32) -> Result<(), BadInput> {
    let DrawCtx { view, .. } = *ctx;
    let point = own_lamp(&view.lamps, lamp)?[0];
    let place =
        crate::race::scene_points::point_number(ctx.reads.course(), &ctx.part.points, point)?;
    view.object.borrow_mut().place = place;
    let matrix: [u32; 16] = view.matrices.borrow().view_matrix.map(f32::to_bits);
    let out = crate::math::x87::rotate_rounding_z(place.map(|v| v as i32), &matrix);
    view.object.borrow_mut().place[0] = out[0];
    view.object.borrow_mut().place[1] = out[1];
    let key = out[2].wrapping_add(0x200_0000) >> 10;
    view.face.borrow_mut().depth_key = key;
    let second = view.lamps.borrow().lamp_count.wrapping_add(0x300_0000);
    let n: u32 = view.lists.depth_entry_count.get();
    let at = view.lists.depth_entry(n)?;
    at[0].set(key);
    at[1].set(second);
    view.lists.depth_entry_count.set(n.wrapping_add(1));
    Ok(())
}

/// Sets `lamp_count` 0; each lit face (`faces`, the count `count`) whose object is
/// 0x3B becomes a lamp while there are fewer than 16: a `lamps` entry (the face's first point, its
/// mark byte, the colour) and its depth entry (`add_lamp_entry`). Refused (before anything): more
/// than 0x100 lit faces, or a depth list without room for 16.
pub fn gather_lamps(ctx: &DrawCtx) -> Result<(), Refused> {
    let lit = ctx.reads.lit_faces;
    let count: u32 = lit.count();
    if count > 0x100 {
        return Err(BadInput("more than 0x100 lit faces").into());
    }
    if count != 0 && ctx.view.lists.depth_entry_count.get() > SORT_MAX - 16 {
        return Err(BadInput("the depth list has no room for the lamps").into());
    }
    ctx.view.lamps.borrow_mut().lamp_count = 0;
    for k in 0..count {
        let face = lit.face(k as usize);
        let object = face_field::<u8>(ctx.reads.course(), face, 1)? as u32;
        if object != rc2k_core::race::faces::LAMP_OBJECT {
            continue;
        }
        let lamp: u32 = ctx.view.lamps.borrow().lamp_count;
        if !rc2k_core::race::faces::takes_lamp(object, lamp) {
            continue;
        }
        own_lamp(&ctx.view.lamps, lamp)?;
        let point = face_field(ctx.reads.course(), face, 4)?;
        ctx.view.lamps.borrow_mut().lamps[lamp as usize][0] = point;
        let mark = crate::race::course_places::face_number(ctx.reads.course(), face)?.mark() as u32;
        ctx.view.lamps.borrow_mut().lamps[lamp as usize][1] = mark;
        ctx.view.lamps.borrow_mut().lamps[lamp as usize][2] = rc2k_core::race::faces::LAMP_COLOUR;
        add_lamp_entry(ctx, lamp)?;
        ctx.view.lamps.borrow_mut().lamp_count = lamp + 1;
    }
    Ok(())
}

/// Two colours added byte by byte (`rc2k_core::race::lights::add_colours`).
pub fn add_colours(a: u32, b: u32) -> u32 {
    rc2k_core::race::lights::add_colours(a, b)
}

/// A car's headlights on the lit place, added to `light_colour` (see `light_colour_at`).
pub(super) fn car_light_on_point(
    course: CourseReads<'_>,
    car: &crate::race::frame_view::CarView,
    marks_count: u32,
    light_tables: crate::scene::light_tables_store::LightTablesRef<'_>,
    object_light: &crate::scene::object_light::ObjectLight,
) -> Result<(), BadInput> {
    use crate::math::x87::{int_square_sum_scaled_to_int, rotate_rounding};
    use rc2k_core::race::lights::headlight_index;
    if car.lights_on.get() == 0 {
        return Ok(());
    }
    let place = object_light.borrow().lit_place;
    let d = [
        place[0].wrapping_sub(car.position[0].get()),
        place[1].wrapping_sub(car.position[1].get()),
        place[2].wrapping_sub(car.position[2].get()),
    ];
    let t = rotate_rounding(
        d.map(|v| v as i32),
        &crate::memory::session::read_bits(&car.car_inverse_matrix),
    );
    object_light.borrow_mut().turned_light = t;
    let k = course.numbers.one_over_65536.get();
    let s1 = int_square_sum_scaled_to_int(
        t[2].wrapping_sub(0x20_0000) as i32,
        t[0].wrapping_add(t[0]) as i32,
        k,
        2,
    );
    // A second spread worked out here would only be stored in `scratch`; nothing reads it, so it is
    // not kept.
    let index = headlight_index(s1, t[1], marks_count);
    let colour = light_tables.car_light(index as usize);
    {
        let value = add_colours(colour, object_light.borrow().light_colour);
        object_light.borrow_mut().light_colour = value;
    };
    Ok(())
}

/// The cars' headlights on the lit place: in a network game the chosen car or all (when
/// `settings_words[4]` is 1), and the same in a replay of a network race, where the chosen car
/// is the drawn view's; else the players' cars (all with `settings_words[4]` 1), only the
/// drawn view's car outside one player and modes 0 and 2.
#[allow(
    clippy::too_many_arguments,
    reason = "the drawing's stores the part reads, as they are lent"
)]
pub(super) fn car_lights_on_point(
    reads: crate::scene::workers::DrawReads<'_>,
    net_race: Option<rc2k_core::net::net_race::NetRace>,
    object_light: &crate::scene::object_light::ObjectLight,
    drawn_view: u32,
) -> Result<(), BadInput> {
    let light_tables = reads.light_tables;
    let blocks = reads.car_blocks;
    let (race_work, race_vars, race_view) = (reads.work, reads.vars, reads.view);
    let cars: u32 = race_view.race_cars();
    let all = race_vars.settings_word(4) == 1;
    let one = net_race.is_some() || race_view.lights_follow_view() != 0;
    for n in 0..cars {
        let trap = Trap::new();
        let view_car = || {
            race_view.drawn_view_car(drawn_view).unwrap_or_else(|e| {
                trap.keep(e);
                0
            })
        };
        let lit = rc2k_core::race::lights::headlights_light(
            n,
            one,
            || net_race.map_or_else(view_car, |n| n.player),
            all,
            || race_view.players(),
            || race_vars.run_kind(),
            view_car,
        );
        trap.finish()?;
        if lit {
            // The marks count is the drawing's own (`race::car_parts`).
            car_light_on_point(
                reads.course(),
                reads.frame_cars.car(n as usize)?,
                blocks.read(|blocks| {
                    Ok::<_, BadInput>(
                        blocks
                            .parts_of(
                                reads.car_parts,
                                crate::race::owner_shared::RaceShared::race_car_in(
                                    &race_work.borrow(),
                                    blocks,
                                    n as usize,
                                )?,
                            )
                            .marks_count,
                    )
                })?,
                light_tables,
                object_light,
            )?;
        }
    }
    Ok(())
}

/// The lamps' light on the lit place: each lamp with a strength gives ((0x200000 - its approximate
/// distance) x strength) >> 17 (held to 0 .. 0xFF) as a grey, times its colour (`lamp_tint`) byte
/// by byte, added to `light_colour`.
pub(super) fn lamp_lights_on_point(
    course: CourseReads<'_>,
    points: &crate::race::scene_points::OwnPoints,
    lamps: &crate::race::lamp_work::LampWork,
    object_light: &crate::scene::object_light::ObjectLight,
) -> Result<(), BadInput> {
    {
        let count: u32 = lamps.borrow().lamp_count;
        if count == 0 {
            return Ok(());
        }
        let place = object_light.borrow().lit_place;
        for n in 0..count {
            let lamp = own_lamp(lamps, n)?;
            let strength = lamp[1];
            if strength == 0 {
                continue;
            }
            let tint = lamp[2];
            object_light.borrow_mut().lamp_tint = tint;
            let point = lamp[0];
            let at = crate::race::scene_points::point_number(course, points, point)?;
            let d: [u32; 3] = std::array::from_fn(|k| at[k].wrapping_sub(place[k]));
            let [distance, _, _] = crate::scene::approximate_distance::approximate_distance(d);
            let (grey, _) = rc2k_core::race::lights::lamp_grey(distance, strength);
            let light = add_colours(
                rc2k_core::race::lights::multiply_colours(grey, tint),
                object_light.borrow().light_colour,
            );
            object_light.borrow_mut().light_colour = light;
        }
    }
    Ok(())
}

/// What `light_colour_at` refuses, before anything: no cars (its loop tests at the end), more than
/// eight, a null block it would read, more than 16 lamps.
pub fn check_light_colour(
    work: &crate::race::work_store::RaceWork,
    race_view: crate::race::view_store::DrawnViewRef<'_>,
    lamps: &crate::race::lamp_work::LampWork,
) -> Result<(), Refused> {
    Ok(rc2k_core::race::lights::check_light_colour(
        race_view.race_cars(),
        |n| u32::from(work.has_car_block(n as usize)),
        || lamps.borrow().lamp_count,
    )?)
}

/// Sets `light_colour` 0, then adds up the cars' headlights and the lamps on the lit place
/// (`lit_place`, `lit_place[1]`, `lit_place[2]`). Refused as
/// `check_light_colour`. `reads`: the drawing's light tables and cars' blocks.
pub fn light_colour_at(
    reads: crate::scene::workers::DrawReads<'_>,
    net_race: Option<rc2k_core::net::net_race::NetRace>,
    points: &crate::race::scene_points::OwnPoints,
    lamps: &crate::race::lamp_work::LampWork,
    object_light: &crate::scene::object_light::ObjectLight,
    drawn_view: u32,
) -> Result<(), Refused> {
    // The turned place, the spreads and the lamps' distances are the x87's and the scene's.
    check_light_colour(&reads.work.borrow(), reads.view, lamps)?;
    object_light.borrow_mut().light_colour = 0u32;
    car_lights_on_point(reads, net_race, object_light, drawn_view)?;
    lamp_lights_on_point(reads.course(), points, lamps, object_light)?;
    Ok(())
}

/// Sets the lit place to the view's car's (`ViewScratch::car`, its `position`); `light_colour_at`;
/// the view's `ObjectLight::view_light` = the view's car's light colour plus the light
/// (`add_colours`). `view_light_base`, the light colour of the last car the previous view drew, is
/// not added, so the views' order does not show in the snowflakes' tint.
pub fn view_light(ctx: &DrawCtx<'_>) -> Result<(), Refused> {
    let (race, view) = (ctx.race, ctx.view);
    let (lamps, object_light, car) = (&view.lamps, &view.light, &view.car);
    let drawn_view = view.drawn_view(race);
    check_light_colour(&ctx.reads.work.borrow(), ctx.reads.view, lamps)?;
    let car_place: [u32; 3] = car.borrow().position.map(f32::to_bits);
    object_light.borrow_mut().lit_place = car_place;
    light_colour_at(
        ctx.reads,
        ctx.reads.net_race.get(),
        &ctx.part.points,
        lamps,
        object_light,
        drawn_view,
    )?;
    let light: u32 = object_light.borrow().light_colour;
    object_light.borrow_mut().view_light = add_colours(car.borrow().light_colour, light);
    Ok(())
}
