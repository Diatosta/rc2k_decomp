//! The view: draw distance, fog, the view's place and matrices, and the sky.

use super::*;
use crate::errors::{BadInput, Refused};
use crate::physics::car_store::CAR;
use crate::race::matrices_store::SceneMatrices;
use crate::race::owner::Race;
use crate::race::owner::race;
use crate::race::owner_course::Course;
use crate::race::owner_frame::Frame;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::view_store::RACE_VIEW;
use crate::scene::draw_store::DRAW;
use crate::scene::engine_tables_store::ENGINE_TABLES;
use crate::scene::light_store::LIGHT_STATE;
use crate::scene::light_work_store::LIGHT_WORK;
use crate::scene::object_store::OBJECT_WORK;
use crate::session::SCREEN;

/// `race::draw_distance_settings` (0x43932A): the draw distance's level (`race().course.stage_prefs.draw_distance_level`,
/// held to 0 ..= 4 by sign) picks from six tables: `race().course.stage_prefs.grid_reach` (at least 1, by sign),
/// `race().course.stage_prefs.fog_end`, `race().course.stage_prefs.fog_start`, `race().course.stage_prefs.fog_low`, `race().course.stage_prefs.fog_high`, `race().course.stage_prefs.nearer`.
pub fn draw_distance_settings(course: &Course) {
    // Converted: `rc2k_core::race::view::{draw_distance_level, grid_reach}`.
    use rc2k_core::race::view::{draw_distance_level, grid_reach};
    let level = draw_distance_level(course.stage_prefs.draw_distance_level.bits());
    let level = level as usize;
    course
        .stage_prefs
        .grid_reach
        .set(grid_reach(course.stage_prefs.reach_by_level[level].bits()));
    course
        .stage_prefs
        .fog_end
        .set_bits(course.stage_prefs.fog_end_by_level[level].bits());
    course
        .stage_prefs
        .fog_start
        .set_bits(course.stage_prefs.fog_start_by_level[level].bits());
    course
        .stage_prefs
        .fog_low
        .set_bits(course.stage_prefs.fog_low_by_level[level].bits());
    course
        .stage_prefs
        .fog_high
        .set_bits(course.stage_prefs.fog_high_by_level[level].bits());
    course
        .stage_prefs
        .nearer
        .set_bits(course.stage_prefs.nearer_by_level[level].bits());
}

/// `race::sample_noise` (0x44FD38): the 64 x 64 byte map `map` bytes into `g_noiseBytes`
/// sampled at the work car's
/// place (`g_car` +0x6A4 and +0x6AC: the top byte the cell, the next two the fraction):
/// the four corners into `LIGHT_STATE.noise_samples` .. `LIGHT_STATE.noise_samples[3]`, blended along x into
/// `LIGHT_STATE.noise_samples[4]` / `LIGHT_STATE.noise_samples[5]` and then along z, each step `imul` and `shrd` by 16;
/// the result shifted left 8 is returned.
pub fn sample_noise(map: usize) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::race::view::sample_noise`.
    let (px, pz) = (CAR.camera_position[0].get(), CAR.camera_position[2].get());
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
    let out = &LIGHT_STATE.noise_samples;
    for (cell, &c) in out.iter().zip(&s.corners) {
        cell.set(c);
    }
    out[4].set(s.near);
    out[5].set(s.far);
    Ok(s.value)
}

// g_fogTable: 0x1000 bytes

/// `race::fog_table` (0x44FC33): `draw_distance_settings`; the noise at the car
/// (`sample_noise` of `g_noiseBytes`) spread between `race().course.stage_prefs.fog_low` and `race().course.stage_prefs.fog_high` into
/// the step `LIGHT_WORK.fog_step`; `LIGHT_STATE.fog_near` / `LIGHT_STATE.fog_far` = (`race().frame.frame_work.sun_size` x
/// `race().frame.frame_work.view_out` >> 4) x `race().course.stage_prefs.fog_near` / `race().course.stage_prefs.fog_far` >> 8; `g_fogTableLength` =
/// `RACE_VIEW.view_depth[1]` x 8 bytes of `g_fogTable`, each ((sum of steps) >> 16) + `race().course.stage_prefs.fog_start` +
/// `LIGHT_STATE.fog_near` held to 0 ..= 0xFF; `LIGHT_STATE.fog_next` the next one's value with
/// `race().course.stage_prefs.fog_end` (held) plus `LIGHT_STATE.fog_near` (held). Refuses (before anything) a table length of 0 or past 0x1000.
pub fn fog_table(race: &Race) -> Result<(), Refused> {
    let length = RACE_VIEW.view_depth[1].get().wrapping_shl(3);
    if length == 0 || length > 0x1000 {
        return Err(BadInput("a fog table length of 0 or past 0x1000").into());
    }
    // Converted: `rc2k_core::race::view::fog_table`.
    draw_distance_settings(&race.course);
    let noise = sample_noise(0)?;
    let fog = rc2k_core::race::view::fog_table(
        noise,
        &rc2k_core::race::view::FogSettings {
            low: race.course.stage_prefs.fog_low.get(),
            high: race.course.stage_prefs.fog_high.get(),
            scale: [
                race.frame.frame_work.sun_size.get(),
                race.frame.frame_work.view_out.get(),
            ],
            near: race.course.stage_prefs.fog_near.get(),
            far: race.course.stage_prefs.fog_far.get(),
            start: race.course.stage_prefs.fog_start.get(),
            end: race.course.stage_prefs.fog_end.get(),
            length,
        },
    );
    LIGHT_WORK.fog_step.set(fog.step);
    LIGHT_STATE.fog_near.set(fog.near);
    LIGHT_STATE.fog_far.set(fog.far);
    LIGHT_STATE.fog_table_length.set_bits(length);
    // The table's bytes (any length up to 0x1000; a last part-dword keeps its other bytes).
    for (cell, b) in LIGHT_STATE.fog_table.iter().zip(fog.table.chunks(4)) {
        let mut d = cell.get().to_le_bytes();
        d[..b.len()].copy_from_slice(b);
        cell.set(u32::from_le_bytes(d));
    }
    LIGHT_STATE.fog_next.set(fog.next);
    Ok(())
}

/// `race::set_up_fog` (0x44FBD9): `scene::restart_clock`; `DRAW.fog_colour` = the fog colour
/// from `race().course.stage_prefs.fog_colour` (red), `race().course.stage_prefs.fog_colour[1]`, `race().course.stage_prefs.fog_colour[2]` (each shifted into its byte
/// and added); `fog_table`; the colour with 0xFF000000 to the device. None of it reads a
/// register at entry, and its callers (`race::draw_view_scene`, `race::mirror_view_window`)
/// read none it leaves. Refuses (before anything) what `fog_table` would.
pub fn set_up_fog() -> Result<(), Refused> {
    let race = race();
    let course = &race.course;
    let length = RACE_VIEW.view_depth[1].get().wrapping_shl(3);
    if length == 0 || length > 0x1000 {
        return Err(BadInput("a fog table length of 0 or past 0x1000").into());
    }
    crate::scene::restart_clock::restart_clock(crate::system::tick_count::tick_count());
    // Converted: `rc2k_core::race::view::{fog_colour, device_fog_colour}`.
    let colour = rc2k_core::race::view::fog_colour(
        course.stage_prefs.fog_colour[0].bits(),
        course.stage_prefs.fog_colour[1].bits(),
        course.stage_prefs.fog_colour[2].bits(),
    );
    DRAW.fog_colour.set_bits(colour);
    fog_table(race)?;
    crate::scene::set_fog_colour::set_fog_colour(rc2k_core::race::view::device_fog_colour(
        DRAW.fog_colour.bits(),
    ));
    Ok(())
}

/// `race::place_view` (0x4389A8): the view's place from the camera (`race().course.course.place_x`,
/// `race().course.course.place_y`, `race().course.course.place_z`) and its angles. Flat (`race().course.stage_prefs.flat_depth` set): the place as
/// it is, `g_car` +0x6A8 += `race().course.course.flat_depth`, `g_viewAngles` = `race().course.course.place_heading`,
/// `race().frame.view_and_records.view_vars.view_angles[1]` = 0x40000000. Else the place moved by (-`race().course.course.flat_depth`, 0, 0) turned by
/// `race().course.course.camera_first` (`math::turn_point_xz_ecx`) and that turned again by `race().course.course.place_heading`
/// (`math::turn_point_xz`, its x 0, y the first turn's z, z its x), `g_viewAngles` and
/// `race().frame.view_and_records.view_vars.view_angles[1]` the two angles, and the place also as whole numbers and floats
/// (`race().frame.view_and_records.view_vars.view_x` / `race().frame.view_and_records.view_vars.view_z`, `race().frame.view_and_records.view_vars.view_place_f` ..). `race().frame.view_and_records.view_vars.view_angles[2]` .. `race().frame.view_and_records.view_vars.view_angles[5]` 0
/// both ways.
pub fn place_view(race: &Race) -> Result<(), BadInput> {
    use crate::math::x87::from_int;
    let place = &race.frame.view_and_records.view_vars.view_place;
    place[0].set(race.course.course.place_x.bits());
    place[1].set(race.course.course.place_y.bits());
    place[2].set(race.course.course.place_z.bits());
    let zero = || {
        for a in [2, 4, 3, 5] {
            race.frame.view_and_records.view_vars.view_angles[a].set_bits(0);
        }
    };
    if race.course.stage_prefs.flat_depth.get() != 0 {
        let at = &CAR.camera_position[1];
        at.set(at.get().wrapping_add(race.course.course.flat_depth.bits()));
        race.frame.view_and_records.view_vars.view_angles[0]
            .set_bits(race.course.course.place_heading.get());
        race.frame.view_and_records.view_vars.view_angles[1].set(0x4000_0000u32);
        zero();
        return Ok(());
    }
    let first: u32 = race.course.course.camera_first.get();
    let second: u32 = race.course.course.place_heading.get();
    let back = (race.course.course.flat_depth.get()).wrapping_neg();
    let [x1, z1, _] = crate::math::turn_point_xz_ecx::turn_point_xz_ecx(back as i32, 0, 0, first);
    let [x2, _, z2, _] = crate::math::turn_point_xz::turn_point_xz(0, z1 as i32, x1 as i32, second);
    place[0].set(place[0].get().wrapping_add(x2));
    place[1].set(place[1].get().wrapping_add(z1));
    place[2].set(place[2].get().wrapping_add(z2));
    race.frame.view_and_records.view_vars.view_angles[0].set_bits(second);
    race.frame.view_and_records.view_vars.view_angles[1].set(first);
    zero();
    race.frame
        .view_and_records
        .view_vars
        .view_x
        .set(place[0].get());
    race.frame.view_and_records.view_vars.view_place_f[0].set_bits(from_int(place[0].get() as i32));
    race.frame.view_and_records.view_vars.view_place_f[1].set_bits(from_int(place[1].get() as i32));
    race.frame
        .view_and_records
        .view_vars
        .view_z
        .set(place[2].get());
    race.frame.view_and_records.view_vars.view_place_f[2].set_bits(from_int(place[2].get() as i32));
    Ok(())
}

/// `race::sky_matrix` (0x42725E): `g_madeObjectMatrix` for the sky: a turn about y by
/// -(`race().course.stage_prefs.sky_turn` << 8), placed at (0, `race().course.stage_prefs.sky_height` << 8, 0) (into `OBJECT_WORK.place` ..),
/// the copy with the place (`g_turnedMatrix`) scaled by (`DRAW.farthest_scaled` << 4) / 16
/// (`OBJECT_WORK.size`, through `DRAW.scratch`), times `g_viewTurnMatrix`.
pub fn sky_matrix(race: &Race) -> Result<(), BadInput> {
    use crate::scene::matrix;
    let angle = (race.course.stage_prefs.sky_turn.get() << 8).wrapping_neg();
    matrix::set_turn_y(angle);
    let height = race.course.stage_prefs.sky_height.get() << 8;
    OBJECT_WORK.place[0].set(0u32);
    OBJECT_WORK.place[2].set(0u32);
    OBJECT_WORK.place[1].set(height);
    matrix::set_place([0, height, 0]);
    matrix::copy_matrix_with_place(
        &race.frame.matrices.turn_y_matrix,
        &race.frame.matrices.placed_matrix,
        &race.frame.matrices.turned_matrix,
    );
    let size = DRAW.farthest_scaled.get() << 4;
    OBJECT_WORK.size.set(size);
    let scale = crate::math::x87::int_over_16(size as i32);
    DRAW.scratch.set_bits(scale);
    for cell in &race.frame.matrices.turned_matrix {
        cell.set_bits(crate::math::x87::mul(cell.bits(), scale));
    }
    matrix::multiply_matrices(
        &race.frame.matrices.turned_matrix,
        &race.frame.matrices.view_turn_matrix,
        &race.frame.matrices.made_object_matrix,
    );
    Ok(())
}

/// The end of `view_matrices` (0x4388E5 .. and 0x438950 ..): the place negated into
/// `g_placedMatrix` (kept also at 0x5EA1CC), times the view's turn into `g_viewMatrix`,
/// the turn undone into `g_viewBackMatrix`.
pub(super) fn view_place_tail(frame: &Frame) -> Result<(), BadInput> {
    use crate::scene::{matrix, view_matrix};
    let p = crate::session::read_bits(&frame.view_and_records.view_vars.view_place)
        .map(u32::wrapping_neg);
    matrix::set_place(p);
    crate::math::cells::copy_cells(
        &frame.matrices.placed_matrix,
        &frame.matrices.view_place_matrix,
        16,
    );
    matrix::multiply_matrices(
        &frame.matrices.placed_matrix,
        &frame.matrices.view_turn_matrix,
        &frame.matrices.view_matrix,
    );
    view_matrix::invert_turn(
        &frame.matrices.view_turn_matrix,
        &frame.matrices.view_back_matrix,
    );
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
pub fn view_matrices(race: &Race) -> Result<(), BadInput> {
    use crate::scene::matrix::multiply_matrices;
    use crate::scene::view_matrix::{Axis, invert_turn, set_fine_turn};
    let (view_turn, turn_y, turn_x, turn_z) = (
        &race.frame.matrices.view_turn_matrix,
        &race.frame.matrices.turn_y_matrix,
        &race.frame.matrices.turn_x_matrix,
        &race.frame.matrices.turn_z_matrix,
    );
    let (turned, lit) = (
        &race.frame.matrices.turned_matrix,
        &race.frame.matrices.lit_matrix,
    );
    if race.physics.ground.view_mirrored.get() != 0 {
        crate::math::cells::copy_cells(&race.frame.matrices.view_turn_copy, &view_turn, 16);
        view_place_tail(&race.frame)?;
        return Ok(());
    }
    let view: u32 = CAR.camera_view.get();
    if view <= 3 {
        invert_turn(&CAR.crash_matrix, view_turn);
    } else {
        crate::math::cells::copy_cells(&race.frame.matrices.identity_matrix, &view_turn, 16);
        for angles in [[0usize, 1, 2], [3, 4, 5]] {
            set_fine_turn(
                Axis::Y,
                race.frame.view_and_records.view_vars.view_angles[angles[0]]
                    .bits()
                    .wrapping_neg(),
            );
            set_fine_turn(
                Axis::X,
                race.frame.view_and_records.view_vars.view_angles[angles[1]]
                    .bits()
                    .wrapping_neg(),
            );
            set_fine_turn(
                Axis::Z,
                race.frame.view_and_records.view_vars.view_angles[angles[2]]
                    .bits()
                    .wrapping_neg(),
            );
            multiply_matrices(view_turn, turn_y, turned);
            multiply_matrices(turned, turn_x, lit);
            multiply_matrices(lit, turn_z, view_turn);
        }
    }
    if view == 1 || view == 2 {
        use crate::math::x87_load::{below_or_unordered, eased_roll, product4_to_int};
        let mut v =
            crate::math::x87::div(CAR.local_velocity[2].bits(), race.course.numbers.ten.bits());
        if below_or_unordered(v, race.course.numbers.zero.bits()) {
            v = race.course.numbers.zero.bits();
        }
        if !below_or_unordered(v, race.course.numbers.one.bits()) {
            v = race.course.numbers.one.bits();
        }
        DRAW.scratch.set_bits(v);
        let roll = eased_roll(
            CAR.steering_raw.get() as i32,
            race.course.numbers.k65536.bits(),
            race.course.numbers.one_over_two_pi.bits(),
            v,
            race.course.numbers.tenth.bits(),
            CAR.camera_roll.bits(),
            race.course.numbers.half.bits(),
        );
        CAR.camera_roll.set_bits(roll);
        let angle = product4_to_int(
            roll,
            race.course.numbers.k65536.bits(),
            race.course.numbers.k4096.bits(),
            race.course.numbers.one_over_two_pi.bits(),
        ) << 4;
        DRAW.scratch.set_bits(angle);
        set_fine_turn(Axis::Z, angle);
        multiply_matrices(view_turn, turn_z, turned);
        crate::math::cells::copy_cells(&turned, &view_turn, 16);
    }
    view_place_tail(&race.frame)?;
    Ok(())
}

/// `race::camera_view` (0x438325): the view for the frame: `race().frame.view_and_records.view_vars.view_car_flag_b`,
/// `race().frame.view_and_records.view_vars.view_car_flag_a`, `race().frame.view_and_records.view_vars.view_car_stage` from `g_car` +0x6F0, +0x69C, +0x684;
/// then without `FRAME_SWITCHES.other_way` `race::place_view`; else the place from the car's
/// camera (`g_car` +0x6A4 .., the angles from +0x6B0 .., `race().frame.view_and_records.view_vars.view_scale` from +0x6C8)
/// or, with `GROUND_WORK.view_mirrored`, from the crash (the kept turn made from
/// `race().frame.matrices.view_work` and `g_crashMatrix`, the place the car's raised by 0x8000), written as
/// whole numbers and floats (also to `g_car` +0x734 ..), and `scene::set_view_window`; then
/// `view_matrices`.
pub fn camera_view() -> Result<(), BadInput> {
    let race = race();
    use crate::math::x87::from_int;
    race.frame
        .view_and_records
        .view_vars
        .view_car_flag_b
        .set(CAR.camera_mode_is_2_or_3.get());
    race.frame
        .view_and_records
        .view_vars
        .view_car_flag_a
        .set(CAR.camera_switch_share.bits());
    race.frame
        .view_and_records
        .view_vars
        .view_car_stage
        .set(CAR.camera_view.get());
    if FRAME_SWITCHES.other_way.get() == 0 {
        place_view(race)?;
        view_matrices(race)?;
        return Ok(());
    }
    let place: [u32; 3] = if race.physics.ground.view_mirrored.get() == 0 {
        race.frame
            .view_and_records
            .view_vars
            .view_scale
            .set_bits(CAR.camera_zoom.bits());
        for (from, to) in [
            (&CAR.camera_heading, 0usize),
            (&CAR.camera_pitch, 1),
            (&CAR.camera_roll_turn, 2),
            (&CAR.camera_offset_turns[1], 3),
            (&CAR.camera_offset_turns[0], 4),
            (&CAR.camera_offset_turns[2], 5),
        ] {
            race.frame.view_and_records.view_vars.view_angles[to].set_bits(from.get());
        }
        crate::session::read_bits(&CAR.camera_position)
    } else {
        crate::scene::matrix::multiply_matrices(
            &race.frame.matrices.view_work,
            &CAR.crash_matrix,
            &race.frame.matrices.turned_matrix,
        );
        crate::scene::view_matrix::invert_turn(
            &race.frame.matrices.turned_matrix,
            &race.frame.matrices.view_turn_copy,
        );
        let kept = &CAR.unknown_6cc;
        kept[0].set(CAR.position[0].get());
        kept[1].set(CAR.position[1].get().wrapping_add(0x8000));
        kept[2].set(CAR.position[2].get());
        race.frame
            .view_and_records
            .view_vars
            .view_scale
            .set_bits(race.frame.view_and_records.view_vars.view_scale_kept.bits());
        [kept[0].get(), kept[1].get(), kept[2].get()]
    };
    for (k, &p) in place.iter().enumerate() {
        CAR.kept_view_place[k].set(p);
        race.frame.view_and_records.view_vars.view_place[k].set(p);
    }
    race.frame.view_and_records.view_vars.view_x.set(place[0]);
    race.frame.view_and_records.view_vars.view_z.set(place[2]);
    race.frame.view_and_records.view_vars.view_place_f[0].set_bits(from_int(place[0] as i32));
    race.frame.view_and_records.view_vars.view_place_f[1].set_bits(from_int(place[1] as i32));
    race.frame.view_and_records.view_vars.view_place_f[2].set_bits(from_int(place[2] as i32));
    crate::scene::set_view_window::set_view_window();
    view_matrices(race)?;
    Ok(())
}

/// `race::draw_sky` (0x455540): `race::fetch_view_value`; `race().course.course.polygon_flags` 0; unless the
/// scene's mode (`LIGHT_STATE.draw_mode`) is 1 or 2: the sky (object 1 through `sky_matrix`,
/// kept at 0x5EA34C, full light, `LIGHT_STATE.view_kind` 4 and `FRAME_SWITCHES.no_far_clip` 1
/// while it is drawn by parts 1 .. 4); then with a sun (`race().course.stage_prefs.fog_near` +
/// `race().course.stage_prefs.fog_far` not 0): its direction from `race().course.stage_prefs.sky_turn` +
/// `race().course.stage_prefs.sun_turn` and `race().course.stage_prefs.sun_height` (`race().frame.frame_work.sun_floats` ..), turned by
/// the sky's matrix (`race().frame.frame_work.sun_points` ..), and when in front (z not below
/// `race().course.numbers.k4096_a`) its place on the screen (`race().frame.frame_work.sun_place` / `race().frame.frame_work.sun_hit`
/// ..), `race().frame.frame_work.sun_on_screen` 1 when on the screen, `race().frame.frame_work.sun_size` its size (256
/// less half its distance from the centre, at least 0).
pub fn draw_sky() -> Result<(), Refused> {
    let race = race();
    let frame = &race.frame;
    fetch_view_value()?;
    race.course.course.polygon_flags.set(0u32);
    let mode: u32 = LIGHT_STATE.draw_mode.get();
    if mode == 1 || mode == 2 {
        return Ok(());
    }
    sky_matrix(race)?;
    crate::math::cells::copy_cells(
        &race.frame.matrices.made_object_matrix,
        &race.frame.matrices.object_matrix_slots,
        16,
    );
    FRAME_SWITCHES.object_number.set_bits(1);
    LIGHT_STATE.view_kind.set_bits(4);
    OBJECT_WORK.light_lowest.set_bits(0xFF);
    OBJECT_WORK.light_highest.set_bits(0xFF);
    OBJECT_WORK.light_level.set_bits(0xFF);
    OBJECT_WORK.light_turns[0].set_bits(0);
    OBJECT_WORK.light_turns[1].set_bits(0);
    FRAME_SWITCHES.no_far_clip.set_bits(1);
    crate::scene::glow_render_states::glow_render_states();
    LIGHT_WORK.light_kind.set(0u32);
    // esi as the `rep movsd` of the matrix's copy leaves it: past `g_madeObjectMatrix`.
    crate::scene::draw_scene_object::draw_scene_object_in_game(Some(matrices_at(
        frame,
        core::mem::offset_of!(SceneMatrices, made_object_matrix) + 0x40,
    )))?;
    crate::race::flush_draw_batch::flush_draw_batch(race)?;
    crate::scene::depth_buffer_on::depth_buffer(true);
    LIGHT_STATE.view_kind.set_bits(0);
    FRAME_SWITCHES.no_far_clip.set_bits(0);
    place_sun(race)
}

/// The sun: its direction from the stage's sun angles, turned by the sky's matrix, and when it is
/// in front its place on the screen and its size. Cleared first, and left cleared when both sun settings are 0 or
/// the sun is behind the view.
fn place_sun(race: &Race) -> Result<(), Refused> {
    use crate::math::x87_load::{sun_direction, sun_on_screen, sun_size, tbyte_to_int, to_tbyte};
    race.frame.frame_work.sun_on_screen.set_bits(0);
    race.frame.frame_work.sun_size.set_bits(0);
    if race
        .course
        .stage_prefs
        .fog_near
        .get()
        .wrapping_add(race.course.stage_prefs.fog_far.bits())
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
    DRAW.scratch.set_bits(angle);
    let dir = sun_direction(
        angle as i32,
        race.course.numbers.sixty_four.bits(),
        race.frame.frame_work.sun_depth.bits(),
        race.course.stage_prefs.sun_height.get() as i32,
        race.course.numbers.k98304.bits(),
    );
    for (n, &d) in dir.iter().enumerate() {
        race.frame.frame_work.sun_floats[n].set_bits(d);
    }
    let turned = crate::scene::rotate_point::rotate_point(
        &race.frame.matrices.made_object_matrix,
        [to_tbyte(dir[0]), to_tbyte(dir[1]), to_tbyte(dir[2])],
    );
    let place = turned.map(tbyte_to_int);
    for (n, &p) in place.iter().enumerate() {
        race.frame.frame_work.sun_points[n].set_bits(p);
    }
    let z = crate::math::x87::from_int(place[2] as i32);
    if crate::math::x87_load::below_or_unordered(z, race.course.numbers.k4096_a.bits()) {
        return Ok(());
    }
    let flat = race.course.stage_prefs.flat_depth.get() != 0;
    let v = [
        place[0],
        place[1],
        place[2],
        race.course.course.flat_depth.get(),
        DRAW.view_scale.bits(),
        DRAW.view_centre[1].bits(),
        DRAW.view_centre[0].bits(),
    ];
    let [fx, ix, fy, iy] = sun_on_screen(&v, flat, &DRAW.scratch);
    race.frame.frame_work.sun_place[0].set(fx);
    race.frame.frame_work.sun_hit[0].set(ix);
    race.frame.frame_work.sun_place[1].set(fy);
    race.frame.frame_work.sun_hit[1].set(iy);
    let on = ix < SCREEN.width.get() && iy < SCREEN.height.get();
    race.frame.frame_work.sun_on_screen.set(u32::from(on));
    let size = sun_size(
        fx,
        DRAW.view_centre[0].bits(),
        fy,
        DRAW.view_centre[1].bits(),
        race.course.numbers.half.bits(),
        race.course.numbers.k256.bits(),
    );
    DRAW.scratch.set_bits(size);
    race.frame
        .frame_work
        .sun_size
        .set(if (size as i32) < 0 { 0 } else { size });
    Ok(())
}

// g_keptReflectionNormals: 0x400 dwords
