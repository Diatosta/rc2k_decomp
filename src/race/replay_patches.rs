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

/// `apply_replay_patches`.
pub(super) fn apply_replay_patches(race: &Race) -> Result<(), Refused> {
    // Logic in `rc2k_core::race::viewer_flow::patches_change`.
    use rc2k_core::race::viewer_flow::patches_change;
    let shared = &race.shared;
    if patches_change(
        race.replay_work().patches_in_header,
        race.viewer_panel().patches_in_force.get(),
        false,
    ) {
        race.viewer_panel().patches_in_force.set(1);
        let current = shared.work().patches;
        for (to, from) in race.viewer_panel().kept_patch_switches.iter().zip(current) {
            to.set(from);
        }
        let mut work = shared.work_mut();
        for (to, from) in work
            .patches
            .iter_mut()
            .zip(race.replay_work().header_patches)
        {
            *to = from;
        }
        work.patches_applied = 0;
        crate::physics::apply_patches::apply_patches(
            &mut race.shared.physics_tables_mut(),
            &race.course.numbers,
            &mut work,
        );
    }
    Ok(())
}

/// `restore_replay_patches`.
pub(super) fn restore_replay_patches(race: &Race) -> Result<(), Refused> {
    // Logic in `rc2k_core::race::viewer_flow::patches_change`.
    use rc2k_core::race::viewer_flow::patches_change;
    let shared = &race.shared;
    if patches_change(
        race.replay_work().patches_in_header,
        race.viewer_panel().patches_in_force.get(),
        true,
    ) {
        race.viewer_panel().patches_in_force.set(0);
        let mut work = shared.work_mut();
        for (to, from) in work
            .patches
            .iter_mut()
            .zip(race.viewer_panel().kept_patch_switches.iter())
        {
            *to = from.get();
        }
        work.patches_applied = 0;
        crate::physics::apply_patches::apply_patches(
            &mut race.shared.physics_tables_mut(),
            &race.course.numbers,
            &mut work,
        );
    }
    Ok(())
}
