//! Looks for the work car's checkpoint near the last one. The car's x and z go into `ground_x`
//! and `ground_z`; with no checkpoints the result is 0. Otherwise `race::nearest_checkpoint_in`
//! runs over the car's `checkpoint` - 1 ..= + 1; its distance (/ 65536) goes into
//! `nearest_checkpoint_distance`; above 0xA0000 (unsigned) `race::nearest_checkpoint` runs over
//! all of them, otherwise the nearest of the three is the result.
//!
//! Refuses what the two calls refuse.

use crate::errors::Refused;
use crate::math::x87::{ext_div2, int_80};
use crate::memory::plain_floats::FloatRow;
use crate::physics::car_store::Car;
use crate::race::owner_course::Course;
use crate::race::owner_physics::Physics;
use crate::race::owner_shared::RaceShared;
use rc2k_core::physics::progress;

/// The car's checkpoint: the nearest within the window about the one it had
/// (`race::nearest_checkpoint_in`), or, when that is too far, of them all
/// (`race::nearest_checkpoint`).
// Core: `rc2k_core::physics::progress::{near_window, search_all}`.
pub(super) fn find_car_checkpoint(
    physics: &Physics,
    course: &Course,
    shared: &RaceShared,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) -> Result<u32, Refused> {
    let at = (car.position.bits_at(0), car.position.bits_at(2));
    {
        let mut search = physics.ground_search().borrow_mut();
        search.x = at.0;
        search.z = at.1;
    }
    if shared.vars.get().checkpoint_count.get() == 0 {
        return Ok(0);
    }
    let (from, to) = progress::near_window(car.checkpoint);
    let near =
        crate::race::start_place::nearest_checkpoint_in(physics, course, shared, from, to, scene)?;
    // The nearest checkpoint's distance.
    shared.physics_tables_mut().nearest_checkpoint_distance = ext_div2(
        int_80(near.distance as i32, false),
        course.numbers.k65536,
        0x3F80_0000,
    );
    if progress::search_all(near.distance) {
        return crate::race::start_place::nearest_checkpoint_at(physics, course, shared, at, scene);
    }
    Ok(near.checkpoint)
}
