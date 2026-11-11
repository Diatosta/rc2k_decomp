//! `physics::update_camera` (0x437A53, two chunks): the camera's step. `physics::camera_low_switch`
//! runs; the car's place goes into `camera_car`; then the car kind's light record (`g_lightRecords`, 0x134 bytes each: places x 1/65536, two of them less `camera_drop`, into the camera's light places;
//! the words +0xC then +0x1C into the light colour figure; +0x24 into the light strength; +0x20 x 1/65536,
//! kept for the cars marked in `kind_has_views` (n below 0x22), else +0x28, into the second
//! strength). With no switch under way and a fixed mode (0x2A .. 0x2D, 9, 10:
//! `g_nearestCamera2` / `g_replayCameras2` of the car's `checkpoint`; 0x2E .. 0x31, 7, 8: the first
//! two) the second chunk runs: the camera is put at that face's first scene point,
//! `physics::camera_height_bias` runs, and the camera is aimed at the car (arctangent heading and
//! pitch, x 65536 x `k4096` x `one_over_two_pi`, << 4, the pitch negated); the distance
//! (`physics::vector_length`) goes into `camera_distance`; the zoom `camera_zoom` is
//! `camera_zoom_kept`, or 0x180 x the distance / `five` held at least `k384` with the record's
//! +0x50; the roll and turns are 0. Else the camera is put at the car (also into `place_x` ..,
//! the heading into `place_heading`); a switch under way ends after `camera_switch_length` steps
//! (the mode back, `value_jump_steps` = 8, modes 7 on to 0) or its share (held 0 ..= 1) goes
//! into `camera_switch_share`; `physics::take_first_view` of the mode and `take_second_view` of the
//! mode before run; their figures are blended by the share into the turn targets (`start_heading`
//! ..), the offsets (`camera_offset_x` ..) and the turns (`camera_offset_turns` .., << 16); the
//! shares (`view_blend_aux` ..); the zoom held at least `camera_zoom_hold`;
//! `physics::ease_view_angles`, `ease_view_blend` and `physics::camera_avoid_ground` run; the
//! offset's length goes into `flat_depth` and the first camera figure into `camera_first`.
//!
//! Caller: `physics::camera_frame`. Refuses a mode past 0x33 on the blending path (the
//! original reads past `view_records`), and what the ground checks refuse; everything written
//! put back.

use crate::errors::Refused;
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::math::x87_float::to_int;
use crate::physics::camera_view::record;
use crate::physics::flow::{ZERO_BITS as ZERO, hold_high, hold_low, hold_low_bits};
use crate::race::checkpoints_store::CHECKPOINTS;
use crate::race::kind_store::KIND_TABLES;
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::race::owner::Race;
use crate::race::view_vars_store::ViewVars;
use crate::scene::matrix3::rounded;

use crate::session::Cell32;
use rc2k_core::physics::camera_view;

/// The fixed camera (the second chunk, 0x4380E9 ..): `second` the second tables.
fn fixed_camera(
    race: &Race,
    second: bool,
    scene: &crate::scene::owner::Scene,
) -> Result<(), crate::errors::BadInput> {
    let race_owner = race;
    let (nearest, replay) = if second {
        (
            &CHECKPOINTS.nearest_camera_2[0],
            &CHECKPOINTS.replay_cameras_2[0],
        )
    } else {
        (
            &CHECKPOINTS.nearest_camera_1[0],
            &CHECKPOINTS.replay_cameras_1[0],
        )
    };
    let face = crate::race::checkpoints_store::nth(
        replay,
        crate::race::checkpoints_store::nth(nearest, race.shared.car.checkpoint.get() as usize)
            .get() as usize,
    )
    .get();
    let point = crate::race::course_places::face_field::<u32>(&race.course, face, 4)
        .unwrap_or_else(|e| crate::fatal::raise::end("a checkpoint face", e));
    for k in 0..3 {
        race.shared.car.camera_position[k as usize].set(
            crate::race::scene_points::point_word(&race.course, point, k)
                .unwrap_or_else(|e| crate::fatal::raise::end("a scene point", e)),
        );
    }
    crate::physics::camera_height_bias::camera_height_bias(race, scene)?;
    let d = |k: usize| {
        race.shared.car.position[k]
            .get()
            .wrapping_sub(race.shared.car.camera_position[k].get())
    };
    let (dx, dy, dz) = (d(0), d(1), d(2));
    let v = [dx, dz, dy];
    x87_1(race, v);
    race.shared
        .car
        .camera_heading
        .set(race.shared.car.camera_heading.get() << 4);
    race.shared
        .car
        .camera_pitch
        .set((race.shared.car.camera_pitch.get() << 4).wrapping_neg());
    let w = [dx, dy, dz];
    x87_2(race, w);
    scene.drawing.scene.scratch.set(dz);
    race_owner
        .frame
        .view_and_records
        .view_vars
        .camera_distance
        .set_bits(rounded({
            crate::physics::vector_length::vector_length(
                &race_owner.frame.view_and_records.view_vars.camera_vector,
            )
        }));
    race.shared
        .car
        .camera_zoom
        .set_bits(race_owner.physics.camera.camera_zoom_kept.bits());
    if record(&race.shared.car)?.fixed_zoom.get() != 0 {
        race.shared.car.camera_zoom.set_bits(0x180);
        x87_3(race);
        hold_low(
            &race.shared.car.camera_zoom,
            &race_owner.course.numbers.k384,
        );
    }
    race.shared.car.camera_roll_turn.set(0);
    for turn in &race.shared.car.camera_offset_turns {
        turn.set(0);
    }
    Ok(())
}

/// The steps.
// Converted: `rc2k_core::physics::camera_view::{fixed_camera, keeps_light_figure,
// switch_over, mode_after_switch, check_modes}` (the x87 sums stay).
pub fn steps(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    crate::physics::camera_low_switch::camera_low_switch(race, scene)?;
    for k in 0..3 {
        race.frame.frame_work.camera_car[k].set(race.shared.car.position[k].get());
    }
    let car = race.shared.car.car_kind.get();
    // The car kind's light record (the original does not bound the kind; one past the 34
    // would read on into the gear figures, refused here).
    let head = LIGHT_RECORDS
        .records
        .get(car as usize)
        .ok_or(crate::errors::BadInput(
            "a car kind past the 34 light records",
        ))?
        .head
        .each_ref()
        .map(crate::session::Dword::get);
    x87_4(race, &head);
    ViewVars::record(3).angles[0].set(head[3]);
    ViewVars::record(3).angles[0].set(head[7]);
    if !camera_view::keeps_light_figure(car, || KIND_TABLES.kind_has_views[car as usize].get()) {
        ViewVars::record(1).focus[2].set_bits(crate::math::x87::loaded(head[10]));
    }
    if race.shared.car.camera_switch_active.get() == 0
        && let Some(second) = camera_view::fixed_camera(race.shared.car.camera_view.get())
    {
        fixed_camera(race, second, scene)?;
        return Ok(());
    }
    // 0x437BE4: the camera at the car, the views blended.
    for (k, copy) in [
        &race.course.course.place_x,
        &race.course.course.place_y,
        &race.course.course.place_z,
    ]
    .into_iter()
    .enumerate()
    {
        let v = race.shared.car.position[k].get();
        copy.set(v);
        race.shared.car.camera_position[k].set(v);
    }
    race.course
        .course
        .place_heading
        .set(race.shared.car.car_heading.get());
    if race.shared.car.camera_switch_active.get() != 0 {
        if camera_view::switch_over(
            race.shared.car.step_clock.get(),
            race.shared.car.camera_switch_clock.get(),
            race.shared.car.camera_switch_length.get(),
        ) {
            race.shared.car.camera_switch_active.set(0);
            race.shared.car.camera_switch_share.set_bits(0);
            race.shared
                .car
                .camera_view
                .set(race.shared.car.camera_blend_view.get());
            race.shared
                .car
                .value_jump_steps
                .set(camera_view::SWITCH_HOLD);
            let after = camera_view::mode_after_switch(race.shared.car.camera_view.get());
            if after != race.shared.car.camera_view.get() {
                race.shared.car.camera_view.set(after);
            }
        } else {
            x87_5(race);
            hold_high(
                &race.shared.car.camera_switch_share,
                &race.course.numbers.one,
            );
            hold_low_bits(&race.shared.car.camera_switch_share, ZERO);
        }
    }
    let (mode, before) = (
        race.shared.car.camera_view.get(),
        race.shared.car.camera_blend_view.get(),
    );
    camera_view::check_modes(mode, before)?;
    crate::physics::take_first_view::take_first_view(
        race,
        ViewVars::record_of(mode as usize)?,
        scene,
    )?;
    crate::physics::take_second_view::take_second_view(
        race,
        ViewVars::record_of(before as usize)?,
        scene,
    );
    x87_6(race);
    // The offsets and turns: (to, from b, from a) each a + (b - a) x share, x 65536.
    let (work, offset_turns) = (&race.physics.camera, &race.shared.car.camera_offset_turns);
    let rows: [(&dyn Cell32, &dyn Cell32, &dyn Cell32); 9] = [
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
        x87_7(race, x, y, to);
    }
    for turn in &race.shared.car.camera_offset_turns {
        turn.set(turn.get() << 16);
    }
    race.physics
        .camera
        .view_share_pitch
        .set_bits(race.physics.camera.view_a_shares[0].bits());
    race.physics
        .camera
        .view_share_heading
        .set_bits(race.physics.camera.view_a_shares[1].bits());
    race.physics
        .camera
        .view_share_roll
        .set_bits(race.physics.camera.view_a_shares[2].bits());
    race.physics
        .camera
        .view_blend_aux
        .set(race.physics.camera.view_a_aux.get());
    race.physics
        .camera
        .view_blend_aux_2
        .set(race.physics.camera.view_a_aux_2.get());
    x87_8(race);
    hold_low(
        &race.shared.car.camera_zoom,
        &race.frame.view_and_records.view_vars.camera_zoom_hold,
    );
    crate::physics::ease_view_angles::ease_view_angles(race, scene)?;
    crate::physics::ease_view_blend::ease_view_blend(race, scene)?;
    crate::physics::camera_avoid_ground::steps(race, scene)?;
    x87_9(race);
    race.course
        .course
        .camera_first
        .set(race.frame.view_and_records.view_vars.view_angles[1].get());
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [camera_heading] = ((atan2(int [v + 0], int [v + 4]) * [k65536]) * [k4096]) * [one_over_two_pi]`
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn x87_1(race: &Race, v: [u32; 3]) {
    let t0 = int(v[0] as i32);
    let t1 = int(v[1] as i32);
    let t2 = t0.atan2(t1);
    let t3 = ld(race.course.numbers.k65536.bits());
    let t4 = t2 * t3;
    let t5 = ld(race.course.numbers.k4096.bits());
    let t6 = t4 * t5;
    let t7 = ld(race.course.numbers.one_over_two_pi.bits());
    let t8 = t6 * t7;
    race.shared.car.camera_heading.set_bits(to_int(t8) as u32);
    let t9 = int(v[2] as i32);
    let t10 = int(v[0] as i32);
    let t11 = t10 * t10;
    let t12 = int(v[1] as i32);
    let t13 = t12 * t12;
    let t14 = t11 + t13;
    let t15 = t14.sqrt();
    let t16 = t9.atan2(t15);
    let t17 = ld(race.course.numbers.k65536.bits());
    let t18 = t16 * t17;
    let t19 = ld(race.course.numbers.k4096.bits());
    let t20 = t18 * t19;
    let t21 = ld(race.course.numbers.one_over_two_pi.bits());
    let t22 = t20 * t21;
    race.shared.car.camera_pitch.set_bits(to_int(t22) as u32);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[camera_vector] = int [v + 0] / [k65536]`
/// `[rc + 0x2062DC] = int [v + 4] / [k65536]`
/// `[rc + 0x2062E0] = int [v + 8] / [k65536]`
fn x87_2(race: &Race, v: [u32; 3]) {
    let t0 = int(v[0] as i32);
    let t1 = ld(race.course.numbers.k65536.bits());
    let t2 = t0 / t1;
    race.frame.view_and_records.view_vars.camera_vector[0].set_bits(st(t2));
    let t3 = int(v[1] as i32);
    let t4 = ld(race.course.numbers.k65536.bits());
    let t5 = t3 / t4;
    race.frame.view_and_records.view_vars.camera_vector[1].set_bits(st(t5));
    let t6 = int(v[2] as i32);
    let t7 = ld(race.course.numbers.k65536.bits());
    let t8 = t6 / t7;
    race.frame.view_and_records.view_vars.camera_vector[2].set_bits(st(t8));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[camera_zoom] = (int [camera_zoom] * [camera_distance]) / [five]`
fn x87_3(race: &Race) {
    let t0 = int(race.shared.car.camera_zoom.bits() as i32);
    let t1 = ld(race.frame.view_and_records.view_vars.camera_distance.bits());
    let t2 = t0 * t1;
    let t3 = ld(race.course.numbers.five.bits());
    let t4 = t2 / t3;
    race.shared.car.camera_zoom.set_bits(st(t4));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2065C8] = int [d + 0] * [one_over_65536]`
/// `[rc + 0x2065CC] = (int [d + 4] * [one_over_65536]) - [camera_drop]`
/// `[rc + 0x2065D0] = int [d + 8] * [one_over_65536]`
/// `[rc + 0x206624] = int [d + 0x10] * [one_over_65536]`
/// `[rc + 0x206628] = (int [d + 0x14] * [one_over_65536]) - [camera_drop]`
/// `[rc + 0x20662C] = int [d + 0x18] * [one_over_65536]`
/// `[rc + 0x206570] = [d + 0x24]`
/// `[rc + 0x206574] = int [d + 0x20] * [one_over_65536]`
fn x87_4(race: &Race, d: &[u32; 11]) {
    let t0 = int(d[0] as i32);
    let t1 = ld(race.course.numbers.one_over_65536.bits());
    let t2 = t0 * t1;
    ViewVars::record(2).focus[0].set_bits(st(t2));
    let t3 = int(d[1] as i32);
    let t4 = ld(race.course.numbers.one_over_65536.bits());
    let t5 = t3 * t4;
    let t6 = ld(race.shared.physics_tables.camera_drop.bits());
    let t7 = t5 - t6;
    ViewVars::record(2).focus[1].set_bits(st(t7));
    let t8 = int(d[2] as i32);
    let t9 = ld(race.course.numbers.one_over_65536.bits());
    let t10 = t8 * t9;
    ViewVars::record(2).focus[2].set_bits(st(t10));
    let t11 = int(d[4] as i32);
    let t12 = ld(race.course.numbers.one_over_65536.bits());
    let t13 = t11 * t12;
    ViewVars::record(3).focus[0].set_bits(st(t13));
    let t14 = int(d[5] as i32);
    let t15 = ld(race.course.numbers.one_over_65536.bits());
    let t16 = t14 * t15;
    let t17 = ld(race.shared.physics_tables.camera_drop.bits());
    let t18 = t16 - t17;
    ViewVars::record(3).focus[1].set_bits(st(t18));
    let t19 = int(d[6] as i32);
    let t20 = ld(race.course.numbers.one_over_65536.bits());
    let t21 = t19 * t20;
    ViewVars::record(3).focus[2].set_bits(st(t21));
    let t22 = ld(d[9]);
    ViewVars::record(1).focus[1].set_bits(st(t22));
    let t23 = int(d[8] as i32);
    let t24 = ld(race.course.numbers.one_over_65536.bits());
    let t25 = t23 * t24;
    ViewVars::record(1).focus[2].set_bits(st(t25));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[camera_switch_share] = (int [step_clock] - int [camera_switch_clock]) / int [camera_switch_length]`
fn x87_5(race: &Race) {
    let t0 = int(race.shared.car.step_clock.bits() as i32);
    let t1 = int(race.shared.car.camera_switch_clock.bits() as i32);
    let t2 = t0 - t1;
    let t3 = int(race.shared.car.camera_switch_length.bits() as i32);
    let t4 = t2 / t3;
    race.shared.car.camera_switch_share.set_bits(st(t4));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [start_pitch] = ((int [view_b_pitch] - int [view_a_pitch]) * [camera_switch_share]) + int [view_a_pitch]`
/// `int [start_heading] = ((int [view_b_heading] - int [view_a_heading]) * [camera_switch_share]) + int [view_a_heading]`
/// `int [start_roll] = ((int [view_b_roll] - int [view_a_roll]) * [camera_switch_share]) + int [view_a_roll]`
fn x87_6(race: &Race) {
    let t0 = int(race.physics.camera.view_b_pitch.bits() as i32);
    let t1 = int(race.physics.camera.view_a_pitch.bits() as i32);
    let t2 = t0 - t1;
    let t3 = ld(race.shared.car.camera_switch_share.bits());
    let t4 = t2 * t3;
    let t5 = int(race.physics.camera.view_a_pitch.bits() as i32);
    let t6 = t4 + t5;
    race.physics.camera.start_pitch.set_bits(to_int(t6) as u32);
    let t7 = int(race.physics.camera.view_b_heading.bits() as i32);
    let t8 = int(race.physics.camera.view_a_heading.bits() as i32);
    let t9 = t7 - t8;
    let t10 = ld(race.shared.car.camera_switch_share.bits());
    let t11 = t9 * t10;
    let t12 = int(race.physics.camera.view_a_heading.bits() as i32);
    let t13 = t11 + t12;
    race.physics
        .camera
        .start_heading
        .set_bits(to_int(t13) as u32);
    let t14 = int(race.physics.camera.view_b_roll.bits() as i32);
    let t15 = int(race.physics.camera.view_a_roll.bits() as i32);
    let t16 = t14 - t15;
    let t17 = ld(race.shared.car.camera_switch_share.bits());
    let t18 = t16 * t17;
    let t19 = int(race.physics.camera.view_a_roll.bits() as i32);
    let t20 = t18 + t19;
    race.physics.camera.start_roll.set_bits(to_int(t20) as u32);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [t + 0] = ((([x + 0] - [y + 0]) * [camera_switch_share]) + [y + 0]) * [k65536]`
fn x87_7(race: &Race, x: &dyn Cell32, y: &dyn Cell32, t: &dyn Cell32) {
    let t0 = ld(x.bits());
    let t1 = ld(y.bits());
    let t2 = t0 - t1;
    let t3 = ld(race.shared.car.camera_switch_share.bits());
    let t4 = t2 * t3;
    let t5 = ld(y.bits());
    let t6 = t4 + t5;
    let t7 = ld(race.course.numbers.k65536.bits());
    let t8 = t6 * t7;
    t.set_bits(to_int(t8) as u32);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[camera_zoom] = int [view_blend_aux_2]`
fn x87_8(race: &Race) {
    let t0 = int(race.physics.camera.view_blend_aux_2.bits() as i32);
    race.shared.car.camera_zoom.set_bits(st(t0));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn x87_9(race: &Race) {
    let t0 = int(race.physics.camera.camera_offset_x.bits() as i32);
    let t1 = t0 * t0;
    let t2 = int(race.physics.camera.camera_offset_y.bits() as i32);
    let t3 = t2 * t2;
    let t4 = int(race.physics.camera.camera_offset_z.bits() as i32);
    let t5 = t4 * t4;
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    let t8 = t7.sqrt();
    race.course.course.flat_depth.set_bits(to_int(t8) as u32);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::view;

    // The original's instructions, one function per run.
    reference!(ref_0(v, rc) "fild dword ptr [{v}]", "fild dword ptr [{v} + 4]", ".byte 0xD9, 0xF3", "fmul dword ptr [{rc} + 0x1C5D98]", "fmul dword ptr [{rc} + 0x1C5E28]", "fmul dword ptr [{rc} + 0x1C5DD4]", "fistp dword ptr [{rc} + 0x31A688]", "fild dword ptr [{v} + 8]", "fild dword ptr [{v}]", ".byte 0xD8, 0xC8", "fild dword ptr [{v} + 4]", ".byte 0xD8, 0xC8", ".byte 0xDE, 0xC1", ".byte 0xD9, 0xFA", ".byte 0xD9, 0xF3", "fmul dword ptr [{rc} + 0x1C5D98]", "fmul dword ptr [{rc} + 0x1C5E28]", "fmul dword ptr [{rc} + 0x1C5DD4]", "fistp dword ptr [{rc} + 0x31A690]");
    reference!(ref_1(v, rc) "fild dword ptr [{v}]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fstp dword ptr [{rc} + 0x2062D8]", "fild dword ptr [{v} + 4]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fstp dword ptr [{rc} + 0x2062DC]", "fild dword ptr [{v} + 8]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fstp dword ptr [{rc} + 0x2062E0]");
    reference!(ref_2(rc) "fild dword ptr [{rc} + 0x31A6A0]", "fmul dword ptr [{rc} + 0x20631C]", "fdiv dword ptr [{rc} + 0x1C5D48]", "fstp dword ptr [{rc} + 0x31A6A0]");
    reference!(ref_3(d, rc) "fild dword ptr [{d}]", "fmul dword ptr [{rc} + 0x1C5DA0]", "fstp dword ptr [{rc} + 0x2065C8]", "fild dword ptr [{d} + 0x4]", "fmul dword ptr [{rc} + 0x1C5DA0]", "fsub dword ptr [{rc} + 0x213FC0]", "fstp dword ptr [{rc} + 0x2065CC]", "fild dword ptr [{d} + 0x8]", "fmul dword ptr [{rc} + 0x1C5DA0]", "fstp dword ptr [{rc} + 0x2065D0]", "fild dword ptr [{d} + 0x10]", "fmul dword ptr [{rc} + 0x1C5DA0]", "fstp dword ptr [{rc} + 0x206624]", "fild dword ptr [{d} + 0x14]", "fmul dword ptr [{rc} + 0x1C5DA0]", "fsub dword ptr [{rc} + 0x213FC0]", "fstp dword ptr [{rc} + 0x206628]", "fild dword ptr [{d} + 0x18]", "fmul dword ptr [{rc} + 0x1C5DA0]", "fstp dword ptr [{rc} + 0x20662C]", "fld dword ptr [{d} + 0x24]", "fstp dword ptr [{rc} + 0x206570]", "fild dword ptr [{d} + 0x20]", "fmul dword ptr [{rc} + 0x1C5DA0]", "fstp dword ptr [{rc} + 0x206574]");
    reference!(ref_4(rc) "fild dword ptr [{rc} + 0x319FE0]", "fisub dword ptr [{rc} + 0x31A668]", "fidiv dword ptr [{rc} + 0x31A670]", "fstp dword ptr [{rc} + 0x31A674]");
    reference!(ref_5(rc) "fild dword ptr [{rc} + 0x208B70]", "fisub dword ptr [{rc} + 0x208B64]", "fmul dword ptr [{rc} + 0x31A674]", "fiadd dword ptr [{rc} + 0x208B64]", "fistp dword ptr [{rc} + 0x208B58]", "fild dword ptr [{rc} + 0x208B68]", "fisub dword ptr [{rc} + 0x208B5C]", "fmul dword ptr [{rc} + 0x31A674]", "fiadd dword ptr [{rc} + 0x208B5C]", "fistp dword ptr [{rc} + 0x208B50]", "fild dword ptr [{rc} + 0x208B6C]", "fisub dword ptr [{rc} + 0x208B60]", "fmul dword ptr [{rc} + 0x31A674]", "fiadd dword ptr [{rc} + 0x208B60]", "fistp dword ptr [{rc} + 0x208B54]");
    reference!(ref_6(x, y, rc, t) "fld dword ptr [{x}]", "fsub dword ptr [{y}]", "fmul dword ptr [{rc} + 0x31A674]", "fadd dword ptr [{y}]", "fmul dword ptr [{rc} + 0x1C5D98]", "fistp dword ptr [{t}]");
    reference!(ref_7(rc) "fild dword ptr [{rc} + 0x208BF0]", "fstp dword ptr [{rc} + 0x31A6A0]");
    reference!(ref_8(rc) "fild dword ptr [{rc} + 0x208B30]", ".byte 0xD8, 0xC8", "fild dword ptr [{rc} + 0x208B34]", ".byte 0xD8, 0xC8", "fild dword ptr [{rc} + 0x208B38]", ".byte 0xD8, 0xC8", ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", ".byte 0xD9, 0xFA", "fistp dword ptr [{rc} + 0x1CCD80]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "update_camera line 64",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::I), (4, In::I), (8, In::I)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x1C5D98, In::F), (0x1C5DD4, In::F), (0x1C5E28, In::F)],
                        writes: &[(0x31A688, Out::I), (0x31A690, Out::I)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1]) },
                plain: |a| x87_1(race(), *view(a[0], 0)),
            },
            Block {
                name: "update_camera line 89",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::I), (4, In::I), (8, In::I)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x1C5D98, In::F)],
                        writes: &[(0x2062D8, Out::F), (0x2062DC, Out::F), (0x2062E0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| x87_2(crate::race::owner::race(), *view(a[0], 0)),
            },
            Block {
                name: "update_camera line 109",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x1C5D48, In::F), (0x20631C, In::F), (0x31A6A0, In::I)],
                    writes: &[(0x31A6A0, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0]) },
                plain: |_| x87_3(crate::race::owner::race()),
            },
            Block {
                name: "update_camera line 133",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0, In::I),
                            (4, In::I),
                            (8, In::I),
                            (0x10, In::I),
                            (0x14, In::I),
                            (0x18, In::I),
                            (0x20, In::I),
                            (0x24, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x1C5DA0, In::F), (0x213FC0, In::F)],
                        writes: &[
                            (0x206570, Out::F),
                            (0x206574, Out::F),
                            (0x2065C8, Out::F),
                            (0x2065CC, Out::F),
                            (0x2065D0, Out::F),
                            (0x206624, Out::F),
                            (0x206628, Out::F),
                            (0x20662C, Out::F),
                        ],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1]) },
                plain: |a| {
                    x87_4(
                        race(),
                        &core::array::from_fn(|k| {
                            if k < 10 {
                                crate::scene::x87_mem::Base::dword(a[0], 4 * k)
                            } else {
                                0
                            }
                        }),
                    );
                },
            },
            Block {
                name: "update_camera line 190",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x319FE0, In::I), (0x31A668, In::I), (0x31A670, In::I)],
                    writes: &[(0x31A674, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0]) },
                plain: |_| x87_5(race()),
            },
            Block {
                name: "update_camera line 205",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x208B5C, In::I),
                        (0x208B60, In::I),
                        (0x208B64, In::I),
                        (0x208B68, In::I),
                        (0x208B6C, In::I),
                        (0x208B70, In::I),
                        (0x31A674, In::F),
                    ],
                    writes: &[(0x208B50, Out::I), (0x208B54, Out::I), (0x208B58, Out::I)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_5(a[0]) },
                plain: |_| x87_6(race()),
            },
            Block {
                name: "update_camera line 235",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x1C5D98, In::F), (0x31A674, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::I)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_6(a[0], a[1], a[2], a[3]) },
                plain: |a| {
                    use crate::scene::x87_mem::HostCell;
                    x87_7(race(), &HostCell(a[0]), &HostCell(a[1]), &HostCell(a[3]));
                },
            },
            Block {
                name: "update_camera line 257",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x208BF0, In::I)],
                    writes: &[(0x31A6A0, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_7(a[0]) },
                plain: |_| x87_8(race()),
            },
            Block {
                name: "update_camera line 266",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x208B30, In::I), (0x208B34, In::I), (0x208B38, In::I)],
                    writes: &[(0x1CCD80, Out::I)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_8(a[0]) },
                plain: |_| x87_9(crate::race::owner::race()),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
