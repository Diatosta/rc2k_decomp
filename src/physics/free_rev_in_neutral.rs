//! `physics::free_rev_in_neutral` (0x43EA54): `physics::free_rev` in neutral.
//!
//! Caller: `physics::car_physics_step` (0x43B7B2), every physics step.

use crate::physics::free_rev::free_rev;
use crate::race::owner::Race;

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub fn free_rev_in_neutral(race: &Race, scene: &crate::scene::owner::Scene) {
    if race.shared.car.gear.get() != 0 {
        return;
    }
    free_rev(race, scene);
}
