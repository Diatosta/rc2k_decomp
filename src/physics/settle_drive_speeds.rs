//! `physics::settle_drive_speeds` (0x43BF6F): driven speeds brought toward 0 by a step
//! (`settle_fast_step`, 1.0, when `brake` is set, else `settle_slow_step`, 0.01; stored in
//! `scratch`), never past it: `g_carDriveSpeed` when `front_wheels_off` is set and the car
//! is in neutral, `g_carDriveSpeed2` when `rear_wheels_off` is set (whatever the gear). A
//! speed of 0 (or NaN) is left; one that reaches or crosses 0 becomes +0 (a NaN after the
//! step too). Then, in gear, `physics::revs_from_wheels`.
//!
//! Returns: in gear, `g_carGear` (`revs_from_wheels`)
//!
//! Caller: `physics::car_physics_step` (0x43B7AD, before `physics::free_rev_in_neutral`), every
//! physics step. Refuses (before anything) a gear outside -1 ..= 7 (`revs_from_wheels`' table).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::physics::flow::FloatView;
use crate::physics::gearing::{GearPlaces, revs_from_wheels_at};
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use crate::session::PlainDword;

/// The switches and the scratch: the game's or a test's.
#[derive(Clone, Copy)]
pub struct SettlePlaces<'a> {
    /// `front_wheels_off`: settle the first driven speed (in neutral).
    pub first: &'a PlainDword,
    /// `rear_wheels_off`: settle the second.
    pub second: &'a PlainDword,
    /// `brake`: the big step.
    pub fast: &'a PlainDword,
    /// `scratch`: the step is stored there (a dword holding the float's bits).
    pub scratch: &'a crate::session::PlainDword,
}

impl<'a> SettlePlaces<'a> {
    pub fn game(shared: &'a RaceShared, scene: &'a crate::scene::owner::Scene) -> Self {
        Self {
            first: &shared.car().front_wheels_off,
            second: &shared.car().rear_wheels_off,
            fast: &shared.car().brake,
            scratch: &scene.drawing.scene.get().scratch,
        }
    }
}

/// One speed `v` toward 0 by the step `step`.
fn settle(v: &(impl FloatView + ?Sized), step: f32) {
    let x = v.float();
    if x == 0.0 || x.is_nan() {
        return;
    }
    if x < 0.0 {
        let y = x87::add(x, step);
        v.set_float(y);
        if y >= 0.0 {
            v.set_float(0.0);
        }
    } else {
        let y = x87::sub(x, step);
        v.set_float(y);
        if y < 0.0 || y.is_nan() {
            v.set_float(0.0);
        }
    }
}

/// The steps.
pub fn settle_drive_speeds_at(
    tables: &PhysicsTables,
    numbers: &Numbers,
    g: &GearPlaces<'_>,
    s: &SettlePlaces<'_>,
) -> Result<(), Refused> {
    let gear = g.gear.bits();
    if gear != 0 && !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    let step = || {
        if s.fast.is_set() {
            tables.settle_fast_step.get()
        } else {
            tables.settle_slow_step.get()
        }
    };
    if s.first.is_set() && gear == 0 {
        let step = step();
        s.scratch.set(step.to_bits());
        settle(g.drive_speed, step);
    }
    if s.second.is_set() {
        let step = step();
        s.scratch.set(step.to_bits());
        settle(g.drive_speed_b, step);
    }
    if gear != 0 {
        return revs_from_wheels_at(numbers, g);
    }
    Ok(())
}

/// The exact version, for `physics::car_physics_step`.
pub fn settle_drive_speeds(
    numbers: &Numbers,
    shared: &RaceShared,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    settle_drive_speeds_at(
        shared.physics_tables.get(),
        numbers,
        &GearPlaces::game(shared.car(), shared.physics_tables.get()),
        &SettlePlaces::game(shared, scene),
    )
}
