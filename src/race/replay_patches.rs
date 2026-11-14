//! The physics patches a replay was recorded with:
//! - `race::apply_replay_patches`: when the replay's header has them (`patches_in_header`) and they
//!   are not in force (`patches_in_force` 0), puts them in force: the current switches
//!   (`patch_switch`, 13 dwords) are kept in `kept_patch_switches`, the replay's (`header_patches`)
//!   take their place, `patches_applied` is 0 and the patches are applied
//!   (`physics::apply_patches`).
//! - `race::restore_replay_patches`: when the header has them and they are in force, takes them
//!   out: the kept switches go back and are applied again.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::session::DwordLike;
use crate::session::PlainDword;

/// `patch_switch` and the twelve settings after it (13 dwords).
pub fn switches(shared: &RaceShared) -> impl Iterator<Item = &PlainDword> {
    shared.work.get().patches.iter()
}

/// Copies the dwords, forwards.
fn move_dwords<'a, T: DwordLike + 'a, F: DwordLike + 'a>(
    to: impl Iterator<Item = &'a T>,
    from: impl Iterator<Item = &'a F>,
) {
    for (to, from) in to.zip(from) {
        to.set(from.get());
    }
}

/// `apply_replay_patches`.
pub fn apply_replay_patches(race: &Race) -> Result<(), Refused> {
    // Logic in `rc2k_core::race::viewer_flow::patches_change`.
    use rc2k_core::race::viewer_flow::patches_change;
    let shared = &race.shared;
    if patches_change(
        race.replay_work().patches_in_header.get(),
        race.viewer_panel().patches_in_force.get(),
        false,
    ) {
        race.viewer_panel().patches_in_force.set(1);
        move_dwords(
            race.viewer_panel().kept_patch_switches.iter(),
            switches(shared),
        );
        move_dwords(switches(shared), race.replay_work().header_patches.iter());
        shared.work.get().patches_applied.set(0);
        crate::physics::apply_patches::apply_patches(
            race.shared.physics_tables.get(),
            race.course.numbers.get(),
            race.shared.work.get(),
        );
    }
    Ok(())
}

/// `restore_replay_patches`.
pub fn restore_replay_patches(race: &Race) -> Result<(), Refused> {
    // Logic in `rc2k_core::race::viewer_flow::patches_change`.
    use rc2k_core::race::viewer_flow::patches_change;
    let shared = &race.shared;
    if patches_change(
        race.replay_work().patches_in_header.get(),
        race.viewer_panel().patches_in_force.get(),
        true,
    ) {
        race.viewer_panel().patches_in_force.set(0);
        move_dwords(
            switches(shared),
            race.viewer_panel().kept_patch_switches.iter(),
        );
        shared.work.get().patches_applied.set(0);
        crate::physics::apply_patches::apply_patches(
            race.shared.physics_tables.get(),
            race.course.numbers.get(),
            race.shared.work.get(),
        );
    }
    Ok(())
}
