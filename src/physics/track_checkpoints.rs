//! `physics::track_checkpoints` (0x44157B): the work car's checkpoints followed.
//! `physics::find_car_checkpoint` into +0x98 and `dword_6365E4`;
//! `physics::checkpoint_offset`; `physics::off_line_warnings`; at the last checkpoint
//! of the current split (`g_checkCounts[+0x98]` - 1 equal to +0x90) +0x94 = +0x98; then,
//! below `g_splitColourCount` and when `physics::passed_split_point` gives CF, +0x94 and
//! +0x98 the split's checkpoint (`dword_601030[+0x90]`) and +0x90 up by one.
//!
//! Caller: `sub_43B635` (0x43B90F), every physics step. Refuses (before writing) a
//! checkpoint past 0x1418 or a split past 0x40, and what the calls refuse.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::race::owner::Race;
use rc2k_core::physics::splits::{at_split_end, check_tracked, split_ahead};

/// The steps. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::splits::{check_tracked, at_split_end, split_ahead}`.
pub fn track_checkpoints(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    let physics_store = &race.physics;
    let checkpoint = { crate::physics::find_car_checkpoint::find_car_checkpoint(race, scene) }?;
    check_tracked(checkpoint, race.shared.car.splits_passed.get())?;
    race.shared.car.checkpoint.set(checkpoint);
    scene.lights.state.line_place.set(checkpoint);
    { crate::physics::checkpoint_offset::checkpoint_offset(race) }?;
    crate::physics::off_line_warnings::off_line_warnings(race, sound)?;
    let next = crate::race::checkpoints_store::cell_in(
        &crate::race::checkpoints_store::CHECKPOINTS.check_counts,
        race.shared.car.checkpoint.get() as usize,
    )
    .get()
    .wrapping_sub(1);
    if at_split_end(next.wrapping_add(1), race.shared.car.splits_passed.get()) {
        race.shared
            .car
            .reset_checkpoint
            .set(race.shared.car.checkpoint.get());
    }
    if !split_ahead(
        race.shared.car.splits_passed.get(),
        physics_store.split_tables.split_colour_count.get(),
    ) {
        return Ok(());
    }
    if !{ crate::physics::passed_split_point::passed_split_point(race, scene) }? {
        return Ok(());
    }
    let split = physics_store.split_tables.checkpoints
        [1 + race.shared.car.splits_passed.get() as usize]
        .get();
    race.shared.car.reset_checkpoint.set(split);
    race.shared.car.checkpoint.set(split);
    race.shared
        .car
        .splits_passed
        .set(race.shared.car.splits_passed.get().wrapping_add(1));
    Ok(())
}
