//! `physics::test_drive` (0x4217E4): the car's figures from a test drive. Its specs
//! (`physics::load_car_specs`) and gear ratios (`physics::make_gear_ratios`); the drive
//! speeds, the car's speed, the gear, the engine and `gearbox_kind` kept (pushed), the
//! drive's setting `test_power_factor` into `engine_power_factor`; `physics::test_drive_run`; the six
//! put back.
//!
//! Caller: `scene::set_up_engine` (0x442AFB, the car's choice). Refuses what its parts refuse
//! (everything written put back).

use crate::errors::Refused;
use crate::physics::gear_store::GearFigures;
use crate::physics::test_drive_run;
use crate::race::car_specs_store::CarSpecs;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;

pub fn test_drive(
    car_specs: &CarSpecs,
    course: &Course,
    shared: &RaceShared,
    gear: &GearFigures,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    crate::physics::load_car_specs::load_car_specs(
        car_specs,
        course,
        &shared.physics_tables,
        shared.car(),
        gear,
        scene,
    )?;
    crate::physics::make_gear_ratios::make_gear_ratios(
        shared.car(),
        &shared.physics_tables,
        &course.numbers,
        gear,
        scene,
    )?;
    // The figures kept round the run: the drive speeds, the car's speed, the gear, the
    // engine, the gearbox kind.
    let kept = [
        shared.car().drive_speed_a.bits(),
        shared.car().drive_speed_b.bits(),
        shared.car().local_velocity[2].bits(),
        shared.car().gear.bits(),
        shared.car().engine_speed.bits(),
        shared.car().gearbox_kind.bits(),
    ];
    shared
        .car()
        .engine_power_factor
        .set(gear.test_power_factor.get());
    test_drive_run::test_drive_run(course, shared, gear, scene)?;
    let [a, b, speed, gear, engine, kind] = kept;
    shared.car().drive_speed_a.set_bits(a);
    shared.car().drive_speed_b.set_bits(b);
    shared.car().local_velocity[2].set_bits(speed);
    shared.car().gear.set_bits(gear);
    shared.car().engine_speed.set_bits(engine);
    shared.car().gearbox_kind.set_bits(kind);
    Ok(())
}
