//! `physics::hit_part_checks` (0x441046): +0x284 (`parts_state`) 1; with bits 5, 6, 1, 2
//! of +0x298 all set -1; with bits 7, 8, 3, 4 also set 0 (a reading: which parts are
//! left); then, with +0x26C 0, `physics::drop_hit_parts` and `physics::sound_hit_parts`.
//!
//! Caller: `physics::car_physics_step` (0x43B957), every physics step. Refuses what the calls
//! refuse; what was written is put back.

use crate::errors::Refused;
use crate::physics::drop_hit_parts::drop_hit_parts;
use crate::physics::sound_hit_parts::sound_hit_parts;
use crate::race::matrices_store::SceneMatrices;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use rc2k_core::physics::parts::parts_left;

// Converted: `rc2k_core::physics::parts::parts_left`.
pub fn hit_part_checks(
    numbers: &Numbers,
    matrices: &SceneMatrices,
    shared: &RaceShared,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let left = parts_left(shared.car.parts_present.get());
    shared.car.parts_state.set(left);
    if left != 0 || shared.car.tyre_change_state.get() != 0 {
        return Ok(());
    }
    drop_hit_parts(numbers, &shared.work, &shared.car, scene)?;
    sound_hit_parts(numbers, matrices, shared)
}
