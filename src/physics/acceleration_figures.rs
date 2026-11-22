//! The car's figures from a test drive. Its specs (`physics::load_car_specs`) and gear ratios
//! (`physics::make_gear_ratios`); the drive speeds, the car's speed, the gear, the engine and
//! `gearbox_kind` are kept, the drive's setting `test_power_factor` goes into
//! `engine_power_factor`; `physics::acceleration_run`; the six are put back.
//!
//! Refuses what its parts refuse (everything written is put back).

use crate::errors::Refused;
use crate::memory::plain_floats::FloatRow;
use crate::memory::plain_floats::FloatRowMut;
use crate::physics::acceleration_run;
use crate::physics::gear_store::GearFigures;
use crate::race::car_specs_store::CarSpecs;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;

pub fn acceleration_figures(
    car_specs: &CarSpecs,
    course: &Course,
    shared: &RaceShared,
    gear: &GearFigures,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    crate::physics::load_car_specs::load_car_specs(
        car_specs,
        course,
        &mut shared.physics_tables_mut(),
        &mut shared.car_mut(),
        gear,
    )?;
    crate::physics::make_gear_ratios::make_gear_ratios(
        &shared.car(),
        &mut shared.physics_tables_mut(),
        &course.numbers,
        gear,
    )?;
    // The figures kept round the run: the drive speeds, the car's speed, the gear, the
    // engine, the gearbox kind.
    let kept = (
        shared.car().drive_speed_a,
        shared.car().drive_speed_b,
        shared.car().local_velocity.at(2),
        shared.car().gear,
        shared.car().engine_speed,
        shared.car().gearbox_kind,
    );
    shared.car_mut().engine_power_factor = gear.test_power_factor.get();
    acceleration_run::acceleration_run(course, shared, gear, scene)?;
    let (a, b, speed, gear, engine, kind) = kept;
    shared.car_mut().drive_speed_a = a;
    shared.car_mut().drive_speed_b = b;
    shared.car_mut().local_velocity.set_at(2, speed);
    shared.car_mut().gear = gear;
    shared.car_mut().engine_speed = engine;
    shared.car_mut().gearbox_kind = kind;
    Ok(())
}
