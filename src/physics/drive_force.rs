//! The engine's drive. After `physics::engine_torque_or_cut`: `drive_ratio` = (torque +
//! `engine_torque`) x `throttle` / 256 - `engine_torque`; `drive_torque` = it x
//! `drive_torque_figure` / the gear's ratio / 10000, or 0 with `drive_cut_steps` set; then
//! `physics::drive_torque_share`.
//!
//! Refuses (before anything) a gear outside -1 ..= 7.

use crate::errors::{BadInput, Refused};
use crate::math::x87::{mul_div2, sum_times_int_over_minus};
use crate::physics::drive_torque_share::drive_torque_share;
use crate::physics::engine_torque_or_cut::engine_torque_or_cut;
use crate::physics::gear_store::GearFigures;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;

/// The exact version, for the Rust callers' exact comparisons.
pub(super) fn drive_force(
    course: &Course,
    shared: &RaceShared,
    gear_figures: &GearFigures,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let gear = shared.car().gear.get();
    if !(-1..=7).contains(&gear) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    engine_torque_or_cut(course, shared, scene);
    {
        let mut tables = shared.physics_tables_mut();
        tables.drive_ratio = sum_times_int_over_minus(
            tables.engine_force,
            tables.engine_torque,
            shared.car().throttle.get() as i32,
            course.numbers.k256,
        );
        tables.drive_torque = mul_div2(
            tables.drive_ratio,
            &gear_figures.drive_torque_figure,
            tables.gear_ratios[(gear + 1) as usize],
            course.numbers.k10000,
        );
        if shared.car().drive_cut_steps.get() != 0 {
            tables.drive_torque = 0.0;
        }
    }
    drive_torque_share(
        shared.car(),
        &mut shared.physics_tables_mut(),
        &course.numbers,
        scene,
    );
    Ok(())
}
