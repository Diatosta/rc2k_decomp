//! `physics::find_car_checkpoint` (0x44139D): the work car's checkpoint looked for near the last
//! one. The car's x and z go into `ground_x` / `ground_z`; with no checkpoints the result is 0.
//! Else `race::nearest_checkpoint_in` runs over the car's `checkpoint` - 1 ..= + 1; its distance
//! (/ 65536) goes into `g_nearestCheckpointDistance`; above 0xA0000 (unsigned)
//! `race::nearest_checkpoint` runs over them all, else the nearest of the three is the result.
//!
//! Caller: `physics::track_checkpoints`. Refuses what the two calls refuse.

use crate::errors::Refused;
use crate::math::x87::{ext_div2, int_80};
use crate::race::owner::Race;
use rc2k_core::physics::progress;

/// The car's checkpoint: the nearest within the window about the one it had
/// (`race::nearest_checkpoint_in`), or, when that is too far, of them all
/// (`race::nearest_checkpoint`).
// Converted: `rc2k_core::physics::progress::{near_window, search_all}`.
pub fn find_car_checkpoint(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<u32, Refused> {
    let race_owner = race;
    race_owner
        .physics
        .ground
        .ground_x
        .set(race.shared.car.position[0].get());
    race_owner
        .physics
        .ground
        .ground_z
        .set(race.shared.car.position[2].get());
    if race.shared.vars.checkpoint_count.get() == 0 {
        return Ok(0);
    }
    let (from, to) = progress::near_window(race.shared.car.checkpoint.get());
    let near = crate::race::start_place::nearest_checkpoint_in(race, from, to, scene)?;
    scene.drawing.scene.scratch.set(near.distance);
    // g_nearestCheckpointDistance
    race.shared
        .physics_tables
        .nearest_checkpoint_distance
        .set_bits(
            ext_div2(
                int_80(near.distance as i32, false),
                race_owner.course.numbers.k65536.bits(),
                0x3F80_0000,
            )
            .to_bits(),
        );
    if progress::search_all(near.distance) {
        return crate::race::start_place::nearest_checkpoint(race, scene);
    }
    Ok(near.checkpoint)
}
