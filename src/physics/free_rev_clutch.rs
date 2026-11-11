//! `physics::free_rev_clutch` (0x43EA63): in gear with the clutch out (+0x250,
//! `driven_wheels_off`): `physics::free_rev`, then `physics::wheels_from_revs`.
//!
//! Caller: `physics::car_physics_step` (0x43B7B7), every physics step. Refuses (before anything) a
//! gear outside -1 ..= 7.

use crate::errors::{BadInput, Refused};
use crate::physics::free_rev::free_rev;
use crate::physics::gearing::{GearPlaces, wheels_from_revs_at};
use crate::race::owner::Race;

/// The exact version, for `physics::car_physics_step`.
pub fn free_rev_clutch(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let gear = race.shared.car.gear.get();
    if gear == 0 || race.shared.car.driven_wheels_off.get() == 0 {
        return Ok(());
    }
    if !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    free_rev(race, scene);
    wheels_from_revs_at(&GearPlaces::game(
        &race.shared.car,
        &race.shared.physics_tables,
    ))?;
    Ok(())
}
