//! The car's figures from a test drive. Its specs (`physics::load_car_specs`) and gear ratios
//! (`physics::make_gear_ratios`); the drive speeds, the car's speed, the gear, the engine and
//! `gearbox_kind` are kept, the drive's setting `test_power_factor` goes into
//! `engine_power_factor`; `physics::test_drive_run`; the six are put back.
//!
//! Refuses what its parts refuse (everything written is put back).

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
        shared.physics_tables.get(),
        shared.car(),
        gear,
        scene,
    )?;
    crate::physics::make_gear_ratios::make_gear_ratios(
        shared.car(),
        shared.physics_tables.get(),
        course.numbers.get(),
        gear,
        scene,
    )?;
    // The figures kept round the run: the drive speeds, the car's speed, the gear, the
    // engine, the gearbox kind.
    let kept = (
        shared.car().drive_speed_a.get(),
        shared.car().drive_speed_b.get(),
        shared.car().local_velocity[2].get(),
        shared.car().gear.get(),
        shared.car().engine_speed.get(),
        shared.car().gearbox_kind.get(),
    );
    shared
        .car()
        .engine_power_factor
        .set(gear.test_power_factor.get());
    test_drive_run::test_drive_run(course, shared, gear, scene)?;
    let (a, b, speed, gear, engine, kind) = kept;
    shared.car().drive_speed_a.set(a);
    shared.car().drive_speed_b.set(b);
    shared.car().local_velocity[2].set(speed);
    shared.car().gear.set(gear);
    shared.car().engine_speed.set(engine);
    shared.car().gearbox_kind.set(kind);
    Ok(())
}
