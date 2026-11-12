//! `physics::drive_force` (0x43CB14): the engine's drive. `physics::engine_torque_or_cut`;
//! `drive_ratio` = (torque + `engine_torque`) x +0x638 (`throttle`, `fild`) / 256 -
//! `engine_torque`; `drive_torque` = it x `drive_torque_figure` / the gear's ratio / 10000, or 0 with
//! +0x1D0 (`drive_cut_steps`) set; `physics::drive_torque_share`.
//!
//! Caller: `physics::car_physics_step` (0x43B732), every physics step. Refuses (before anything) a
//! gear outside -1 ..= 7.

use crate::errors::{BadInput, Refused};
use crate::math::x87::{mul_div2, sum_times_int_over_minus};
use crate::physics::drive_torque_share::drive_torque_share;
use crate::physics::engine_torque_or_cut::engine_torque_or_cut;
use crate::physics::gear_store::GearFigures;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;

/// The exact version, for the Rust callers' exact comparisons.
pub fn drive_force(
    course: &Course,
    shared: &RaceShared,
    gear_figures: &GearFigures,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let gear = shared.car().gear.get();
    if !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    engine_torque_or_cut(course, shared, scene);
    shared.physics_tables.get().drive_ratio.set_bits(
        sum_times_int_over_minus(
            &shared.physics_tables.get().engine_force,
            &shared.physics_tables.get().engine_torque,
            shared.car().throttle.get() as i32,
            &course.numbers.get().k256,
        )
        .to_bits(),
    );
    shared.physics_tables.get().drive_torque.set_bits(
        mul_div2(
            &shared.physics_tables.get().drive_ratio,
            &gear_figures.drive_torque_figure,
            &shared.physics_tables.get().gear_ratios[(gear as i32 + 1) as usize],
            &course.numbers.get().k10000,
        )
        .to_bits(),
    );
    if shared.car().drive_cut_steps.get() != 0 {
        shared.physics_tables.get().drive_torque.set(0.0);
    }
    drive_torque_share(
        shared.car(),
        shared.physics_tables.get(),
        course.numbers.get(),
        scene,
    );
    Ok(())
}
