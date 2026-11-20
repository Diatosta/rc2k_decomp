//! The camera's step. `physics::camera_low_switch` runs; the car's place goes into `camera_car`;
//! then the car kind's light record (`light_records`, 0x134 bytes each): places x 1/65536, two of
//! them less `camera_drop`, into the camera's light places; two words into the light colour figure;
//! one into the light strength; another x 1/65536, kept for the cars marked in `kind_has_views` (n
//! below 0x22), else the next, into the second strength. With no switch under way and a fixed mode
//! (0x2A .. 0x2D, 9, 10: `nearest_camera_2` / `replay_cameras_2` of the car's `checkpoint`; 0x2E ..
//! 0x31, 7, 8: the first two) the camera is put at that face's first scene point,
//! `physics::camera_height_bias` runs, and the camera is aimed at the car (arctangent heading and
//! pitch, x 65536 x `k4096` x `one_over_two_pi`, << 4, the pitch negated); the distance
//! (`physics::vector_length`) goes into `camera_distance`; the zoom `camera_zoom` is
//! `camera_zoom_kept`, or 0x180 x the distance / `five` held at least `k384` with the record's
//! `fixed_zoom`; the roll and turns are 0. Otherwise the camera is put at the car (also into
//! `place_x`, the heading into `place_heading`); a switch under way ends after
//! `camera_switch_length` steps (the mode back, `value_jump_steps` = 8, modes 7 on to 0) or its
//! share (held 0 ..= 1) goes into `camera_switch_share`; `physics::take_first_view` of the mode and
//! `take_second_view` of the mode before run; their figures are blended by the share into the turn
//! targets (`start_heading`), the offsets (`camera_offset_x`) and the turns (`camera_offset_turns`,
//! << 16); the shares (`view_blend_aux`); the zoom held at least `camera_zoom_hold`;
//! `physics::ease_view_angles`, `ease_view_blend` and `physics::camera_avoid_ground` run; the
//! offset's length goes into `flat_depth` and the first camera figure into `camera_first`.
//!
//! Refuses a mode past 0x33 on the blending path (past `view_records`), and what the ground checks
//! refuse; everything written is put back.

use crate::errors::Refused;
use crate::math::cell_x87::CellX87;
use crate::math::matrix3::rounded_value;
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::to_int;
use crate::physics::camera_store::CameraWork;
use crate::physics::camera_view::record;
use crate::physics::car_store::Car;
use crate::physics::flow::{ZERO_BITS as ZERO, hold_high, hold_low, hold_low_bits};
use crate::physics::step_families::CameraInputs;
use crate::physics::tables_store::PhysicsTables;
use crate::race::course_store::CourseWork;
use crate::race::groups_store::ViewAndRecords;
use crate::race::numbers_store::Numbers;
use crate::race::view_vars_store::ViewVars;

use crate::memory::session::DwordLike;
use crate::memory::session::PlainFloat;
use rc2k_core::physics::camera_view;

/// The fixed camera: `second` picks the second tables.
fn fixed_camera(c: &CameraInputs<'_>, second: bool) -> Result<(), crate::errors::BadInput> {
    let (nearest, replay) = if second {
        (
            &c.checkpoints.nearest_camera_2,
            &c.checkpoints.replay_cameras_2,
        )
    } else {
        (
            &c.checkpoints.nearest_camera_1,
            &c.checkpoints.replay_cameras_1,
        )
    };
    // The car's checkpoint is held to the line's count (5144 at most; the tables hold 5146),
    // its nearest camera is a number of the list (0x400 at most; the table holds 0x400).
    let camera = nearest
        .get(c.car.checkpoint.get() as usize)
        .ok_or(crate::errors::BadInput(
            "a checkpoint past the nearest cameras",
        ))?
        .get();
    let face = replay
        .get(camera as usize)
        .ok_or(crate::errors::BadInput("a camera past the replay cameras"))?
        .get();
    let point = crate::race::course_places::face_field::<u32>(c.course.reads(), face, 4)
        .unwrap_or_else(|e| crate::fatal::raise::end("a checkpoint face", e));
    for k in 0..3 {
        c.car.camera_position[k as usize].set(
            crate::race::scene_points::point_word(
                c.course.reads(),
                &crate::race::scene_points::no_own(),
                point,
                k,
            )
            .unwrap_or_else(|e| crate::fatal::raise::end("a scene point", e)),
        );
    }
    crate::physics::camera_height_bias::camera_height_bias(c.numbers, c.view_records, c.car)?;
    let d = |k: usize| {
        c.car.position[k]
            .get()
            .wrapping_sub(c.car.camera_position[k].get())
    };
    let (dx, dy, dz) = (d(0), d(1), d(2));
    let v = [dx, dz, dy];
    x87_1(c.car, c.numbers, v);
    c.car.camera_heading.set(c.car.camera_heading.get() << 4);
    c.car
        .camera_pitch
        .set((c.car.camera_pitch.get() << 4).wrapping_neg());
    let w = [dx, dy, dz];
    x87_2(c.numbers, c.view_records, w);
    c.view_records.view_vars.camera_distance.set(rounded_value({
        crate::physics::vector_length::vector_length(&c.view_records.view_vars.camera_vector)
    }));
    c.car.camera_zoom.set(c.camera.camera_zoom_kept.get());
    if record(c.view_records, c.car)?.fixed_zoom.get() != 0 {
        x87_3(c.car, c.numbers, c.view_records, 0x180);
        hold_low(&c.car.camera_zoom, &c.numbers.k384);
    }
    c.car.camera_roll_turn.set(0);
    for turn in &c.car.camera_offset_turns {
        turn.set(0);
    }
    Ok(())
}

/// The steps.
// Core: `rc2k_core::physics::camera_view::{fixed_camera, keeps_light_figure, switch_over,
// mode_after_switch, check_modes}` (the x87 sums stay).
pub fn steps(c: &CameraInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    crate::physics::camera_low_switch::camera_low_switch(&c.ground)?;
    for k in 0..3 {
        c.frame_work.camera_car[k].set(c.car.position[k].get());
    }
    let car = c.car.car_kind.get();
    // The car kind's light record (a kind one past the 34 would read on into the gear figures:
    // refused).
    let head = c
        .light_records
        .records
        .get(car as usize)
        .ok_or(crate::errors::BadInput(
            "a car kind past the 34 light records",
        ))?
        .head
        .each_ref()
        .map(crate::memory::session::PlainDword::get);
    x87_4(c.tables, c.numbers, c.view_records, &head);
    ViewVars::record(c.view_records, 3).angles[0].set(head[3]);
    ViewVars::record(c.view_records, 3).angles[0].set(head[7]);
    if !camera_view::keeps_light_figure(car, || c.kind_tables.kind_has_views[car as usize].get()) {
        ViewVars::record(c.view_records, 1).focus[2]
            .set(crate::math::x87::loaded(head[10]).to_bits());
    }
    if c.car.camera_switch_active.get() == 0
        && let Some(second) = camera_view::fixed_camera(c.car.camera_view.get())
    {
        fixed_camera(c, second)?;
        return Ok(());
    }
    // The camera at the car, the views blended.
    for (k, copy) in [
        &c.course.course.get().place_x,
        &c.course.course.get().place_y,
        &c.course.course.get().place_z,
    ]
    .into_iter()
    .enumerate()
    {
        let v = c.car.position[k].get();
        copy.set(v);
        c.car.camera_position[k].set(v);
    }
    c.course
        .course
        .get()
        .place_heading
        .set(c.car.car_heading.get());
    if c.car.camera_switch_active.get() != 0 {
        if camera_view::switch_over(
            c.car.step_clock.get(),
            c.car.camera_switch_clock.get(),
            c.car.camera_switch_length.get(),
        ) {
            c.car.camera_switch_active.set(0);
            c.car.camera_switch_share.set(0.0);
            c.car.camera_view.set(c.car.camera_blend_view.get());
            c.car.value_jump_steps.set(camera_view::SWITCH_HOLD);
            let after = camera_view::mode_after_switch(c.car.camera_view.get());
            if after != c.car.camera_view.get() {
                c.car.camera_view.set(after);
            }
        } else {
            x87_5(c.car);
            hold_high(&c.car.camera_switch_share, &c.numbers.one);
            hold_low_bits(&c.car.camera_switch_share, ZERO);
        }
    }
    let (mode, before) = (c.car.camera_view.get(), c.car.camera_blend_view.get());
    camera_view::check_modes(mode, before)?;
    crate::physics::take_first_view::take_first_view(
        c.camera,
        c.car,
        c.view_records,
        ViewVars::record_of(c.view_records, mode as usize)?,
        scene,
    )?;
    crate::physics::take_second_view::take_second_view(
        c.camera,
        c.car,
        ViewVars::record_of(c.view_records, before as usize)?,
        scene,
    );
    x87_6(c.camera, c.car);
    // The offsets and turns: (to, from b, from a) each a + (b - a) x share, x 65536.
    let (work, offset_turns) = (&c.camera, &c.car.camera_offset_turns);
    let rows: [(&dyn DwordLike, &PlainFloat, &PlainFloat); 9] = [
        (
            &work.camera_offset_x,
            &work.view_b_offset[0],
            &work.view_a_offset[0],
        ),
        (
            &work.camera_offset_y,
            &work.view_b_offset[1],
            &work.view_a_offset[1],
        ),
        (
            &work.camera_offset_z,
            &work.view_b_offset[2],
            &work.view_a_offset[2],
        ),
        (
            &work.view_turn_x,
            &work.view_b_turn[0],
            &work.view_a_turn[0],
        ),
        (
            &work.view_turn_y,
            &work.view_b_turn[1],
            &work.view_a_turn[1],
        ),
        (
            &work.view_turn_z,
            &work.view_b_turn[2],
            &work.view_a_turn[2],
        ),
        (
            &offset_turns[0],
            &work.view_b_point[0],
            &work.view_a_point[0],
        ),
        (
            &offset_turns[1],
            &work.view_b_point[1],
            &work.view_a_point[1],
        ),
        (
            &offset_turns[2],
            &work.view_b_point[2],
            &work.view_a_point[2],
        ),
    ];
    for (to, x, y) in rows {
        x87_7(c.car, c.numbers, x, y, to);
    }
    for turn in &c.car.camera_offset_turns {
        turn.set(turn.get() << 16);
    }
    c.camera
        .view_share_pitch
        .set(c.camera.view_a_shares[0].get());
    c.camera
        .view_share_heading
        .set(c.camera.view_a_shares[1].get());
    c.camera
        .view_share_roll
        .set(c.camera.view_a_shares[2].get());
    c.camera.view_blend_aux.set(c.camera.view_a_aux.get());
    c.camera.view_blend_aux_2.set(c.camera.view_a_aux_2.get());
    x87_8(c.camera, c.car);
    hold_low(
        &c.car.camera_zoom,
        &c.view_records.view_vars.camera_zoom_hold,
    );
    crate::physics::ease_view_angles::ease_view_angles(c.numbers, c.shared, c.view_records)?;
    crate::physics::ease_view_blend::ease_view_blend(c)?;
    crate::physics::camera_avoid_ground::steps(c, scene)?;
    x87_9(c.camera, c.course.course.get());
    c.course
        .course
        .get()
        .camera_first
        .set(c.view_records.view_vars.view_angles[1].get());
    Ok(())
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [camera_heading] = ((atan2(int [v + 0], int [v + 4]) * [k65536]) * [k4096]) *
///  [one_over_two_pi]` (a long formula: checked by the tests' `reference_vectors`)
fn x87_1(car: &Car, numbers: &Numbers, v: [u32; 3]) {
    let t0 = int(v[0] as i32);
    let t1 = int(v[1] as i32);
    let t2 = t0.atan2(t1);
    let t3 = numbers.k65536.ld();
    let t4 = t2 * t3;
    let t5 = numbers.k4096.ld();
    let t6 = t4 * t5;
    let t7 = numbers.one_over_two_pi.ld();
    let t8 = t6 * t7;
    car.camera_heading.set(to_int(t8) as u32);
    let t9 = int(v[2] as i32);
    let t10 = int(v[0] as i32);
    let t11 = t10 * t10;
    let t12 = int(v[1] as i32);
    let t13 = t12 * t12;
    let t14 = t11 + t13;
    let t15 = t14.sqrt();
    let t16 = t9.atan2(t15);
    let t17 = numbers.k65536.ld();
    let t18 = t16 * t17;
    let t19 = numbers.k4096.ld();
    let t20 = t18 * t19;
    let t21 = numbers.one_over_two_pi.ld();
    let t22 = t20 * t21;
    car.camera_pitch.set(to_int(t22) as u32);
}

/// One straight x87 run as plain Rust over `X`.
fn x87_2(numbers: &Numbers, view_records: &ViewAndRecords, v: [u32; 3]) {
    let t0 = int(v[0] as i32);
    let t1 = numbers.k65536.ld();
    let t2 = t0 / t1;
    view_records.view_vars.camera_vector[0].st(t2);
    let t3 = int(v[1] as i32);
    let t4 = numbers.k65536.ld();
    let t5 = t3 / t4;
    view_records.view_vars.camera_vector[1].st(t5);
    let t6 = int(v[2] as i32);
    let t7 = numbers.k65536.ld();
    let t8 = t6 / t7;
    view_records.view_vars.camera_vector[2].st(t8);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[camera_zoom] = (int [camera_zoom] * [camera_distance]) / [five]`
fn x87_3(car: &Car, numbers: &Numbers, view_records: &ViewAndRecords, zoom: i32) {
    let t0 = int(zoom);
    let t1 = view_records.view_vars.camera_distance.ld();
    let t2 = t0 * t1;
    let t3 = numbers.five.ld();
    let t4 = t2 / t3;
    car.camera_zoom.st(t4);
}

/// One straight x87 run as plain Rust over `X`.
fn x87_4(tables: &PhysicsTables, numbers: &Numbers, view_records: &ViewAndRecords, d: &[u32; 11]) {
    let t0 = int(d[0] as i32);
    let t1 = numbers.one_over_65536.ld();
    let t2 = t0 * t1;
    ViewVars::record(view_records, 2).focus[0].st(t2);
    let t3 = int(d[1] as i32);
    let t4 = numbers.one_over_65536.ld();
    let t5 = t3 * t4;
    let t6 = tables.camera_drop.ld();
    let t7 = t5 - t6;
    ViewVars::record(view_records, 2).focus[1].st(t7);
    let t8 = int(d[2] as i32);
    let t9 = numbers.one_over_65536.ld();
    let t10 = t8 * t9;
    ViewVars::record(view_records, 2).focus[2].st(t10);
    let t11 = int(d[4] as i32);
    let t12 = numbers.one_over_65536.ld();
    let t13 = t11 * t12;
    ViewVars::record(view_records, 3).focus[0].st(t13);
    let t14 = int(d[5] as i32);
    let t15 = numbers.one_over_65536.ld();
    let t16 = t14 * t15;
    let t17 = tables.camera_drop.ld();
    let t18 = t16 - t17;
    ViewVars::record(view_records, 3).focus[1].st(t18);
    let t19 = int(d[6] as i32);
    let t20 = numbers.one_over_65536.ld();
    let t21 = t19 * t20;
    ViewVars::record(view_records, 3).focus[2].st(t21);
    let t22 = ld(d[9]);
    ViewVars::record(view_records, 1).focus[1].st(t22);
    let t23 = int(d[8] as i32);
    let t24 = numbers.one_over_65536.ld();
    let t25 = t23 * t24;
    ViewVars::record(view_records, 1).focus[2].st(t25);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[camera_switch_share] = (int [step_clock] - int [camera_switch_clock]) / int
///  [camera_switch_length]`
fn x87_5(car: &Car) {
    let t0 = int(car.step_clock.get() as i32);
    let t1 = int(car.camera_switch_clock.get() as i32);
    let t2 = t0 - t1;
    let t3 = int(car.camera_switch_length.get() as i32);
    let t4 = t2 / t3;
    car.camera_switch_share.st(t4);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [start_pitch] = ((int [view_b_pitch] - int [view_a_pitch]) * [camera_switch_share]) + int
///  [view_a_pitch]`
/// `int [start_heading] = ((int [view_b_heading] - int [view_a_heading]) * [camera_switch_share]) +
///  int [view_a_heading]`
/// `int [start_roll] = ((int [view_b_roll] - int [view_a_roll]) * [camera_switch_share]) + int
///  [view_a_roll]`
fn x87_6(camera: &CameraWork, car: &Car) {
    let t0 = int(camera.view_b_pitch.get());
    let t1 = int(camera.view_a_pitch.get());
    let t2 = t0 - t1;
    let t3 = car.camera_switch_share.ld();
    let t4 = t2 * t3;
    let t5 = int(camera.view_a_pitch.get());
    let t6 = t4 + t5;
    camera.start_pitch.set(to_int(t6) as u32);
    let t7 = int(camera.view_b_heading.get() as i32);
    let t8 = int(camera.view_a_heading.get() as i32);
    let t9 = t7 - t8;
    let t10 = car.camera_switch_share.ld();
    let t11 = t9 * t10;
    let t12 = int(camera.view_a_heading.get() as i32);
    let t13 = t11 + t12;
    camera.start_heading.set(to_int(t13) as u32);
    let t14 = int(camera.view_b_roll.get());
    let t15 = int(camera.view_a_roll.get());
    let t16 = t14 - t15;
    let t17 = car.camera_switch_share.ld();
    let t18 = t16 * t17;
    let t19 = int(camera.view_a_roll.get());
    let t20 = t18 + t19;
    camera.start_roll.set(to_int(t20) as u32);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [t + 0] = ((([x + 0] - [y + 0]) * [camera_switch_share]) + [y + 0]) * [k65536]`
fn x87_7(car: &Car, numbers: &Numbers, x: &PlainFloat, y: &PlainFloat, t: &dyn DwordLike) {
    let t0 = x.ld();
    let t1 = y.ld();
    let t2 = t0 - t1;
    let t3 = car.camera_switch_share.ld();
    let t4 = t2 * t3;
    let t5 = y.ld();
    let t6 = t4 + t5;
    let t7 = numbers.k65536.ld();
    let t8 = t6 * t7;
    t.set(to_int(t8) as u32);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[camera_zoom] = int [view_blend_aux_2]`
fn x87_8(camera: &CameraWork, car: &Car) {
    let t0 = int(camera.view_blend_aux_2.get() as i32);
    car.camera_zoom.st(t0);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables named below (`[name]`: a
/// float; `int [..]`: a whole number; `ext [..]`: 80 bits): A long formula, covered by the tests.
fn x87_9(camera: &CameraWork, course_work: &CourseWork) {
    let t0 = int(camera.camera_offset_x.get() as i32);
    let t1 = t0 * t0;
    let t2 = int(camera.camera_offset_y.get() as i32);
    let t3 = t2 * t2;
    let t4 = int(camera.camera_offset_z.get() as i32);
    let t5 = t4 * t4;
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    let t8 = t7.sqrt();
    course_work.flat_depth.set(to_int(t8) as u32);
}

#[cfg(test)]
mod tests;
