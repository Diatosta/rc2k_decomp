//! `physics::test_drive` (0x4217E4): the car's figures from a test drive. Its specs
//! (`physics::load_car_specs`) and gear ratios (`physics::make_gear_ratios`); the drive
//! speeds, the car's speed, the gear, the engine and `unk_71A010` kept (pushed), the
//! drive's setting `dword_5CC190` into `unk_71A3CC`; `physics::test_drive_run`; the six
//! put back.
//!
//! In:  nothing
//! Out: as `test_drive_run` leaves the registers, which nothing reads; no flags read
//!
//! Caller: `scene::set_up_engine` (0x442AFB, the car's choice). Refuses what its parts refuse
//! (everything written put back). The registers it leaves nothing reads (2026-11-03: the main
//! menu writes ebx at 0x474AC7 before reading it).

use crate::errors::Refused;
use crate::physics::gear_store::GEAR;
use crate::physics::test_drive_run;
use crate::race::owner::Race;

/// The Rust version.
pub fn test_drive(race: &Race) -> Result<(), Refused> {
    crate::physics::load_car_specs::load_car_specs(race)?;
    crate::physics::make_gear_ratios::make_gear_ratios(race)?;
    // The figures kept round the run: the drive speeds, the car's speed, the gear, the
    // engine, the gearbox kind.
    let kept = [
        race.shared.car.drive_speed_a.bits(),
        race.shared.car.drive_speed_b.bits(),
        race.shared.car.local_velocity[2].bits(),
        race.shared.car.gear.bits(),
        race.shared.car.engine_speed.bits(),
        race.shared.car.gearbox_kind.bits(),
    ];
    race.shared
        .car
        .engine_power_factor
        .set_bits(GEAR.test_power_factor.bits());
    test_drive_run::test_drive_run(race)?;
    let [a, b, speed, gear, engine, kind] = kept;
    race.shared.car.drive_speed_a.set_bits(a);
    race.shared.car.drive_speed_b.set_bits(b);
    race.shared.car.local_velocity[2].set_bits(speed);
    race.shared.car.gear.set_bits(gear);
    race.shared.car.engine_speed.set_bits(engine);
    race.shared.car.gearbox_kind.set_bits(kind);
    Ok(())
}
