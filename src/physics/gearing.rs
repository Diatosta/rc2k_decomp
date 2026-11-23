//! The gearing's sums between the engine's speed and the driven wheels' (fields of the current
//! car's work copy, `drive/car.rs`): `physics::revs_from_wheels`, `physics::wheels_from_revs`.
//! Both do nothing in gear 0 (neutral); otherwise they divide or multiply by the gear's ratio,
//! `gear_ratios[gear + 1]` (gear -1, the reverse, just before it; seven forward gears made by
//! `physics::make_gear_ratios`). With the drive kind 1 the two driven speeds are averaged or
//! both set: by the look of it four-wheel drive (a reading, not seen in a race). The exact x87
//! versions (`*_at`) serve the Rust callers in the gearbox.

use crate::errors::Refused;
use crate::math::cell_x87::PlainX87;
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::memory::plain_floats::FloatRow;
use crate::physics::car_store::Car;
use crate::race::numbers_store::Numbers;

/// The gear (None for neutral) and its ratio in the ratio table (gear -1 first); refused outside
/// the table (-1 ..= 7).
fn gear_ratio(car: &Car, ratios: &[f32; 9]) -> Result<Option<(i32, f32)>, Refused> {
    let gear = car.gear;
    let Some(index) = rc2k_core::physics::gearing::ratio_index(gear)? else {
        return Ok(None);
    };
    Ok(Some((gear, ratios[index])))
}

/// `physics::revs_from_wheels`: in gear, engine speed = the driven speed (with
/// drive kind 1 the two driven speeds' sum times 0.5) over the gear's ratio.
pub(super) fn revs_from_wheels_at(
    numbers: &Numbers,
    car: &mut Car,
    ratios: &[f32; 9],
) -> Result<(), Refused> {
    let Some((_, ratio)) = gear_ratio(car, ratios)? else {
        return Ok(());
    };
    let speed = if car.drive_kind == 1 {
        bh::stf(
            (car.drive_speed_a.ld() + car.drive_speed_b.ld()) * numbers.half.ld() / bh::ld(ratio),
        )
    } else {
        x87::div(car.drive_speed_a, ratio)
    };
    car.engine_speed = speed;
    Ok(())
}

/// `physics::wheels_from_revs`: in gear, driven speed = engine speed times the
/// gear's ratio; with drive kind 1 the second driven speed the same (reloaded first).
pub(super) fn wheels_from_revs_at(car: &mut Car, ratios: &[f32; 9]) -> Result<(), Refused> {
    let Some((_, ratio)) = gear_ratio(car, ratios)? else {
        return Ok(());
    };
    let speed = x87::mul(car.engine_speed, ratio);
    car.drive_speed_a = speed;
    if car.drive_kind == 1 {
        car.drive_speed_b = x87::loaded(speed);
    }
    Ok(())
}

/// `physics::revs_from_speed`: in gear, engine speed = the car speed over the gear's ratio.
pub(super) fn revs_from_speed_at(car: &mut Car, ratios: &[f32; 9]) -> Result<(), Refused> {
    let Some((_, ratio)) = gear_ratio(car, ratios)? else {
        return Ok(());
    };
    car.engine_speed = x87::div(car.local_velocity.at(2), ratio);
    Ok(())
}
