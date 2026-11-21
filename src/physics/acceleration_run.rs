//! Measures the car's acceleration for its figures: from standing (the drive speeds 0, gear 1, the
//! engine 0, `gearbox_kind` 1), 5000 steps of the engine and gearbox alone (the car's speed is the
//! drive speed; `physics::count_down_timers`, `physics::gearbox_step`, `physics::engine_torque`,
//! the drive force `engine_force` x `drive_torque_figure` / the gear's ratio / `k10000` into
//! `drive_torque`, `physics::drive_torque_share`, the drag and the drive force
//! step the drive speeds, `physics::revs_from_wheels`); the steps at which the speed first reaches
//! `test_limits` and `LightsAndGear.gear.test_limits[1]` go into `test_marks` and
//! `LightsAndGear.gear.test_marks[1]`; every twentieth step the speed x `two_and_a_quarter` and the
//! gear go into `test_speeds[k]`, `test_gears[k]`; at the end the speed x `two_and_a_quarter` goes
//! into `LightsAndGear.gear.test_marks[2]` and the two steps / `hundred` (times) back into theirs.
//!
//! Refuses a gear outside -1 ..= 7 (past `gear_ratios`) and what its calls refuse; everything
//! written is put back.

use crate::errors::{BadInput, Refused};
use crate::math::cell_x87::CellX87;
use crate::math::x87_float::int;
use crate::memory::session::PlainFloat;
use crate::physics::car_store::Car;
use crate::physics::flow::c0;
use crate::physics::gear_store::GearFigures;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;

pub const STEPS: u32 = 0x1388;

/// One step.
fn step(
    course: &Course,
    shared: &RaceShared,
    figures: &GearFigures,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    shared.car().local_velocity[2].set(crate::math::x87::loaded(&shared.car().drive_speed_a));
    crate::physics::count_down_timers::count_down_timers(shared.car(), shared.vars.get());
    crate::physics::gearbox_step::gearbox_step(&course.numbers, shared, figures)?;
    crate::physics::engine_torque::engine_torque(course, shared, scene);
    let gear = shared.car().gear.get();
    if !(-1..=7).contains(&gear) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    x87_1(
        shared.physics_tables.get(),
        figures,
        &course.numbers,
        &shared.physics_tables.get().gear_ratios[gear.wrapping_add(1) as usize],
    );
    crate::physics::drive_torque_share::drive_torque_share(
        shared.car(),
        shared.physics_tables.get(),
        &course.numbers,
        scene,
    );
    x87_2(shared.car(), shared.physics_tables.get(), &course.numbers);
    {
        crate::physics::revs_from_wheels::revs_from_wheels(
            shared.car(),
            shared.physics_tables.get(),
            &course.numbers,
        )
    }?;
    Ok(())
}

fn steps(
    course: &Course,
    shared: &RaceShared,
    figures: &GearFigures,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    for mark in &figures.test_marks {
        mark.set(0);
    }
    shared.car().drive_speed_a.set(0.0);
    shared.car().drive_speed_b.set(0.0);
    shared.car().engine_speed.set(0.0);
    shared.car().gear.set(1);
    shared.car().gearbox_kind.set(1);
    for n in 0..STEPS {
        step(course, shared, figures, scene)?;
        if figures.test_marks[0].get() == 0
            && !c0(&shared.car().drive_speed_a, &figures.test_limits[0])
        {
            figures.test_marks[0].set(n);
        }
        if figures.test_marks[1].get() == 0
            && !c0(&shared.car().drive_speed_a, &figures.test_limits[1])
        {
            figures.test_marks[1].set(n);
        }
        if n % 20 == 0 {
            let slot = (n / 20) as usize;
            figures.test_speeds[slot].set(crate::math::x87::mul(
                &shared.car().drive_speed_a,
                course.numbers.two_and_a_quarter,
            ));
            figures.test_gears[slot].set(shared.car().gear.get());
        }
    }
    x87_3(shared.car(), figures, &course.numbers);
    Ok(())
}

pub fn acceleration_run(
    course: &Course,
    shared: &RaceShared,
    figures: &GearFigures,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    steps(course, shared, figures, scene)
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[drive_torque] = (([engine_force] * [drive_torque_figure]) / [r + 0]) / [k10000]`
fn x87_1(tables: &PhysicsTables, figures: &GearFigures, numbers: &Numbers, r: &PlainFloat) {
    let t0 = tables.engine_force.ld();
    let t1 = figures.drive_torque_figure.ld();
    let t2 = t0 * t1;
    let t3 = r.ld();
    let t4 = t2 / t3;
    let t5 = numbers.k10000.ld();
    let t6 = t4 / t5;
    tables.drive_torque.st(t6);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// The drag (`[drag]`, the value the test compares) is returned.
/// `[drag] = -(((([drive_speed_a] * [drive_speed_a]) * [air_drag]) / [k10000]) +
///  [rolling_drag])` (a long formula: checked by the tests' `reference_vectors`)
fn x87_2(car: &Car, tables: &PhysicsTables, numbers: &Numbers) -> crate::math::x87_float::X {
    let t0 = car.drive_speed_a.ld();
    let t1 = t0 * t0;
    let t2 = numbers.air_drag.ld();
    let t3 = t1 * t2;
    let t4 = numbers.k10000.ld();
    let t5 = t3 / t4;
    let t6 = numbers.rolling_drag.ld();
    let t7 = t5 + t6;
    let t8 = -t7;
    let t9 = tables.drive_torque.ld();
    let t10 = t8 + t9;
    let t11 = numbers.step_time.ld();
    let t12 = t10 * t11;
    let t13 = car.drive_speed_a.ld();
    let t14 = t12 + t13;
    car.drive_speed_a.st(t14);
    car.drive_speed_b.st(t14);
    t8
}

/// One straight x87 run as plain Rust over `X`.
fn x87_3(car: &Car, figures: &GearFigures, numbers: &Numbers) {
    let t0 = car.drive_speed_a.ld();
    let t1 = numbers.two_and_a_quarter.ld();
    let t2 = t0 * t1;
    figures.test_marks[2].st(t2);
    let t3 = int(crate::physics::flow::whole(&figures.test_marks[0]));
    let t4 = numbers.hundred.ld();
    let t5 = t3 / t4;
    figures.test_marks[0].st(t5);
    let t6 = int(crate::physics::flow::whole(&figures.test_marks[1]));
    let t7 = numbers.hundred.ld();
    let t8 = t6 / t7;
    figures.test_marks[1].st(t8);
}

#[cfg(test)]
mod tests;
