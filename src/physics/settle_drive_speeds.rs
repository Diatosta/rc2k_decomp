//! `physics::settle_drive_speeds` (0x43BF6F): driven speeds brought toward 0 by a step
//! (`flt_613F3C`, 1.0, when `dword_71A614` is set, else `flt_613F40`, 0.01; stored in
//! `dword_62400C`), never past it: `g_carDriveSpeed` when `dword_71A220` is set and the car
//! is in neutral, `g_carDriveSpeed2` when `dword_71A224` is set (whatever the gear). A
//! speed of 0 (or NaN) is left; one that reaches or crosses 0 becomes +0 (a NaN after the
//! step too). Then, in gear, `physics::revs_from_wheels`.
//!
//! In:  nothing
//! Out: in gear, ebx = `g_carGear` (`revs_from_wheels`); the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B7AD, before `sub_43EA54`), every physics step. Refuses
//! (before anything) a gear outside -1 ..= 7 (`revs_from_wheels`' table).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::physics::gearing::{GearPlaces, revs_from_wheels_at};
use crate::race::owner_course::Course;
use crate::scene::owner::scene;
use crate::session::Cell32;

/// The switches and the scratch: the game's or a test's.
#[derive(Clone, Copy)]
pub struct SettlePlaces {
    /// `dword_71A220`: settle the first driven speed (in neutral).
    pub first: &'static dyn Cell32,
    /// `dword_71A224`: settle the second.
    pub second: &'static dyn Cell32,
    /// `dword_71A614`: the big step.
    pub fast: &'static dyn Cell32,
    /// `dword_62400C`.
    pub scratch: &'static dyn Cell32,
}

impl SettlePlaces {
    pub fn game() -> Self {
        Self {
            first: &crate::race::owner::race().shared.car.front_wheels_off,
            second: &crate::race::owner::race().shared.car.rear_wheels_off,
            fast: &crate::race::owner::race().shared.car.brake,
            scratch: &scene().drawing.scene.scratch,
        }
    }
}

/// One speed `v` toward 0 by the step `step`.
fn settle(v: &dyn Cell32, step: &dyn Cell32) {
    let x = f32::from_bits(v.bits());
    if x == 0.0 || x.is_nan() {
        return;
    }
    if x < 0.0 {
        let y = x87::add(v.bits(), step.bits());
        v.set_bits(y);
        if f32::from_bits(y) >= 0.0 {
            v.set_bits(0);
        }
    } else {
        let y = x87::sub(v.bits(), step.bits());
        v.set_bits(y);
        let y = f32::from_bits(y);
        if y < 0.0 || y.is_nan() {
            v.set_bits(0);
        }
    }
}

/// The Rust core.
pub fn settle_drive_speeds_at(
    course: &Course,
    g: &GearPlaces,
    s: &SettlePlaces,
) -> Result<(), Refused> {
    let gear = g.gear.bits();
    if gear != 0 && !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    let step = || {
        if s.fast.bits() != 0 {
            crate::race::owner::race()
                .shared
                .physics_tables
                .settle_fast_step
                .bits()
        } else {
            crate::race::owner::race()
                .shared
                .physics_tables
                .settle_slow_step
                .bits()
        }
    };
    if s.first.bits() != 0 && gear == 0 {
        s.scratch.set_bits(step());
        settle(g.drive_speed, s.scratch);
    }
    if s.second.bits() != 0 {
        s.scratch.set_bits(step());
        settle(g.drive_speed_2, s.scratch);
    }
    if gear != 0 {
        return revs_from_wheels_at(course, g);
    }
    Ok(())
}

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub fn settle_drive_speeds(course: &Course) -> Result<(), Refused> {
    settle_drive_speeds_at(course, &GearPlaces::game(), &SettlePlaces::game())
}
