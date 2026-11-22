//! At or past the engine's top speed the cut: `rev_cut_count` = 5 and the torque `engine_force`
//! = 0; below it (an unordered compare counts as below), while `rev_cut_count` counts down the
//! torque stays 0; otherwise `physics::engine_torque`.

use crate::physics::engine_torque::engine_torque;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;
use rc2k_core::physics::engine::torque_cut;

/// The exact version, for `physics::drive_force`.
pub(super) fn engine_torque_or_cut(
    course: &Course,
    shared: &RaceShared,
    scene: &crate::scene::owner::Scene,
) {
    let Some(cut) = torque_cut(
        shared.car().engine_speed.get(),
        shared.car().engine_speed_limit.get(),
        shared.car().rev_cut_count.get(),
    ) else {
        engine_torque(course, shared, scene);
        return;
    };
    shared.car().rev_cut_count.set(cut);
    shared.physics_tables_mut().engine_force = 0.0;
}
