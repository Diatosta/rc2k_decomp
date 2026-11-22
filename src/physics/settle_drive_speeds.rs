//! Brings driven speeds toward 0 by a step (`settle_fast_step`, 1.0, when `brake` is set, else
//! `settle_slow_step`, 0.01; stored in `scratch`), never past it: `drive_speed_a` when
//! `front_wheels_off` is set and the car is in neutral, `drive_speed_b` when `rear_wheels_off` is
//! set (whatever the gear). A speed of 0 (or NaN) is left; one that reaches or crosses 0 becomes +0
//! (a NaN after the step too). Then, in gear, `physics::revs_from_wheels`.
//!
//! Refuses (before anything) a gear outside -1 ..= 7 (`revs_from_wheels`' table).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::physics::figures::{SETTLE_FAST_STEP, SETTLE_SLOW_STEP};
use crate::physics::gearing::revs_from_wheels_at;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;

/// One speed `v` toward 0 by the step `step`.
fn settle(v: &mut f32, step: f32) {
    let x = *v;
    if x == 0.0 || x.is_nan() {
        return;
    }
    if x < 0.0 {
        let y = x87::add(x, step);
        *v = y;
        if y >= 0.0 {
            *v = 0.0;
        }
    } else {
        let y = x87::sub(x, step);
        *v = y;
        if y < 0.0 || y.is_nan() {
            *v = 0.0;
        }
    }
}

/// The exact version, for `physics::car_physics_step`.
pub(super) fn settle_drive_speeds(numbers: &Numbers, shared: &RaceShared) -> Result<(), Refused> {
    let ratios = &shared.physics_tables().gear_ratios.clone();
    let mut car = shared.car_mut();
    let gear = car.gear;
    if gear != 0 && !(-1..=7).contains(&gear) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    let step = if car.brake != 0 {
        SETTLE_FAST_STEP
    } else {
        SETTLE_SLOW_STEP
    };
    if car.front_wheels_off != 0 && gear == 0 {
        settle(&mut car.drive_speed_a, step);
    }
    if car.rear_wheels_off != 0 {
        settle(&mut car.drive_speed_b, step);
    }
    if gear != 0 {
        return revs_from_wheels_at(numbers, &mut car, ratios);
    }
    Ok(())
}
