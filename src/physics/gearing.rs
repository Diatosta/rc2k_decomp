//! The gearing's sums between the engine's speed and the driven wheels' (fields of the
//! current car's work copy, `drive/car.rs`): `physics::revs_from_wheels` (0x43C0C9),
//! `physics::wheels_from_revs` (0x43C124). Both do nothing in gear 0 (neutral); else they
//! divide or multiply by the gear's ratio, `g_gearRatios[gear]` (0x6140C0; gear -1, the
//! reverse, at 0x6140BC; seven forward gears by `physics::make_gear_ratios`). With `g_carDriveKind`
//! 1 the two driven speeds are averaged / both set: by the look of it four-wheel drive (a
//! reading, not seen in a race). The exact x87 versions (`*_at`) serve the Rust callers in
//! the gearbox.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;
use crate::session::{Cell32, Float};

/// The variables: the car's work copy's and the ratio table (gear -1 first).
#[derive(Clone, Copy)]
pub struct GearPlaces<'a> {
    pub gear: &'a dyn Cell32,
    pub engine_speed: &'a dyn Cell32,
    pub drive_speed: &'a dyn Cell32,
    pub drive_speed_b: &'a dyn Cell32,
    pub drive_kind: &'a dyn Cell32,
    pub ratios: &'a [Float; 9],
    pub speed: &'a dyn Cell32,
}

impl<'a> GearPlaces<'a> {
    pub fn game(shared: &'a RaceShared) -> Self {
        Self {
            gear: &shared.car.gear,
            engine_speed: &shared.car.engine_speed,
            drive_speed: &shared.car.drive_speed_a,
            drive_speed_b: &shared.car.drive_speed_b,
            drive_kind: &shared.car.drive_kind,
            ratios: &shared.physics_tables.gear_ratios,
            speed: &shared.car.local_velocity[2],
        }
    }
}

/// The gear (None for neutral) and its ratio; refused outside the ratio table (-1 ..= 7).
fn gear_ratio(p: &GearPlaces<'_>) -> Result<Option<(u32, u32)>, Refused> {
    let gear = p.gear.bits();
    if gear == 0 {
        return Ok(None);
    }
    if !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    Ok(Some((gear, p.ratios[(gear as i32 + 1) as usize].bits())))
}

/// `physics::revs_from_wheels` (0x43C0C9): in gear, engine speed = the driven speed (with
/// drive kind 1 the two driven speeds' sum times 0.5) over the gear's ratio.
pub fn revs_from_wheels_at(course: &Course, p: &GearPlaces<'_>) -> Result<(), Refused> {
    let Some((_, ratio)) = gear_ratio(p)? else {
        return Ok(());
    };
    let speed = if p.drive_kind.bits() == 1 {
        crate::math::x87_run::sum_times_over(
            p.drive_speed.bits(),
            p.drive_speed_b.bits(),
            course.numbers.half.bits(),
            ratio,
        )
    } else {
        x87::div(p.drive_speed.bits(), ratio)
    };
    p.engine_speed.set_bits(speed);
    Ok(())
}

/// `physics::wheels_from_revs` (0x43C124): in gear, driven speed = engine speed times the
/// gear's ratio; with drive kind 1 the second driven speed the same (reloaded: `fld;
/// fstp`).
pub fn wheels_from_revs_at(p: &GearPlaces<'_>) -> Result<(), Refused> {
    let Some((_, ratio)) = gear_ratio(p)? else {
        return Ok(());
    };
    let speed = x87::mul(p.engine_speed.bits(), ratio);
    p.drive_speed.set_bits(speed);
    if p.drive_kind.bits() == 1 {
        p.drive_speed_b.set_bits(x87::loaded(speed));
    }
    Ok(())
}

/// `physics::revs_from_speed` (0x43C101): in gear, engine speed = `g_carSpeed` over the
/// gear's ratio.
pub fn revs_from_speed_at(p: &GearPlaces<'_>) -> Result<(), Refused> {
    let Some((_, ratio)) = gear_ratio(p)? else {
        return Ok(());
    };
    p.engine_speed.set_bits(x87::div(p.speed.bits(), ratio));
    Ok(())
}
