//! The view: draw distance, fog, the view's place and matrices, and the sky.

use super::*;
use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::view_state::ViewState;
use crate::scene::draw_described_object::{DescribedMatrix, Description};
use crate::scene::view_matrices::ViewMatrices;

/// `race::draw_distance_settings`: the draw distance's level
/// (`race().course.stage_prefs.draw_distance_level`, held to 0 ..= 4 by sign) picks from six
/// tables: `race().course.stage_prefs.grid_reach` (at least 1, by sign),
/// `race().course.stage_prefs.fog_end`, `race().course.stage_prefs.fog_start`,
/// `race().course.stage_prefs.fog_low`, `race().course.stage_prefs.fog_high`,
/// `race().course.stage_prefs.nearer`.
pub fn draw_distance_settings(course: &Course) {
    use rc2k_core::race::view::{draw_distance_level, grid_reach};
    let level = draw_distance_level(course.stage_prefs.borrow().draw_distance_level);
    let level = level as usize;
    let mut prefs = course.stage_prefs.borrow_mut();
    prefs.grid_reach = grid_reach(prefs.reach_by_level[level]);
    prefs.fog_end = prefs.fog_end_by_level[level];
    prefs.fog_start = prefs.fog_start_by_level[level];
    prefs.fog_low = prefs.fog_low_by_level[level];
    prefs.fog_high = prefs.fog_high_by_level[level];
    prefs.nearer = prefs.nearer_by_level[level];
}

/// Samples the 64 x 64 byte map `map` bytes into `noise_bytes` at `car`'s camera place
/// (`car.camera_position` x and z: the top byte the cell, the next two the fraction): the four
/// corners are blended along x and then along z, each step a product shifted right by 16; the
/// result shifted left 8 is returned.
pub fn sample_noise(
    tables: &crate::math::engine_tables::EngineTables,
    car: &crate::physics::car_store::Car<impl crate::memory::session::CellKind>,
    map: usize,
) -> Result<u32, BadInput> {
    let (px, pz) = (
        car.camera_position.bits_at(0),
        car.camera_position.bits_at(2),
    );
    let refused = std::cell::Cell::new(None);
    let noise = &tables.noise_bytes;
    let s = rc2k_core::race::view::sample_noise(
        |n| match noise.byte_at(map + n as usize, "a noise map") {
            Ok(b) => b,
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

/// `race::fog_table`: the noise at the car (`sample_noise` of `noise_bytes`) spread between
/// `race().course.stage_prefs.fog_low` and `race().course.stage_prefs.fog_high` into a step; the
/// near and far values = (`race().frame.frame_work.sun_size` x the view's value,
/// `ViewSun::view_out`, >> 4) x `race().course.stage_prefs.fog_near` /
/// `race().course.stage_prefs.fog_far` >> 8 (the far one to `fog.far`); `fog.length` =
/// `view_depth[1]` x 8 bytes of the table, each ((sum of steps) >> 16) +
/// `race().course.stage_prefs.fog_start` + the near value held to 0 ..= 0xFF; `fog.next` the next
/// `fog.next` the next one's value with `race().course.stage_prefs.fog_end` (held) plus the near
/// value (held). The step (`fog_step`) and the near value (`fog_near`) are not stored: nothing
/// reads them. Refused (before anything): a table length of 0 or past 0x1000.
pub fn fog_table(
    tables: &crate::math::engine_tables::EngineTables,
    prefs: crate::race::prefs_store::StagePrefsRef<'_>,
    fog: &crate::scene::fog::Fog,
    car: &crate::physics::car_store::ViewCar,
    sun: &crate::scene::polygon_work_store::ViewSun,
    window: &crate::scene::view_window::ViewWindow,
) -> Result<(), Refused> {
    let length = window.borrow().view_depth[1].wrapping_shl(3);
    if length == 0 || length > 0x1000 {
        return Err(BadInput("a fog table length of 0 or past 0x1000").into());
    }
    let noise = sample_noise(tables, car, 0)?;
    let (sun_size, sun_view_out) = {
        let sun = sun.borrow();
        (sun.size, sun.view_out)
    };
    let settings = prefs.read(|p| rc2k_core::race::view::FogSettings {
        low: p.fog_low,
        high: p.fog_high,
        scale: [sun_size, sun_view_out],
        near: p.fog_near,
        far: p.fog_far,
        start: p.fog_start,
        end: p.fog_end,
        length,
    });
    let set = rc2k_core::race::view::fog_table(noise, &settings);
    let mut fog = fog.borrow_mut();
    fog.far = set.far;
    fog.length = length;
    // The table's bytes (any length up to 0x1000; the rest kept).
    fog.set_levels(&set.table);
    fog.next = set.next;
    Ok(())
}

/// `race::set_up_fog`: `fog_colour` = the fog colour from
/// `race().course.stage_prefs.fog_colour` (red), `race().course.stage_prefs.fog_colour[1]`,
/// `race().course.stage_prefs.fog_colour[2]` (each shifted into its byte and added); `fog_table`
/// into the view's `fog`; the colour with 0xFF000000 goes to the device. Refused (before anything):
/// what `fog_table` would refuse.
pub fn set_up_fog(
    tables: &crate::math::engine_tables::EngineTables,
    prefs: crate::race::prefs_store::StagePrefsRef<'_>,
    fog: &crate::scene::fog::Fog,
    poly: &crate::scene::polygon_work_store::PolygonWork,
    car: &crate::physics::car_store::ViewCar,
    part: &crate::scene::scratch_store::PartScratch,
) -> Result<(), Refused> {
    let length = poly.window.borrow().view_depth[1].wrapping_shl(3);
    if length == 0 || length > 0x1000 {
        return Err(BadInput("a fog table length of 0 or past 0x1000").into());
    }
    let [red, green, blue] = prefs.read(|p| p.fog_colour);
    let colour = rc2k_core::race::view::fog_colour(red, green, blue);
    fog.borrow_mut().colour = colour;
    fog_table(tables, prefs, fog, car, &poly.sun, &poly.window)?;
    crate::scene::set_fog_colour::set_fog_colour(
        rc2k_core::race::view::device_fog_colour(fog.borrow().colour),
        poly,
        part,
    );
    Ok(())
}

/// `race::place_view`: the view's place from the camera (`race().course.course.place_x`,
/// `race().course.course.place_y`, `race().course.course.place_z`) and its angles. Flat
/// (`race().course.stage_prefs.flat_depth` set): the place as it is, `car.camera_position` y +=
/// `race().course.course.flat_depth`, `view_angles` = `race().course.course.place_heading`,
/// `race().frame.view_and_records.view_vars.view_angles[1]` = 0x40000000. Else the place moved by
/// (-`race().course.course.flat_depth`, 0, 0) turned by `race().course.course.camera_first`
/// (`math::rotate_point_about_y_ecx`) and that turned again by `race().course.course.place_heading`
/// (`math::rotate_point_about_y`, its x 0, y the first turn's z, z its x), `view_angles` and
/// `race().frame.view_and_records.view_vars.view_angles[1]` the two angles, and the place also as
/// whole numbers and floats (`race().frame.view_and_records.view_vars.view_x` /
/// `race().frame.view_and_records.view_vars.view_z`,
/// `race().frame.view_and_records.view_vars.view_place_f` ..).
/// `race().frame.view_and_records.view_vars.view_angles[2]` ..
/// `race().frame.view_and_records.view_vars.view_angles[5]` 0 both ways.
pub fn place_view(
    tables: &crate::math::engine_tables::EngineTables,
    course: crate::race::owner_course::CourseReads<'_>,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), BadInput> {
    use crate::math::x87::from_int;
    let place = &view.vars.view_place;
    place[0].set(course.course.get().place_x);
    place[1].set(course.course.get().place_y);
    place[2].set(course.course.get().place_z);
    view.state.borrow_mut().view_place = [
        course.course.get().place_x,
        course.course.get().place_y,
        course.course.get().place_z,
    ];
    let zero = || {
        for a in [2, 4, 3, 5] {
            view.vars.view_angles[a].set(0);
            view.state.borrow_mut().view_angles[a] = 0;
        }
    };
    if course.stage_prefs.read(|p| p.flat_depth) != 0 {
        let place = &view.car.camera_position;
        place.set_bits_at(
            1,
            place
                .bits_at(1)
                .wrapping_add(course.course.get().flat_depth),
        );
        view.vars.view_angles[0].set(course.course.get().place_heading);
        view.vars.view_angles[1].set(0x4000_0000u32);
        view.state.borrow_mut().view_angles[0] = course.course.get().place_heading;
        view.state.borrow_mut().view_angles[1] = 0x4000_0000u32;
        zero();
        return Ok(());
    }
    let first: u32 = course.course.get().camera_first;
    let second: u32 = course.course.get().place_heading;
    let back = (course.course.get().flat_depth).wrapping_neg();
    let [x1, z1, _] = crate::math::rotate_point_about_y_ecx::rotate_point_about_y_ecx(
        tables,
        back as i32,
        0,
        0,
        first,
    );
    let [x2, _, z2, _] = crate::math::rotate_point_about_y::rotate_point_about_y(
        tables, 0, z1 as i32, x1 as i32, second,
    );
    place[0].set(place[0].get().wrapping_add(x2));
    place[1].set(place[1].get().wrapping_add(z1));
    place[2].set(place[2].get().wrapping_add(z2));
    {
        let mut own = view.state.borrow_mut();
        own.view_place[0] = own.view_place[0].wrapping_add(x2);
        own.view_place[1] = own.view_place[1].wrapping_add(z1);
        own.view_place[2] = own.view_place[2].wrapping_add(z2);
    }
    view.vars.view_angles[0].set(second);
    view.vars.view_angles[1].set(first);
    {
        let mut own = view.state.borrow_mut();
        own.view_angles[0] = second;
        own.view_angles[1] = first;
    }
    zero();
    view.vars.view_x.set(place[0].get());
    view.vars.view_place_f[0].set(from_int(place[0].get() as i32));
    view.vars.view_place_f[1].set(from_int(place[1].get() as i32));
    view.vars.view_z.set(place[2].get());
    view.vars.view_place_f[2].set(from_int(place[2].get() as i32));
    Ok(())
}

/// `race::sky_matrix`: `made_object_matrix` for the sky: a turn about y by
/// -(`race().course.stage_prefs.sky_turn` << 8), placed at (0,
/// `race().course.stage_prefs.sky_height` << 8, 0) (into `place` ..), the copy with the
/// place (`turned_matrix`) scaled by (`farthest_scaled` << 4) / 16 (`size`), times
/// `view_turn_matrix`.
pub fn sky_matrix(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), BadInput> {
    use crate::scene::matrix;
    let angle = (ctx.reads.stage_prefs.read(|p| p.sky_turn) << 8).wrapping_neg();
    let mut kept = ctx.view.matrices.borrow_mut();
    let m = &mut *kept;
    matrix::set_turn_y(
        &ctx.reads.buffers.engine_tables,
        &mut m.turn_y_matrix,
        &mut m.turn_y_work,
        angle,
    );
    let height = ctx.reads.stage_prefs.read(|p| p.sky_height) << 8;
    ctx.view.object.borrow_mut().place[0] = 0u32;
    ctx.view.object.borrow_mut().place[2] = 0u32;
    ctx.view.object.borrow_mut().place[1] = height;
    matrix::set_place(&mut m.placed_matrix, [0, height, 0]);
    matrix::copy_matrix_with_place(&m.turn_y_matrix, &m.placed_matrix, &mut m.turned_matrix);
    let size = ctx.view.polygon.window.borrow().farthest_scaled << 4;
    ctx.view.object.borrow_mut().size = size;
    let scale = crate::math::x87::int_over_16(size as i32);
    for cell in &mut m.turned_matrix {
        *cell = crate::math::x87::mul(*cell, scale);
    }
    matrix::multiply_matrices(
        &m.turned_matrix,
        &m.view_turn_matrix,
        &mut m.made_object_matrix,
    );
    Ok(())
}

/// The end of `view_matrices` : the place negated into `placed_matrix` (kept also at), times the
/// view's turn into `view_matrix`, the turn undone into `view_back_matrix` (the view's own).
fn view_place_tail(
    course: crate::race::owner_course::CourseReads<'_>,
    m: &ViewMatrices,
    state: ViewState,
) -> Result<(), BadInput> {
    use crate::scene::{matrix, view_matrix};
    let p = state.view_place.map(u32::wrapping_neg);
    let mut kept = m.borrow_mut();
    let kept = &mut *kept;
    matrix::set_place(&mut kept.placed_matrix, p);
    kept.view_place_matrix = kept.placed_matrix;
    matrix::multiply_matrices(
        &kept.placed_matrix,
        &kept.view_turn_matrix,
        &mut kept.view_matrix,
    );
    kept.view_back_matrix = view_matrix::inverted_turn(course, &kept.view_turn_matrix);
    Ok(())
}

/// `race::view_matrices`: the view's matrices for the frame. With `view_mirrored` set:
/// `view_turn_matrix` the kept turn (`race().frame.matrices.view_turn_copy`). Else for the car's
/// views 0 .. 3 (`car.camera_view`) the car's turn undone (`crash_matrix`); for the others the
/// identity turned by the view's angles (fine y, fine x, z of `view_angles`,
/// `race().frame.view_and_records.view_vars.view_angles[1]`,
/// `race().frame.view_and_records.view_vars.view_angles[2]`, each negated; then again by
/// `race().frame.view_and_records.view_vars.view_angles[3]` ..
/// `race().frame.view_and_records.view_vars.view_angles[5]`), each step `turn y`, `turn x`, `turn
/// z` multiplied in. Views 1 and 2 then roll with the car (by its speed and steering, eased into
/// `car.camera_roll`, turned about z). Then the place (see `view_place_tail`).
pub fn view_matrices(
    course: crate::race::owner_course::CourseReads<'_>,
    m: &ViewMatrices,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), BadInput> {
    // The matrices are handed on to what reads them outside the drawing, the listener and the car
    // sounds, on the game's thread in the views' order: `hand_on_view_matrices`.
    view_own_matrices(
        course,
        m,
        *view.state.borrow(),
        &view.car,
        view.polygon.scalars.borrow().view_mirrored != 0,
    )
}

/// A drawn view's matrices into the shared ones (`race().frame.matrices`), read outside the drawing
/// (the listener before any view, the car sounds before the first view's turn is kept): done on the
/// game's thread, once the view is drawn, in the views' order (`race::draw_race_scene`'s
/// `view_drawn`; no drawing part reads them).
pub fn hand_on_view_matrices(race: &Race, m: &ViewMatrices) {
    let own = m.borrow();
    let mut shared = race.frame.view_kept.borrow_mut();
    shared.view_turn_matrix = own.view_turn_matrix;
    shared.view_place_matrix = own.view_place_matrix;
    shared.view_matrix = own.view_matrix;
    shared.view_back_matrix = own.view_back_matrix;
}

/// `view_matrices` into the view's own matrices.
fn view_own_matrices(
    course: crate::race::owner_course::CourseReads<'_>,
    m: &ViewMatrices,
    state: ViewState,
    car: &crate::physics::car_store::ViewCar,
    mirrored: bool,
) -> Result<(), BadInput> {
    use crate::scene::matrix::{matrix_of, multiply_matrices};
    use crate::scene::view_matrix::{Axis, inverted_turn, set_fine_turn};
    if mirrored {
        {
            let mut kept = m.borrow_mut();
            kept.view_turn_matrix = kept.view_turn_copy;
        }
        return view_place_tail(course, m, state);
    }
    let view: u32 = car.camera_view.get();
    if view <= 3 {
        m.borrow_mut().view_turn_matrix = inverted_turn(course, &matrix_of(&car.crash_matrix));
    } else {
        m.borrow_mut().view_turn_matrix = crate::race::matrices_store::identity_matrix();
        for angles in [[0usize, 1, 2], [3, 4, 5]] {
            set_fine_turn(
                course,
                m,
                Axis::Y,
                state.view_angles[angles[0]].wrapping_neg(),
            );
            set_fine_turn(
                course,
                m,
                Axis::X,
                state.view_angles[angles[1]].wrapping_neg(),
            );
            set_fine_turn(
                course,
                m,
                Axis::Z,
                state.view_angles[angles[2]].wrapping_neg(),
            );
            let mut kept = m.borrow_mut();
            let k = &mut *kept;
            multiply_matrices(&k.view_turn_matrix, &k.turn_y_matrix, &mut k.turned_matrix);
            multiply_matrices(&k.turned_matrix, &k.turn_x_matrix, &mut k.lit_matrix);
            multiply_matrices(&k.lit_matrix, &k.turn_z_matrix, &mut k.view_turn_matrix);
        }
    }
    if view == 1 || view == 2 {
        set_fine_turn(course, m, Axis::Z, rolled_angle(course.numbers, car));
        let mut kept = m.borrow_mut();
        let k = &mut *kept;
        multiply_matrices(&k.view_turn_matrix, &k.turn_z_matrix, &mut k.turned_matrix);
        k.view_turn_matrix = k.turned_matrix;
    }
    view_place_tail(course, m, state)
}

/// The angle views 1 and 2 roll by: the car's speed and steering eased into the camera roll.
fn rolled_angle(
    numbers: &crate::race::numbers_store::Numbers,
    car: &crate::physics::car_store::ViewCar,
) -> u32 {
    use crate::math::x87::{below_or_unordered, eased_roll};
    let mut v = crate::math::x87::div(car.local_velocity.bits_at(2), numbers.ten);
    if below_or_unordered(v, numbers.zero) {
        v = numbers.zero.get();
    }
    if !below_or_unordered(v, numbers.one) {
        v = numbers.one.get();
    }
    let roll = eased_roll(
        car.steering_raw.get() as i32,
        numbers.k65536.get(),
        numbers.one_over_two_pi.get(),
        v,
        numbers.tenth.get(),
        car.camera_roll.get(),
        numbers.half.get(),
    );
    car.camera_roll.set(roll);
    (crate::math::x87::mul3_to_int(
        roll,
        numbers.k65536.get(),
        numbers.k4096.get(),
        numbers.one_over_two_pi.get(),
    ) as u32)
        << 4
}

/// `race::camera_view`: the view for the frame:
/// `race().frame.view_and_records.view_vars.view_car_flag_b`,
/// `race().frame.view_and_records.view_vars.view_car_flag_a`,
/// `race().frame.view_and_records.view_vars.view_car_stage` from `car.camera_mode_is_2_or_3`,
/// `camera_switch_share`, `camera_view`; then without `scene_set_up` `race::place_view`; else the
/// place from the car's camera (`car.camera_position`, the angles from `camera_heading` ..,
/// `race().frame.view_and_records.view_vars.view_scale` from `camera_zoom`) or, with
/// `view_mirrored`, from the crash (the kept turn made from `race().frame.matrices.view_work` and
/// `crash_matrix`, the place the car's raised by 0x8000), written as whole numbers and floats (also
/// to `car.kept_view_place`), and `scene::set_view_window`; then `view_matrices`.
pub fn camera_view(
    reads: crate::scene::workers::DrawReads<'_>,
    m: &ViewMatrices,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), BadInput> {
    use crate::math::x87::from_int;
    // The place and angles as they are (the camera's, or what the view before left: the mirror's
    // way sets neither, and its objects turn by the main view's angles).
    let vars = &view.vars;
    {
        let mut own = view.state.borrow_mut();
        own.view_place = crate::memory::session::read_bits(&vars.view_place);
        own.view_angles = crate::memory::session::read_bits(&vars.view_angles);
        own.view_car_flag_a_bits = view.car.camera_switch_share.get().to_bits();
        own.view_car_stage = view.car.camera_view.get();
    }
    view.vars
        .view_car_flag_b
        .set(view.car.camera_mode_is_2_or_3.get());
    view.vars
        .view_car_flag_a
        .set(view.car.camera_switch_share.get());
    view.vars.view_car_stage.set(view.car.camera_view.get());
    if reads.switches.scene_set_up() == 0 {
        place_view(&reads.buffers.engine_tables, reads.course(), view)?;
        view_matrices(reads.course(), m, view)?;
        return Ok(());
    }
    let place: [u32; 3] = if view.polygon.scalars.borrow().view_mirrored == 0 {
        view.vars.view_scale.set(view.car.camera_zoom.get());
        for (from, to) in [
            (view.car.camera_heading.get(), 0usize),
            (view.car.camera_pitch.get(), 1),
            (view.car.camera_roll_turn.get(), 2),
            (view.car.camera_offset_turns.bits_at(1), 3),
            (view.car.camera_offset_turns.bits_at(0), 4),
            (view.car.camera_offset_turns.bits_at(2), 5),
        ] {
            view.vars.view_angles[to].set(from);
            view.state.borrow_mut().view_angles[to] = from;
        }
        view.car.camera_position.bits()
    } else {
        let mut kept = m.borrow_mut();
        kept.turned_matrix = crate::scene::matrix::product(
            &crate::race::matrices_store::mirror_work_matrix(),
            &crate::scene::matrix::matrix_of(&view.car.crash_matrix),
            kept.turned_matrix,
        );
        kept.view_turn_copy =
            crate::scene::view_matrix::inverted_turn(reads.course(), &kept.turned_matrix);
        drop(kept);
        let kept = &view.car.kept_focus_point;
        kept[0].set(view.car.position.bits_at(0));
        kept[1].set(view.car.position.bits_at(1).wrapping_add(0x8000));
        kept[2].set(view.car.position.bits_at(2));
        view.vars.view_scale.set(view.vars.view_scale_kept.get());
        [kept[0].get(), kept[1].get(), kept[2].get()]
    };
    for (k, &p) in place.iter().enumerate() {
        view.car.kept_view_place.set_bits_at(k, p);
        view.vars.view_place[k].set(p);
    }
    view.vars.view_x.set(place[0]);
    view.vars.view_z.set(place[2]);
    view.state.borrow_mut().view_place = place;
    view.vars.view_place_f[0].set(from_int(place[0] as i32));
    view.vars.view_place_f[1].set(from_int(place[1] as i32));
    view.vars.view_place_f[2].set(from_int(place[2] as i32));
    if reads.view.players() > 2 {
        // A part of a larger split screen: the projection's scale follows the part's own shape
        // (as where the part's window is first set, `race::draw_race_scene`), not the screen's.
        let size = view.polygon.window.borrow().view_size;
        crate::scene::set_view_window::set_view_window_plain(
            reads.course(),
            &reads.scene_draw,
            (size[0], size[1]),
            &view.polygon.window,
            &view.vars,
            &crate::race::vars_store::SCALE_FACTOR,
        );
    } else {
        crate::scene::set_view_window::set_view_window(
            reads.course(),
            &reads.scene_draw,
            reads.screen,
            &view.polygon.window,
            &view.vars,
            &crate::race::vars_store::SCALE_FACTOR,
        );
    }
    view_matrices(reads.course(), m, view)?;
    Ok(())
}

/// `race::draw_sky`: `race::fetch_view_value`; unless the scene's mode (`draw_mode`) is
/// 1 or 2: the sky (object 1 through `sky_matrix`, kept, full light, `view_kind` 4 and
/// `no_far_clip` 1 while it is drawn by parts 1 .. 4); then with a sun
/// (`race().course.stage_prefs.fog_near` + `race().course.stage_prefs.fog_far` not 0): its
/// direction from `race().course.stage_prefs.sky_turn` + `race().course.stage_prefs.sun_turn` and
/// `race().course.stage_prefs.sun_height`, turned by the sky's matrix, and when in front (z not
/// below `race().course.numbers.k4096_a`) its place on the screen (`ViewSun::place` / `hit` ..),
/// the view's sun (`ViewSun`) on the screen and its size (256 less half its distance from the
/// centre, at least 0).
pub fn draw_sky(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    let race = ctx.race;
    fetch_view_value(
        ctx.reads.frame_cars,
        ctx.reads.view,
        &ctx.view.polygon.sun,
        ctx.view.drawn_view(race),
    )?;
    // Not left from the previous view's last object.
    ctx.view.light.borrow_mut().other_light = 0;
    let mode: u32 = ctx.reads.light_work.draw_mode;
    if mode == 1 || mode == 2 {
        return Ok(());
    }
    sky_matrix(ctx)?;
    {
        let mut kept = ctx.view.matrices.borrow_mut();
        kept.object_matrix_slots = kept.made_object_matrix.map(f32::to_bits);
    }
    ctx.view.cars.borrow_mut().object_number = 1;
    ctx.view.light.borrow_mut().view_kind = 4;
    ctx.view.light.borrow_mut().light_lowest = 0xFF;
    ctx.view.light.borrow_mut().light_highest = 0xFF;
    ctx.view.light.borrow_mut().light_level = 0xFF;
    ctx.view.light.borrow_mut().light_turns[0] = 0;
    ctx.view.light.borrow_mut().light_turns[1] = 0;
    ctx.view.face.borrow_mut().no_far_clip = 1;
    crate::scene::glow_render_states::glow_render_states(ctx.part);
    ctx.view.light.borrow_mut().light_kind = 0u32;
    // The matrix is taken where its copy was left: past `made_object_matrix`.
    crate::scene::draw_scene_object::draw_scene_object_in_game(
        ctx,
        Some(Description::Floats(
            &ctx.view.matrices,
            DescribedMatrix::SkyCopy,
        )),
    )?;
    crate::scene::depth_buffer_on::depth_buffer(ctx.part, true);
    ctx.view.light.borrow_mut().view_kind = 0;
    ctx.view.face.borrow_mut().no_far_clip = 0;
    place_sun(ctx)
}

/// The sun: its direction from the stage's sun angles, turned by the sky's matrix, and when it is
/// in front its place on the screen and its size. Cleared first, and left cleared when both sun
/// settings are 0 or the sun is behind the view.
fn place_sun(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    use crate::math::x87::{sun_direction, sun_on_screen, sun_size, tbyte_to_int, to_tbyte};
    let sun = &ctx.view.polygon.sun;
    {
        let mut sun = sun.borrow_mut();
        sun.on_screen = 0;
        sun.size = 0;
    }
    if ctx
        .reads
        .stage_prefs
        .read(|p| p.fog_near.wrapping_add(p.fog_far))
        == 0
    {
        return Ok(());
    }
    let angle = ctx
        .reads
        .stage_prefs
        .read(|p| p.sun_turn.wrapping_add(p.sky_turn))
        & 0xFF;
    let dir = sun_direction(
        angle as i32,
        ctx.reads.numbers.sixty_four,
        crate::race::frame_store::SUN_DEPTH,
        ctx.reads.stage_prefs.read(|p| p.sun_height) as i32,
        ctx.reads.numbers.k98304,
    );
    // The direction and the turned point are locals: nothing reads `FrameWork.sun_floats` or
    // `sun_points`.
    let turned = crate::scene::rotate_point::rotate_point_by(
        &ctx.view.matrices.borrow().made_object_matrix,
        [to_tbyte(dir[0]), to_tbyte(dir[1]), to_tbyte(dir[2])],
    );
    let place = turned.map(tbyte_to_int);
    let z = crate::math::x87::from_int(place[2] as i32);
    if crate::math::x87::below_or_unordered(z, ctx.reads.numbers.k4096_a) {
        return Ok(());
    }
    let flat = ctx.reads.stage_prefs.read(|p| p.flat_depth) != 0;
    let window = &ctx.view.polygon.window;
    let ([fx, fy], [ix, iy]) = sun_on_screen(
        [
            place[0] as i32,
            place[1] as i32,
            place[2] as i32,
            ctx.reads.course_work.get().flat_depth as i32,
        ],
        window.borrow().view_scale,
        [
            window.borrow().view_centre[1],
            window.borrow().view_centre[0],
        ],
        flat,
        &crate::memory::session::Dword::new(0),
    );
    let (ix, iy) = (ix as u32, iy as u32);
    let on = ix < ctx.reads.screen.width.get() && iy < ctx.reads.screen.height.get();
    {
        let mut sun = sun.borrow_mut();
        sun.place = [fx.to_bits(), fy.to_bits()];
        sun.hit = [ix, iy];
        sun.on_screen = u32::from(on);
    }
    let size = sun_size(
        fx,
        ctx.view.polygon.window.borrow().view_centre[0],
        fy,
        ctx.view.polygon.window.borrow().view_centre[1],
        ctx.reads.numbers.half.get(),
        ctx.reads.numbers.k256.get(),
    );
    sun.borrow_mut().size = if (size as i32) < 0 { 0 } else { size };
    Ok(())
}

// kept_reflection_normals: 0x400 dwords
