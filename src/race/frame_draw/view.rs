//! The view: draw distance, fog, the view's place and matrices, and the sky.

use super::*;
use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::view_state::ViewState;
use crate::scene::draw_described_object::Description;
use crate::scene::engine_tables_store::ENGINE_TABLES;
use crate::scene::view_matrices::ViewMatrices;

/// `race::draw_distance_settings` (0x43932A): the draw distance's level (`race().course.stage_prefs.draw_distance_level`,
/// held to 0 ..= 4 by sign) picks from six tables: `race().course.stage_prefs.grid_reach` (at least 1, by sign),
/// `race().course.stage_prefs.fog_end`, `race().course.stage_prefs.fog_start`, `race().course.stage_prefs.fog_low`, `race().course.stage_prefs.fog_high`, `race().course.stage_prefs.nearer`.
pub fn draw_distance_settings(course: &Course) {
    // Converted: `rc2k_core::race::view::{draw_distance_level, grid_reach}`.
    use rc2k_core::race::view::{draw_distance_level, grid_reach};
    let level = draw_distance_level(course.stage_prefs.draw_distance_level.get());
    let level = level as usize;
    course
        .stage_prefs
        .grid_reach
        .set(grid_reach(course.stage_prefs.reach_by_level[level].get()));
    course
        .stage_prefs
        .fog_end
        .set(course.stage_prefs.fog_end_by_level[level].get());
    course
        .stage_prefs
        .fog_start
        .set(course.stage_prefs.fog_start_by_level[level].get());
    course
        .stage_prefs
        .fog_low
        .set(course.stage_prefs.fog_low_by_level[level].get());
    course
        .stage_prefs
        .fog_high
        .set(course.stage_prefs.fog_high_by_level[level].get());
    course
        .stage_prefs
        .nearer
        .set(course.stage_prefs.nearer_by_level[level].get());
}

/// `race::sample_noise` (0x44FD38): the 64 x 64 byte map `map` bytes into `g_noiseBytes`
/// sampled at `car`'s camera
/// place (`g_car` +0x6A4 and +0x6AC: the top byte the cell, the next two the fraction):
/// the four corners blended along x and then along z, each step `imul` and `shrd` by 16;
/// the result shifted left 8 is returned (the original kept its steps in
/// `LIGHT_STATE.noise_samples`, read by nothing else).
pub fn sample_noise(car: &crate::physics::car_store::Car, map: usize) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::race::view::sample_noise`.
    let (px, pz) = (car.camera_position[0].get(), car.camera_position[2].get());
    let refused = std::cell::Cell::new(None);
    let noise = &ENGINE_TABLES.noise_bytes;
    let s = rc2k_core::race::view::sample_noise(
        |n| match noise.read_at(map + n as usize, 1, "a noise map") {
            Ok(b) => b[0],
            Err(e) => {
                refused.set(refused.take().or(Some(e)));
                0
            }
        },
        px,
        pz,
    );
    if let Some(e) = refused.take() {
        return Err(e);
    }
    Ok(s.value)
}

/// `race::fog_table` (0x44FC33): the noise at the car
/// (`sample_noise` of `g_noiseBytes`) spread between `race().course.stage_prefs.fog_low` and
/// `race().course.stage_prefs.fog_high` into a step; the near and far values = (`race().frame.frame_work.sun_size` x
/// the view's value, `ViewSun::view_out`, >> 4) x `race().course.stage_prefs.fog_near` / `race().course.stage_prefs.fog_far` >> 8 (the far one to
/// `fog.far`); `fog.length` = `RACE_VIEW.view_depth[1]` x 8 bytes of the table, each ((sum
/// of steps) >> 16) + `race().course.stage_prefs.fog_start` + the near value held to 0 ..= 0xFF; `fog.next`
/// the next one's value with `race().course.stage_prefs.fog_end` (held) plus the near value (held). The
/// original kept them in the light state (`scene::fog`), and also stored the step
/// (`fog_step`) and the near value (`fog_near`), which nothing read. Refuses (before
/// anything) a table length of 0 or past 0x1000.
pub fn fog_table(
    race: &Race,
    fog: &crate::scene::fog::Fog,
    car: &crate::physics::car_store::Car,
    sun: &crate::scene::polygon_work::ViewSun,
    window: &crate::scene::view_window::ViewWindow,
) -> Result<(), Refused> {
    let length = window.view_depth[1].get().wrapping_shl(3);
    if length == 0 || length > 0x1000 {
        return Err(BadInput("a fog table length of 0 or past 0x1000").into());
    }
    // Converted: `rc2k_core::race::view::fog_table`.
    let noise = sample_noise(car, 0)?;
    let set = rc2k_core::race::view::fog_table(
        noise,
        &rc2k_core::race::view::FogSettings {
            low: race.course.stage_prefs.fog_low.get(),
            high: race.course.stage_prefs.fog_high.get(),
            scale: [sun.size.get(), sun.view_out.get()],
            near: race.course.stage_prefs.fog_near.get(),
            far: race.course.stage_prefs.fog_far.get(),
            start: race.course.stage_prefs.fog_start.get(),
            end: race.course.stage_prefs.fog_end.get(),
            length,
        },
    );
    fog.far.set(set.far);
    fog.length.set(length);
    // The table's bytes (any length up to 0x1000; the rest kept).
    fog.set_levels(&set.table);
    fog.next.set(set.next);
    Ok(())
}

/// `race::set_up_fog` (0x44FBD9): `DRAW.fog_colour` = the fog colour
/// from `race().course.stage_prefs.fog_colour` (red), `race().course.stage_prefs.fog_colour[1]`, `race().course.stage_prefs.fog_colour[2]` (each shifted into its byte
/// and added); `fog_table` into the view's `fog`; the colour with 0xFF000000 to the device.
/// None of it reads a register at entry, and its callers (`race::draw_view_scene`,
/// `race::mirror_view_window`) read none it leaves. Refuses (before anything) what
/// `fog_table` would.
pub fn set_up_fog(
    race: &Race,
    fog: &crate::scene::fog::Fog,
    poly: &crate::scene::polygon_work::PolygonWork,
    car: &crate::physics::car_store::Car,
) -> Result<(), Refused> {
    let course = &race.course;
    let length = poly.window.view_depth[1].get().wrapping_shl(3);
    if length == 0 || length > 0x1000 {
        return Err(BadInput("a fog table length of 0 or past 0x1000").into());
    }
    // Converted: `rc2k_core::race::view::{fog_colour, device_fog_colour}`.
    let colour = rc2k_core::race::view::fog_colour(
        course.stage_prefs.fog_colour[0].get(),
        course.stage_prefs.fog_colour[1].get(),
        course.stage_prefs.fog_colour[2].get(),
    );
    fog.colour.set(colour);
    fog_table(race, fog, car, &poly.sun, &poly.window)?;
    crate::scene::set_fog_colour::set_fog_colour(
        rc2k_core::race::view::device_fog_colour(fog.colour.get()),
        poly,
    );
    Ok(())
}

/// `race::place_view` (0x4389A8): the view's place from the camera (`race().course.course.place_x`,
/// `race().course.course.place_y`, `race().course.course.place_z`) and its angles. Flat (`race().course.stage_prefs.flat_depth` set): the place as
/// it is, `g_car` +0x6A8 += `race().course.course.flat_depth`, `g_viewAngles` = `race().course.course.place_heading`,
/// `race().frame.view_and_records.view_vars.view_angles[1]` = 0x40000000. Else the place moved by (-`race().course.course.flat_depth`, 0, 0) turned by
/// `race().course.course.camera_first` (`math::rotate_point_about_y_ecx`) and that turned again by `race().course.course.place_heading`
/// (`math::rotate_point_about_y`, its x 0, y the first turn's z, z its x), `g_viewAngles` and
/// `race().frame.view_and_records.view_vars.view_angles[1]` the two angles, and the place also as whole numbers and floats
/// (`race().frame.view_and_records.view_vars.view_x` / `race().frame.view_and_records.view_vars.view_z`, `race().frame.view_and_records.view_vars.view_place_f` ..). `race().frame.view_and_records.view_vars.view_angles[2]` .. `race().frame.view_and_records.view_vars.view_angles[5]` 0
/// both ways.
pub fn place_view(
    race: &Race,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), BadInput> {
    use crate::math::x87::from_int;
    let place = &view.vars.view_place;
    let own = &view.state;
    place[0].set(race.course.course.place_x.get());
    place[1].set(race.course.course.place_y.get());
    place[2].set(race.course.course.place_z.get());
    own.view_place[0].set(race.course.course.place_x.get());
    own.view_place[1].set(race.course.course.place_y.get());
    own.view_place[2].set(race.course.course.place_z.get());
    let zero = || {
        for a in [2, 4, 3, 5] {
            view.vars.view_angles[a].set(0);
            own.view_angles[a].set(0);
        }
    };
    if race.course.stage_prefs.flat_depth.get() != 0 {
        let at = &view.car.camera_position[1];
        at.set(at.get().wrapping_add(race.course.course.flat_depth.get()));
        view.vars.view_angles[0].set(race.course.course.place_heading.get());
        view.vars.view_angles[1].set(0x4000_0000u32);
        own.view_angles[0].set(race.course.course.place_heading.get());
        own.view_angles[1].set(0x4000_0000u32);
        zero();
        return Ok(());
    }
    let first: u32 = race.course.course.camera_first.get();
    let second: u32 = race.course.course.place_heading.get();
    let back = (race.course.course.flat_depth.get()).wrapping_neg();
    let [x1, z1, _] =
        crate::math::rotate_point_about_y_ecx::rotate_point_about_y_ecx(back as i32, 0, 0, first);
    let [x2, _, z2, _] =
        crate::math::rotate_point_about_y::rotate_point_about_y(0, z1 as i32, x1 as i32, second);
    place[0].set(place[0].get().wrapping_add(x2));
    place[1].set(place[1].get().wrapping_add(z1));
    place[2].set(place[2].get().wrapping_add(z2));
    own.view_place[0].set(own.view_place[0].get().wrapping_add(x2));
    own.view_place[1].set(own.view_place[1].get().wrapping_add(z1));
    own.view_place[2].set(own.view_place[2].get().wrapping_add(z2));
    view.vars.view_angles[0].set(second);
    view.vars.view_angles[1].set(first);
    own.view_angles[0].set(second);
    own.view_angles[1].set(first);
    zero();
    view.vars.view_x.set(place[0].get());
    view.vars.view_place_f[0].set(from_int(place[0].get() as i32));
    view.vars.view_place_f[1].set(from_int(place[1].get() as i32));
    view.vars.view_z.set(place[2].get());
    view.vars.view_place_f[2].set(from_int(place[2].get() as i32));
    Ok(())
}

/// `race::sky_matrix` (0x42725E): `g_madeObjectMatrix` for the sky: a turn about y by
/// -(`race().course.stage_prefs.sky_turn` << 8), placed at (0, `race().course.stage_prefs.sky_height` << 8, 0) (into `OBJECT_WORK.place` ..),
/// the copy with the place (`g_turnedMatrix`) scaled by (`DRAW.farthest_scaled` << 4) / 16
/// (`OBJECT_WORK.size`), times `g_viewTurnMatrix`.
pub fn sky_matrix(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), BadInput> {
    let race = ctx.race;
    use crate::scene::matrix;
    let angle = (race.course.stage_prefs.sky_turn.get() << 8).wrapping_neg();
    let m = &ctx.view.matrices;
    matrix::set_turn_y(&m.turn_y_matrix, &m.turn_y_work, angle);
    let height = race.course.stage_prefs.sky_height.get() << 8;
    ctx.view.object.place[0].set(0u32);
    ctx.view.object.place[2].set(0u32);
    ctx.view.object.place[1].set(height);
    matrix::set_place(&m.placed_matrix, [0, height, 0]);
    matrix::copy_matrix_with_place(&m.turn_y_matrix, &m.placed_matrix, &m.turned_matrix);
    let size = ctx.view.polygon.window.farthest_scaled.get() << 4;
    ctx.view.object.size.set(size);
    let scale = crate::math::x87::int_over_16(size as i32);
    for cell in &m.turned_matrix {
        cell.set(crate::math::x87::mul(cell, scale));
    }
    matrix::multiply_matrices(&m.turned_matrix, &m.view_turn_matrix, &m.made_object_matrix);
    Ok(())
}

/// The end of `view_matrices` (0x4388E5 .. and 0x438950 ..): the place negated into
/// `g_placedMatrix` (kept also at 0x5EA1CC), times the view's turn into `g_viewMatrix`,
/// the turn undone into `g_viewBackMatrix` (the view's own).
fn view_place_tail(course: &Course, m: &ViewMatrices, state: &ViewState) -> Result<(), BadInput> {
    use crate::scene::{matrix, view_matrix};
    let p = crate::session::read_bits(&state.view_place).map(u32::wrapping_neg);
    matrix::set_place(&m.placed_matrix, p);
    crate::math::cells::copy_cells(&m.placed_matrix, &m.view_place_matrix, 16);
    matrix::multiply_matrices(&m.placed_matrix, &m.view_turn_matrix, &m.view_matrix);
    view_matrix::invert_turn(course, &m.view_turn_matrix, &m.view_back_matrix);
    Ok(())
}

/// `race::view_matrices` (0x4386A4): the view's matrices for the frame. With
/// `GROUND_WORK.view_mirrored` set: `g_viewTurnMatrix` the kept turn (`race().frame.matrices.view_turn_copy`). Else for the car's
/// views 0 .. 3 (`g_car` +0x684) the car's turn undone (`g_crashMatrix`); for the others
/// the identity turned by the view's angles (fine y, fine x, z of `g_viewAngles`,
/// `race().frame.view_and_records.view_vars.view_angles[1]`, `race().frame.view_and_records.view_vars.view_angles[2]`, each negated; then again by `race().frame.view_and_records.view_vars.view_angles[3]` ..
/// `race().frame.view_and_records.view_vars.view_angles[5]`), each step `turn y`, `turn x`, `turn z` multiplied in. Views 1 and 2
/// then roll with the car (`g_car` +0x648 by its speed, eased into `g_car` +0x6A0, turned
/// about z). Then the place (see `view_place_tail`).
pub fn view_matrices(
    race: &Race,
    m: &ViewMatrices,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), BadInput> {
    // (Handed on to what reads them outside the drawing, the listener and the car sounds,
    // on the game's thread in the views' order: `hand_on_view_matrices`.)
    view_own_matrices(
        race,
        m,
        &view.state,
        &view.car,
        view.polygon.view_mirrored.get() != 0,
    )
}

/// A drawn view's matrices into the shared ones (`race().frame.matrices`), read outside the
/// drawing (the listener before any view, the car sounds before the first view's turn is
/// kept) as the original's were: the game's thread, once the view is drawn, in the views'
/// order (`race::draw_race_scene`'s `view_drawn`; no drawing part reads them).
pub fn hand_on_view_matrices(race: &Race, m: &ViewMatrices) {
    let shared = &race.frame.matrices;
    for (own, to) in [
        (&m.view_turn_matrix, &shared.view_turn_matrix),
        (&m.view_place_matrix, &shared.view_place_matrix),
        (&m.view_matrix, &shared.view_matrix),
        (&m.view_back_matrix, &shared.view_back_matrix),
    ] {
        crate::math::cells::copy_cells(own, to, 16);
    }
}

/// `view_matrices` into the view's own matrices.
fn view_own_matrices(
    race: &Race,
    m: &ViewMatrices,
    state: &ViewState,
    car: &crate::physics::car_store::Car,
    mirrored: bool,
) -> Result<(), BadInput> {
    use crate::scene::matrix::multiply_matrices;
    use crate::scene::view_matrix::{Axis, invert_turn, set_fine_turn};
    let (view_turn, turn_y, turn_x, turn_z) = (
        &m.view_turn_matrix,
        &m.turn_y_matrix,
        &m.turn_x_matrix,
        &m.turn_z_matrix,
    );
    let (turned, lit) = (&m.turned_matrix, &m.lit_matrix);
    if mirrored {
        crate::math::cells::copy_cells(&m.view_turn_copy, view_turn, 16);
        return view_place_tail(&race.course, m, state);
    }
    let view: u32 = car.camera_view.get();
    if view <= 3 {
        invert_turn(&race.course, &car.crash_matrix, view_turn);
    } else {
        crate::math::cells::copy_cells(&race.frame.matrices.identity_matrix, &view_turn, 16);
        for angles in [[0usize, 1, 2], [3, 4, 5]] {
            set_fine_turn(
                &race.course,
                m,
                Axis::Y,
                state.view_angles[angles[0]].get().wrapping_neg(),
            );
            set_fine_turn(
                &race.course,
                m,
                Axis::X,
                state.view_angles[angles[1]].get().wrapping_neg(),
            );
            set_fine_turn(
                &race.course,
                m,
                Axis::Z,
                state.view_angles[angles[2]].get().wrapping_neg(),
            );
            multiply_matrices(view_turn, turn_y, turned);
            multiply_matrices(turned, turn_x, lit);
            multiply_matrices(lit, turn_z, view_turn);
        }
    }
    if view == 1 || view == 2 {
        set_fine_turn(&race.course, m, Axis::Z, rolled_angle(race, car));
        multiply_matrices(view_turn, turn_z, turned);
        crate::math::cells::copy_cells(&turned, &view_turn, 16);
    }
    view_place_tail(&race.course, m, state)
}

/// The angle views 1 and 2 roll by: the car's speed and steering eased into the camera roll.
fn rolled_angle(race: &Race, car: &crate::physics::car_store::Car) -> u32 {
    use crate::math::x87::{below_or_unordered, eased_roll};
    let mut v = crate::math::x87::div(&car.local_velocity[2], &race.course.numbers.ten).to_bits();
    if below_or_unordered(v, race.course.numbers.zero.bits()) {
        v = race.course.numbers.zero.bits();
    }
    if !below_or_unordered(v, race.course.numbers.one.bits()) {
        v = race.course.numbers.one.bits();
    }
    let roll = eased_roll(
        car.steering_raw.get() as i32,
        race.course.numbers.k65536.get(),
        race.course.numbers.one_over_two_pi.get(),
        v,
        race.course.numbers.tenth.get(),
        car.camera_roll.get(),
        race.course.numbers.half.get(),
    );
    car.camera_roll.set(roll);
    (crate::math::x87::mul3_to_int(
        roll,
        race.course.numbers.k65536.get(),
        race.course.numbers.k4096.get(),
        race.course.numbers.one_over_two_pi.get(),
    ) as u32)
        << 4
}

/// `race::camera_view` (0x438325): the view for the frame: `race().frame.view_and_records.view_vars.view_car_flag_b`,
/// `race().frame.view_and_records.view_vars.view_car_flag_a`, `race().frame.view_and_records.view_vars.view_car_stage` from `g_car` +0x6F0, +0x69C, +0x684;
/// then without `FRAME_SWITCHES.scene_set_up` `race::place_view`; else the place from the car's
/// camera (`g_car` +0x6A4 .., the angles from +0x6B0 .., `race().frame.view_and_records.view_vars.view_scale` from +0x6C8)
/// or, with `GROUND_WORK.view_mirrored`, from the crash (the kept turn made from
/// `race().frame.matrices.view_work` and `g_crashMatrix`, the place the car's raised by 0x8000), written as
/// whole numbers and floats (also to `g_car` +0x734 ..), and `scene::set_view_window`; then
/// `view_matrices`.
pub fn camera_view(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
    m: &ViewMatrices,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), BadInput> {
    use crate::math::x87::from_int;
    let own = &view.state;
    // The place and angles as they are (the camera's, or what the view before left: the
    // mirror's way sets neither, and its objects turn by the main view's angles).
    let vars = &view.vars;
    crate::math::cells::copy_cells(&vars.view_place, &own.view_place, 3);
    crate::math::cells::copy_cells(&vars.view_angles, &own.view_angles, 6);
    own.view_car_flag_a.set(view.car.camera_switch_share.bits());
    own.view_car_stage.set(view.car.camera_view.get());
    view.vars
        .view_car_flag_b
        .set(view.car.camera_mode_is_2_or_3.get());
    view.vars
        .view_car_flag_a
        .set(view.car.camera_switch_share.bits());
    view.vars.view_car_stage.set(view.car.camera_view.get());
    if race.shared.switches.scene_set_up.get() == 0 {
        place_view(race, view)?;
        view_matrices(race, m, view)?;
        return Ok(());
    }
    let place: [u32; 3] = if view.polygon.view_mirrored.get() == 0 {
        view.vars.view_scale.set(view.car.camera_zoom.get());
        for (from, to) in [
            (&view.car.camera_heading, 0usize),
            (&view.car.camera_pitch, 1),
            (&view.car.camera_roll_turn, 2),
            (&view.car.camera_offset_turns[1], 3),
            (&view.car.camera_offset_turns[0], 4),
            (&view.car.camera_offset_turns[2], 5),
        ] {
            view.vars.view_angles[to].set(from.get());
            own.view_angles[to].set(from.get());
        }
        crate::session::read_bits(&view.car.camera_position)
    } else {
        crate::scene::matrix::multiply_matrices(
            &race.frame.matrices.view_work,
            &view.car.crash_matrix,
            &m.turned_matrix,
        );
        crate::scene::view_matrix::invert_turn(&race.course, &m.turned_matrix, &m.view_turn_copy);
        let kept = &view.car.kept_focus_point;
        kept[0].set(view.car.position[0].get());
        kept[1].set(view.car.position[1].get().wrapping_add(0x8000));
        kept[2].set(view.car.position[2].get());
        view.vars.view_scale.set(view.vars.view_scale_kept.get());
        [kept[0].get(), kept[1].get(), kept[2].get()]
    };
    for (k, &p) in place.iter().enumerate() {
        view.car.kept_view_place[k].set(p);
        view.vars.view_place[k].set(p);
    }
    view.vars.view_x.set(place[0]);
    view.vars.view_z.set(place[2]);
    for (k, &p) in place.iter().enumerate() {
        own.view_place[k].set(p);
    }
    view.vars.view_place_f[0].set(from_int(place[0] as i32));
    view.vars.view_place_f[1].set(from_int(place[1] as i32));
    view.vars.view_place_f[2].set(from_int(place[2] as i32));
    crate::scene::set_view_window::set_view_window(
        race,
        scene,
        system,
        &view.polygon.window,
        &view.vars,
    );
    view_matrices(race, m, view)?;
    Ok(())
}

/// `race::draw_sky` (0x455540): `race::fetch_view_value`; (the original's `g_polygonFlags` 0 not kept: dead); unless the
/// scene's mode (`LIGHT_STATE.draw_mode`) is 1 or 2: the sky (object 1 through `sky_matrix`,
/// kept at 0x5EA34C, full light, `LIGHT_STATE.view_kind` 4 and `FRAME_SWITCHES.no_far_clip` 1
/// while it is drawn by parts 1 .. 4); then with a sun (`race().course.stage_prefs.fog_near` +
/// `race().course.stage_prefs.fog_far` not 0): its direction from `race().course.stage_prefs.sky_turn` +
/// `race().course.stage_prefs.sun_turn` and `race().course.stage_prefs.sun_height` , turned by
/// the sky's matrix , and when in front (z not below
/// `race().course.numbers.k4096_a`) its place on the screen (`ViewSun::place` / `hit`
/// ..), the view's sun (`ViewSun`) on the screen and its size (256
/// less half its distance from the centre, at least 0).
pub fn draw_sky(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    fetch_view_value(race, &ctx.view.polygon.sun, ctx.view.drawn_view(race))?;
    // Not left from the previous view's last object (the original's slip, fixed:
    // `wiki/Fixed-bugs.md`).
    ctx.view.light.other_light.set(0);
    let mode: u32 = scene.lights.state.draw_mode.get();
    if mode == 1 || mode == 2 {
        return Ok(());
    }
    sky_matrix(ctx)?;
    crate::math::cells::copy_cells(
        &ctx.view.matrices.made_object_matrix,
        &ctx.view.matrices.object_matrix_slots,
        16,
    );
    ctx.view.cars.object_number.set(1);
    ctx.view.light.view_kind.set(4);
    ctx.view.light.light_lowest.set(0xFF);
    ctx.view.light.light_highest.set(0xFF);
    ctx.view.light.light_level.set(0xFF);
    ctx.view.light.light_turns[0].set(0);
    ctx.view.light.light_turns[1].set(0);
    ctx.view.face.no_far_clip.set(1);
    crate::scene::glow_render_states::glow_render_states();
    ctx.view.light.light_kind.set(0u32);
    // The matrix pointer is where the matrix's copy leaves it: past `g_madeObjectMatrix`.
    crate::scene::draw_scene_object::draw_scene_object_in_game(
        ctx,
        Some(Description::Cells(&ctx.view.matrices.object_matrix_slots)),
    )?;
    crate::scene::depth_buffer_on::depth_buffer(true);
    ctx.view.light.view_kind.set(0);
    ctx.view.face.no_far_clip.set(0);
    place_sun(ctx, ctx.system)
}

/// The sun: its direction from the stage's sun angles, turned by the sky's matrix, and when it is
/// in front its place on the screen and its size. Cleared first, and left cleared when both sun settings are 0 or
/// the sun is behind the view.
fn place_sun(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let race = ctx.race;
    let session = &system.session;
    use crate::math::x87::{sun_direction, sun_on_screen, sun_size, tbyte_to_int, to_tbyte};
    let sun = &ctx.view.polygon.sun;
    sun.on_screen.set(0);
    sun.size.set(0);
    if race
        .course
        .stage_prefs
        .fog_near
        .get()
        .wrapping_add(race.course.stage_prefs.fog_far.get())
        == 0
    {
        return Ok(());
    }
    let angle = race
        .course
        .stage_prefs
        .sun_turn
        .get()
        .wrapping_add(race.course.stage_prefs.sky_turn.get())
        & 0xFF;
    let dir = sun_direction(
        angle as i32,
        race.course.numbers.sixty_four.bits(),
        race.frame.frame_work.sun_depth.bits(),
        race.course.stage_prefs.sun_height.get() as i32,
        race.course.numbers.k98304.bits(),
    );
    // (The original kept the direction in `FrameWork.sun_floats` and the turned point in
    // `sun_points`, read by nothing.)
    let turned = crate::scene::rotate_point::rotate_point(
        &ctx.view.matrices.made_object_matrix,
        [to_tbyte(dir[0]), to_tbyte(dir[1]), to_tbyte(dir[2])],
    );
    let place = turned.map(tbyte_to_int);
    let z = crate::math::x87::from_int(place[2] as i32);
    if crate::math::x87::below_or_unordered(z, race.course.numbers.k4096_a.bits()) {
        return Ok(());
    }
    let flat = race.course.stage_prefs.flat_depth.get() != 0;
    let v = [
        place[0],
        place[1],
        place[2],
        race.course.course.flat_depth.get(),
        ctx.view.polygon.window.view_scale.bits(),
        ctx.view.polygon.window.view_centre[1].bits(),
        ctx.view.polygon.window.view_centre[0].bits(),
    ];
    let [fx, ix, fy, iy] = sun_on_screen(&v, flat, &crate::session::Dword::new(0));
    sun.place[0].set(fx);
    sun.hit[0].set(ix);
    sun.place[1].set(fy);
    sun.hit[1].set(iy);
    let on = ix < session.screen.width.get() && iy < session.screen.height.get();
    sun.on_screen.set(u32::from(on));
    let size = sun_size(
        fx,
        ctx.view.polygon.window.view_centre[0].get(),
        fy,
        ctx.view.polygon.window.view_centre[1].get(),
        race.course.numbers.half.get(),
        race.course.numbers.k256.get(),
    );
    sun.size.set(if (size as i32) < 0 { 0 } else { size });
    Ok(())
}

// g_keptReflectionNormals: 0x400 dwords
