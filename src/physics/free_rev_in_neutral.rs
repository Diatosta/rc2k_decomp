//! `physics::free_rev` in neutral.

use crate::physics::free_rev::free_rev;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;

/// The exact version, for `physics::car_physics_step`.
pub fn free_rev_in_neutral(
    course: &Course,
    shared: &RaceShared,
    gear: &crate::physics::gear_store::GearFigures,
    scene: &crate::scene::owner::Scene,
) {
    if shared.car().gear.get() != 0 {
        return;
    }
    free_rev(course, shared, gear, scene);
}
