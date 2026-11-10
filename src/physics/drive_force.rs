//! `physics::drive_force` (0x43CB14): the engine's drive. `physics::engine_torque_or_cut`;
//! `flt_614740` = (torque + `flt_614744`) x +0x638 (`unk_71A610`, `fild`) / 256 -
//! `flt_614744`; `flt_614750` = it x `flt_5CB968` / the gear's ratio / 10000, or 0 with
//! +0x1D0 (`unk_71A1A8`) set; `physics::drive_torque_share`.
//!
//! Caller: `physics::car_physics_step` (0x43B732), every physics step. Refuses (before anything) a
//! gear outside -1 ..= 7.

use crate::errors::{BadInput, Refused};
use crate::math::x87_run::{mul_div2, sum_times_int_over_minus};
use crate::physics::drive_torque_share::drive_torque_share;
use crate::physics::engine_torque_or_cut::engine_torque_or_cut;
use crate::physics::gear_store::GEAR;
use crate::race::owner::Race;

/// The exact version, for the Rust callers' exact comparisons.
pub fn drive_force(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let gear = race.shared.car.gear.get();
    if !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    engine_torque_or_cut(race, scene);
    race.shared
        .physics_tables
        .drive_ratio
        .set_bits(sum_times_int_over_minus(
            race.shared.physics_tables.engine_force.bits(),
            race.shared.physics_tables.engine_torque.bits(),
            race.shared.car.throttle.get() as i32,
            race.course.numbers.k256.bits(),
        ));
    race.shared.physics_tables.drive_torque.set_bits(mul_div2(
        race.shared.physics_tables.drive_ratio.bits(),
        GEAR.drive_torque_figure.bits(),
        race.shared.physics_tables.gear_ratios[(gear as i32 + 1) as usize].bits(),
        race.course.numbers.k10000.bits(),
    ));
    if race.shared.car.drive_cut_steps.get() != 0 {
        race.shared.physics_tables.drive_torque.set_bits(0);
    }
    drive_torque_share(race, scene);
    Ok(())
}
