//! `physics::engine_torque_or_cut` (0x43CB7D): at or past the engine's top the cut:
//! `rev_cut_count` 5 and the torque `engine_force` 0; below it (an unordered compare
//! counts as below), while `rev_cut_count` counts down the torque stays
//! 0; else `physics::engine_torque`.
//!
//! Caller: `physics::drive_force`.

use crate::physics::engine_torque::{below, engine_torque};
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;

/// The exact version, for `physics::drive_force`.
pub fn engine_torque_or_cut(
    course: &Course,
    shared: &RaceShared,
    scene: &crate::scene::owner::Scene,
) {
    if !below(
        shared.car().engine_speed.bits(),
        shared.car().engine_speed_limit.bits(),
    ) {
        shared.car().rev_cut_count.set(5);
    } else if shared.car().rev_cut_count.get() != 0 {
        shared
            .car()
            .rev_cut_count
            .set(shared.car().rev_cut_count.get().wrapping_sub(1));
    } else {
        engine_torque(course, shared, scene);
        return;
    }
    shared.physics_tables.engine_force.set(0.0);
}
