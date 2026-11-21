//! `parts_state` = 1; with bits 5, 6, 1, 2 of `parts_present` all set, -1; with bits 7, 8, 3, 4
//! also set, 0 (a reading: which parts are left); then, with `tyre_change_state` 0,
//! `physics::drop_hit_parts` and `physics::sound_hit_parts`.
//!
//! Refuses what the calls refuse; what was written is put back.

use crate::errors::Refused;
use crate::physics::drop_hit_parts::drop_hit_parts;
use crate::physics::sound_hit_parts::sound_hit_parts;
use crate::race::matrices_store::ViewKept;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use rc2k_core::physics::parts::parts_left;

// Core: `rc2k_core::physics::parts::parts_left`.
pub(super) fn hit_part_checks(
    numbers: &Numbers,
    matrices: &ViewKept,
    shared: &RaceShared,
    kind_tables: &crate::race::kind_store::KindTables,
    scene: &crate::scene::owner::Scene,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    let left = parts_left(shared.car().parts_present.get());
    shared.car().parts_state.set(left);
    if left != 0 || shared.car().tyre_change_state.get() != 0 {
        return Ok(());
    }
    drop_hit_parts(numbers, shared.work.get(), shared.car(), scene)?;
    sound_hit_parts(numbers, matrices, shared, kind_tables, sound)
}
