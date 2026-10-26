//! The gearing's sums between the engine's speed and the driven wheels' (fields of the
//! current car's work copy, `drive/car.rs`): `physics::revs_from_wheels` (0x43C0C9),
//! `physics::wheels_from_revs` (0x43C124). Both do nothing in gear 0 (neutral); else they
//! divide or multiply by the gear's ratio, `g_gearRatios[gear]` (0x6140C0; gear -1, the
//! reverse, at 0x6140BC; seven forward gears by `sub_4219E6`). With `g_carDriveKind` 1 the
//! two driven speeds are averaged / both set: by the look of it four-wheel drive (a
//! reading, not seen in a race). The exact x87 versions (`*_at`) serve the Rust callers in
//! the gearbox.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::numbers_store::NUMBERS;

/// The variables' addresses: the game's, or a test's (`ratios` is gear 0's entry).
#[derive(Clone, Copy)]
pub struct GearPlaces {
    pub gear: u32,
    pub engine_speed: u32,
    pub drive_speed: u32,
    pub drive_speed_2: u32,
    pub drive_kind: u32,
    pub ratios: u32,
    pub speed: u32,
}

impl GearPlaces {
    pub fn game() -> Self {
        Self {
            gear: CAR.gear.addr() as u32,
            engine_speed: CAR.engine_speed.addr() as u32,
            drive_speed: CAR.drive_speed_a.addr() as u32,
            drive_speed_2: CAR.drive_speed_b.addr() as u32,
            drive_kind: CAR.drive_kind.addr() as u32,
            ratios: PHYSICS.gear_ratios[1].addr() as u32,
            speed: CAR.local_velocity[2].addr() as u32,
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    crate::pointers::read_or_end::<u32>(a, "physics::gearing's operand")
}

unsafe fn put(a: u32, v: u32) {
    crate::pointers::write_or_end::<u32>(a, v, "physics::gearing's operand")
}

/// The gear (None for neutral) and its ratio; refused outside the ratio table (-1 ..= 7).
unsafe fn gear_ratio(p: &GearPlaces) -> Result<Option<(u32, u32)>, Refused> {
    unsafe {
        let gear = at(p.gear);
        if gear == 0 {
            return Ok(None);
        }
        if !(-1..=7).contains(&(gear as i32)) {
            return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
        }
        Ok(Some((
            gear,
            at(p.ratios.wrapping_add(gear.wrapping_mul(4))),
        )))
    }
}

/// `physics::revs_from_wheels` (0x43C0C9): in gear, engine speed = the driven speed (with
/// drive kind 1 the two driven speeds' sum times 0.5) over the gear's ratio.
pub unsafe fn revs_from_wheels_at(p: &GearPlaces) -> Result<(), Refused> {
    unsafe {
        let Some((_, ratio)) = gear_ratio(p)? else {
            return Ok(());
        };
        let speed = if at(p.drive_kind) == 1 {
            crate::math::x87_run::sum_times_over(
                at(p.drive_speed),
                at(p.drive_speed_2),
                NUMBERS.half.bits(),
                ratio,
            )
        } else {
            x87::div(at(p.drive_speed), ratio)
        };
        put(p.engine_speed, speed);
        Ok(())
    }
}

/// `physics::wheels_from_revs` (0x43C124): in gear, driven speed = engine speed times the
/// gear's ratio; with drive kind 1 the second driven speed the same (reloaded: `fld;
/// fstp`).
pub unsafe fn wheels_from_revs_at(p: &GearPlaces) -> Result<(), Refused> {
    unsafe {
        let Some((_, ratio)) = gear_ratio(p)? else {
            return Ok(());
        };
        let speed = x87::mul(at(p.engine_speed), ratio);
        put(p.drive_speed, speed);
        if at(p.drive_kind) == 1 {
            put(p.drive_speed_2, x87::loaded(speed));
        }
        Ok(())
    }
}

/// `physics::revs_from_speed` (0x43C101): in gear, engine speed = `g_carSpeed` over the
/// gear's ratio.
pub unsafe fn revs_from_speed_at(p: &GearPlaces) -> Result<(), Refused> {
    unsafe {
        let Some((_, ratio)) = gear_ratio(p)? else {
            return Ok(());
        };
        put(p.engine_speed, x87::div(at(p.speed), ratio));
        Ok(())
    }
}
