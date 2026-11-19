//! The gearing's sums between the engine's speed and the driven wheels' (fields of the current
//! car's work copy, `drive/car.rs`): `physics::revs_from_wheels`, `physics::wheels_from_revs`.
//! Both do nothing in gear 0 (neutral); otherwise they divide or multiply by the gear's ratio,
//! `gear_ratios[gear + 1]` (gear -1, the reverse, just before it; seven forward gears made by
//! `physics::make_gear_ratios`). With the drive kind 1 the two driven speeds are averaged or
//! both set: by the look of it four-wheel drive (a reading, not seen in a race). The exact x87
//! versions (`*_at`) serve the Rust callers in the gearbox.

use crate::errors::{BadInput, Refused};
use crate::math::cell_x87::CellX87;
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::memory::session::{PlainDword, PlainFloat, PlainInt};
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;

/// The variables: the car's work copy's and the ratio table (gear -1 first).
#[derive(Clone, Copy)]
pub struct GearPlaces<'a> {
    pub gear: &'a PlainInt,
    pub engine_speed: &'a PlainFloat,
    pub drive_speed: &'a PlainFloat,
    pub drive_speed_b: &'a PlainFloat,
    pub drive_kind: &'a PlainDword,
    pub ratios: &'a [PlainFloat; 9],
    pub speed: &'a PlainFloat,
}

impl<'a> GearPlaces<'a> {
    pub fn game(car: &'a Car, tables: &'a PhysicsTables) -> Self {
        Self {
            gear: &car.gear,
            engine_speed: &car.engine_speed,
            drive_speed: &car.drive_speed_a,
            drive_speed_b: &car.drive_speed_b,
            drive_kind: &car.drive_kind,
            ratios: &tables.gear_ratios,
            speed: &car.local_velocity[2],
        }
    }
}

/// The gear (None for neutral) and its ratio; refused outside the ratio table (-1 ..= 7).
fn gear_ratio(p: &GearPlaces<'_>) -> Result<Option<(i32, f32)>, Refused> {
    let gear = p.gear.get();
    if gear == 0 {
        return Ok(None);
    }
    if !(-1..=7).contains(&gear) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    Ok(Some((gear, p.ratios[(gear + 1) as usize].get())))
}

/// `physics::revs_from_wheels`: in gear, engine speed = the driven speed (with
/// drive kind 1 the two driven speeds' sum times 0.5) over the gear's ratio.
pub fn revs_from_wheels_at(numbers: &Numbers, p: &GearPlaces<'_>) -> Result<(), Refused> {
    let Some((_, ratio)) = gear_ratio(p)? else {
        return Ok(());
    };
    let speed = if p.drive_kind.get() == 1 {
        bh::stf((p.drive_speed.ld() + p.drive_speed_b.ld()) * numbers.half.ld() / bh::ld(ratio))
    } else {
        x87::div(p.drive_speed.get(), ratio)
    };
    p.engine_speed.set(speed);
    Ok(())
}

/// `physics::wheels_from_revs`: in gear, driven speed = engine speed times the
/// gear's ratio; with drive kind 1 the second driven speed the same (reloaded first).
pub fn wheels_from_revs_at(p: &GearPlaces<'_>) -> Result<(), Refused> {
    let Some((_, ratio)) = gear_ratio(p)? else {
        return Ok(());
    };
    let speed = x87::mul(p.engine_speed.get(), ratio);
    p.drive_speed.set(speed);
    if p.drive_kind.get() == 1 {
        p.drive_speed_b.set(x87::loaded(speed));
    }
    Ok(())
}

/// `physics::revs_from_speed`: in gear, engine speed = the car speed over the gear's ratio.
pub fn revs_from_speed_at(p: &GearPlaces<'_>) -> Result<(), Refused> {
    let Some((_, ratio)) = gear_ratio(p)? else {
        return Ok(());
    };
    p.engine_speed.set(x87::div(p.speed.get(), ratio));
    Ok(())
}
