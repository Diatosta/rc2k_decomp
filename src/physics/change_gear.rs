//! Changes the gear to the one asked for (`gear_asked`); nothing when it is the gear already.
//! From neutral: the gear is set and `physics::wheels_from_revs` runs. To neutral, or when
//! `physics::gear_bit` refuses the new gear: the gear becomes 0. Otherwise `wheels_from_revs`
//! (old gear), the gear set, `physics::revs_from_wheels`. Every change sets the car's
//! `gear_changed` and `gear_change_pending` to 1.
//!
//! Refuses (before anything) a gear or new gear outside -1 ..= 7 where the gearing reads its ratio,
//! and what `gear_bit` refuses.

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::Car;
use crate::physics::gear_bit::gear_bit_at;
use crate::physics::gearing::{revs_from_wheels_at, wheels_from_revs_at};
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use rc2k_core::physics::gear_shifts::Change;

fn ratio_gear(g: i32) -> Result<(), Refused> {
    if (-1..=7).contains(&g) {
        Ok(())
    } else {
        Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into())
    }
}

/// The core, on the car and the physics tables (the asked gear, the gear ratios).
// Core: `rc2k_core::physics::gear_shifts::change`.
fn change_gear_at(numbers: &Numbers, car: &mut Car, tables: &PhysicsTables) -> Result<(), Refused> {
    let gear = car.gear;
    let w = tables.gear_asked;
    let ratios = &tables.gear_ratios;
    let way = rc2k_core::physics::gear_shifts::change(gear, w);
    if way == Change::Same {
        return Ok(());
    }
    let mark = |car: &mut Car| {
        car.gear_changed = 1;
        car.gear_change_pending = 1;
    };
    let neutral = |car: &mut Car| {
        car.gear = 0;
        mark(car);
        Ok(())
    };
    if way == Change::FromNeutral {
        ratio_gear(w)?;
        car.gear = w;
        wheels_from_revs_at(car, ratios)?;
        mark(car);
        return Ok(());
    }
    if way == Change::ToNeutral {
        return neutral(car);
    }
    if gear_bit_at(car.lost_gear_bits, w)? {
        return neutral(car);
    }
    ratio_gear(gear)?;
    ratio_gear(w)?;
    wheels_from_revs_at(car, ratios)?;
    car.gear = w;
    revs_from_wheels_at(numbers, car, ratios)?;
    mark(car);
    Ok(())
}

pub fn change_gear(numbers: &Numbers, shared: &RaceShared) -> Result<(), Refused> {
    change_gear_at(numbers, &mut shared.car_mut(), &shared.physics_tables())
}
