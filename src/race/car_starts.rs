//! Race init's first step, the cars' starts (`race::start_cars`, 0x446463, IDA
//! 2026-10-17): each car of the race loaded into the work copy (`g_car`), set at its start
//! (`race::start_car`: its checkpoint, its place on the line and to the side, the nearest
//! checkpoint, then `race::reset_car_motion`: settled on the ground by
//! `race::settle_car`, its matrix undone, its motion cleared), two blocks cleared, stored
//! back. `sub_43EB5D` and `sub_45D305` are stream B's `physics::set_handling` and
//! `physics::probe_ground` (since 2026-10-17).

use crate::errors::{BadInput, Refused};
use crate::physics::camera_store::CAMERA_WORK;
use crate::physics::car_store::CAR;
use crate::physics::contact_store::CONTACT_WORK;
use crate::physics::tables_store::PHYSICS;
use crate::race::best_line::MOST_CHECKS;
use crate::race::course_store::COURSE;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::scene::light_store::LIGHT_STATE;
use crate::session::Dword;

/// The most cars `g_carBlocks` has.
pub const MOST_CARS: u32 = 8;

fn add(cell: &Dword, by: u32) {
    cell.set(cell.get().wrapping_add(by));
}

fn zero(cells: &[&Dword]) {
    for cell in cells {
        cell.set(0);
    }
}

fn wheels() -> [u32; 4] {
    crate::session::read_first(&CONTACT_WORK.probe_height)
}

/// One bisection step of `settle_car`: `angle` moved by `step` towards the higher pair
/// (down when `low` - `high` is negative), the step halved (`shr`).
fn settle_step(low: u32, high: u32, angle: &Dword, step: &Dword) {
    let s = step.get();
    if (low.wrapping_sub(high) as i32) < 0 {
        angle.set(angle.get().wrapping_sub(s));
    } else {
        add(angle, s);
    }
    step.set(s >> 1);
}

/// The matrix (`get_mat_crash`) and the wheels (`physics::probe_ground`).
fn matrix_and_wheels() -> Result<[u32; 4], Refused> {
    crate::race::stage_prefs::get_mat_crash();
    crate::physics::probe_ground::probe_ground()?;
    Ok(wheels())
}

/// `race::settle_car` (0x43FC32): `sub_43EB5D` (`physics::set_handling`); the work car's
/// pitch and roll (+0x9C4, +0x9C8) 0, both steps 0x20000000, `dword_620470` 0x80; eight
/// times the matrix (`get_mat_crash`) and the wheels (`sub_45D305`), the roll moved by the
/// step towards wheels 0 + 2 against 1 + 3, the pitch towards 0 + 1 against 2 + 3, each
/// step halved; once more the matrix and the wheels, then the car's y, the four wheels and
/// the four at 0x6D1F14 lowered by the lowest wheel (from 0x640000). The registers it
/// leaves are read by no caller (conversion phase 3, 2026-10-24).
pub fn settle_car() -> Result<(), Refused> {
    crate::physics::set_handling::set_handling()?;
    CAR.car_pitch.set(0);
    CAR.car_roll.set(0);
    PHYSICS.settle_steps[0].set(0x2000_0000u32);
    PHYSICS.settle_steps[1].set(0x2000_0000u32);
    crate::physics::ground_probe_store::GROUND_PROBE
        .settle_mark
        .set(0x80);
    for _ in 0..8 {
        let w = matrix_and_wheels()?;
        settle_step(
            w[0].wrapping_add(w[2]),
            w[1].wrapping_add(w[3]),
            &CAR.car_roll,
            &PHYSICS.settle_steps[1],
        );
        settle_step(
            w[0].wrapping_add(w[1]),
            w[2].wrapping_add(w[3]),
            &CAR.car_pitch,
            &PHYSICS.settle_steps[0],
        );
    }
    let w = matrix_and_wheels()?;
    let mut lowest = 0x64_0000u32;
    for &h in &w {
        if (h.wrapping_sub(lowest) as i32) < 0 {
            lowest = h;
        }
    }
    CAR.position[1].set(CAR.position[1].get().wrapping_sub(lowest));
    // dword_6D2514 .. 20: the four wheels' heights (made by `sub_45D305`).
    for (k, h) in w.iter().enumerate() {
        CONTACT_WORK.probe_height[k].set(h.wrapping_sub(lowest));
    }
    // dword_6D1F14 .. 20: four more lowered with them.
    for cell in &CONTACT_WORK.probe_y[..4] {
        cell.set(cell.get().wrapping_sub(lowest));
    }
    Ok(())
}

/// `race::reset_car_motion` (0x43FB47): `settle_car`, `invert_car_matrix`; the work car's
/// +0x198 .. +0x1A0 0, +0x1A4 1.0, +0x1A8 0, `g_carSpeed` (+0x1AC) 0, +0x184 .. +0x18C 0;
/// heading, pitch and roll copied to `dword_608B50` / `58` / `54` and +0x6B0 / +0x6B8 /
/// +0x6B4; +0x1E8, `g_carDriveSpeed`, `g_carDriveSpeed2`, `g_carGear`, +0x200, +0x204,
/// +0x1C4, +0x1F4 0; with +0x68C not 0, +0x688 and +0x684 it less one and +0x68C 0.
pub fn reset_car_motion() -> Result<(), Refused> {
    settle_car()?;
    crate::race::start_place::invert_car_matrix();
    for v in &CAR.velocity {
        v.set_bits(0);
    }
    CAR.local_velocity[0].set_bits(0x3F80_0000);
    CAR.local_velocity[1].set_bits(0);
    CAR.local_velocity[2].set_bits(0);
    for v in &CAR.spin {
        v.set_bits(0);
    }
    let heading = CAR.car_heading.get();
    CAMERA_WORK.start_heading.set_bits(heading);
    CAR.camera_heading.set(heading);
    let pitch = CAR.car_pitch.get();
    CAMERA_WORK.start_pitch.set_bits(pitch);
    CAR.camera_pitch.set(pitch);
    let roll = CAR.car_roll.get();
    CAMERA_WORK.start_roll.set_bits(roll);
    CAR.camera_roll_turn.set(roll);
    CAR.stuck_count.set(0);
    CAR.drive_speed_a.set_bits(0);
    CAR.drive_speed_b.set_bits(0);
    CAR.gear.set(0);
    CAR.eased_yaw_spin.set_bits(0);
    CAR.grip_force_sum.set_bits(0);
    CAR.speed.set_bits(0);
    CAR.wall_hit_timer.set(0);
    let pending = CAR.camera_view_return.get();
    if pending != 0 {
        CAR.camera_blend_view.set(pending - 1);
        CAR.camera_view.set(pending - 1);
        CAR.camera_view_return.set(0);
    }
    Ok(())
}

/// `math::turn_point_xz` of (x, 0, z) by the start place's heading: x and z moved. The
/// original's y there is a leftover in ebx; it lands only in `g_scratchY`, which every
/// reader writes first.
fn move_turned(x: u32, z: u32) {
    let [x, _, z, _] = crate::math::turn_point_xz::turn_point_xz(
        x as i32,
        0,
        z as i32,
        COURSE.place_heading.bits(),
    );
    add(&CAR.position[0], x);
    add(&CAR.position[2], z);
}

/// `race::start_car` (0x4464AD): the work car's +0x478, +0x694 0, +0xAD4/+0xAD8
/// 0x80000000, +0x1E4, +0x1D8, +0x1D4 0, +0x644 .. +0x650 and +0x638 .. +0x640 0. Unless
/// `dword_62389C` is 4: its checkpoint `first_check_checkpoint` (less the car's number /
/// 2 but with `dword_62389C` 1 or 3) into `dword_6365E4`, `dword_6365EC` 0 (both 0 with
/// `dword_64BB18`); `place_car_on_line`; without `dword_64BB18` moved back (0, -0x20000
/// turned by `dword_5CCD60`) and, with `dword_52A910` or (`dword_62389C` 0 and
/// `dword_63DB94` not 1), to the side (+-0x14000 by the car's number's low bit). Then
/// `nearest_checkpoint` into +0x98, +0x94, +0x9C; its check count less one held to 0 ..
/// 100 into +0x90 and +0xAC; `reset_car_motion`; +0x264 .. +0x26C 0. The registers it
/// leaves are read by no caller. Refuses a nearest checkpoint past the lists (when met).
pub fn start_car() -> Result<(), Refused> {
    CAR.notice_count.set(0);
    CAR.camera_switch_active.set(0);
    CAR.map_corner[0].set(0x8000_0000);
    CAR.map_corner[1].set(0x8000_0000);
    CAR.engine_sound_revs.set(0);
    CAR.engine_speed.set_bits(0);
    CAR.gear.set(0);
    zero(&[
        &CAR.steering,
        &CAR.steering_raw,
        &CAR.steer_angle,
        &CAR.steer_angle_visual,
    ]);
    zero(&[&CAR.throttle, &CAR.brake, &CAR.handbrake]);
    if RACE_VARS.run_kind.bits() != 4 || crate::debug::screens::free_play_placed() {
        // Converted: `rc2k_core::race::start::{start_checkpoint, starts_aside,
        // side_offset, start_checks}`.
        use rc2k_core::race::start::{side_offset, start_checkpoint, starts_aside};
        let first = crate::race::race_init::first_check_checkpoint()?;
        let mode: u32 = RACE_VARS.run_kind.bits();
        let place = start_checkpoint(
            first,
            mode,
            crate::race::work_store::RACE_WORK.current_car.get(),
        );
        LIGHT_STATE.line_place.set(place);
        LIGHT_STATE.line_fraction.set(0);
        if crate::drive::finish_store::FINISH.no_place.get() != 0 {
            LIGHT_STATE.line_place.set(0);
            LIGHT_STATE.line_fraction.set(0);
        }
        crate::race::start_place::place_car_on_line()?;
        if crate::drive::finish_store::FINISH.no_place.get() == 0 {
            move_turned(0, 0xFFFE_0000);
            let side = starts_aside(
                crate::net::store::NET.session.get(),
                || RACE_VARS.run_kind.bits(),
                || RACE_VIEW.players.bits(),
            );
            if side {
                move_turned(
                    side_offset(crate::race::work_store::RACE_WORK.current_car.get()),
                    0,
                );
            }
        }
    }
    let nearest = crate::race::start_place::nearest_checkpoint()?;
    CAR.checkpoint.set(nearest);
    CAR.reset_checkpoint.set(nearest);
    CAR.last_pace_call.set(nearest);
    if nearest >= MOST_CHECKS {
        return Err(BadInput("a nearest checkpoint past the lists").into());
    }
    let checks = rc2k_core::race::start::start_checks(
        crate::race::checkpoints_store::nth(
            &crate::race::checkpoints_store::CHECKPOINTS.check_counts[0],
            nearest as usize,
        )
        .get(),
    );
    CAR.splits_passed.set(checks);
    CAR.splits_recorded.set(checks);
    reset_car_motion()?;
    zero(&[
        &CAR.reset_key_count,
        &CAR.stopped_steps,
        &CAR.tyre_change_state,
    ]);
    Ok(())
}

/// `race::start_cars` (0x446463): for each car of the race (`g_raceCars`), `g_currentCar`
/// it, `drive::load_car`, `start_car`, the work car's +0xA3C (8 dwords) and +0xA5C (0x11)
/// 0, `drive::store_car`; every register kept round each car (`pusha` / `popa`). Refuses
/// no cars (the original's `jnz` runs on) or more than 8, before anything.
pub fn start_cars() -> Result<(), Refused> {
    let cars: u32 = RACE_VIEW.race_cars.bits();
    if cars == 0 || cars > MOST_CARS {
        return Err(BadInput("no cars or more than eight").into());
    }
    for n in 0..cars {
        crate::race::work_store::RACE_WORK.current_car.set(n);
        crate::drive::load_car::load_car()?;
        start_car()?;
        // +0xA3C (8 dwords) and +0xA5C (0x11) 0.
        for cell in CAR.part_damage_limits.iter().chain(&CAR.unknown_a58[..18]) {
            cell.set(0);
        }
        crate::drive::store_car::store_car()?;
    }
    Ok(())
}
