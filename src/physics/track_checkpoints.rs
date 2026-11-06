//! `physics::track_checkpoints` (0x44157B): the work car's checkpoints followed.
//! `physics::find_car_checkpoint` into +0x98 and `dword_6365E4`;
//! `physics::checkpoint_offset`; `physics::off_line_warnings`; at the last checkpoint
//! of the current split (`g_checkCounts[+0x98]` - 1 equal to +0x90) +0x94 = +0x98; then,
//! below `g_splitColourCount` and when `physics::passed_split_point` gives CF, +0x94 and
//! +0x98 the split's checkpoint (`dword_601030[+0x90]`) and +0x90 up by one.
//!
//! In:  nothing
//! Out: the registers as the calls leave them; no flags read (the caller calls on)
//!
//! Caller: `sub_43B635` (0x43B90F), every physics step. Refuses (before writing) a
//! checkpoint past 0x1418 or a split past 0x40, and what the calls refuse.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::race::owner::Race;
use rc2k_core::physics::splits::{at_split_end, check_tracked, split_ahead};

/// The steps. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::splits::{check_tracked, at_split_end, split_ahead}`.
pub fn track_checkpoints(race: &Race) -> Result<(), Refused> {
    let physics_store = &race.physics;
    let checkpoint = { crate::physics::find_car_checkpoint::find_car_checkpoint(race) }?;
    check_tracked(checkpoint, CAR.splits_passed.get())?;
    CAR.checkpoint.set(checkpoint);
    crate::scene::owner::scene()
        .lights
        .state
        .line_place
        .set(checkpoint);
    { crate::physics::checkpoint_offset::checkpoint_offset(&race.course) }?;
    crate::physics::off_line_warnings::off_line_warnings(&race.frame)?;
    let next = crate::race::checkpoints_store::cell_in(
        &crate::race::checkpoints_store::CHECKPOINTS.check_counts,
        CAR.checkpoint.get() as usize,
    )
    .get()
    .wrapping_sub(1);
    if at_split_end(next.wrapping_add(1), CAR.splits_passed.get()) {
        CAR.reset_checkpoint
            .set(crate::physics::car_store::CAR.checkpoint.get());
    }
    if !split_ahead(
        CAR.splits_passed.get(),
        physics_store.split_tables.split_colour_count.get(),
    ) {
        return Ok(());
    }
    if !{ crate::physics::passed_split_point::passed_split_point(race) }? {
        return Ok(());
    }
    let split = physics_store.split_tables.checkpoints[1 + CAR.splits_passed.get() as usize].get();
    CAR.reset_checkpoint.set(split);
    CAR.checkpoint.set(split);
    CAR.splits_passed.set(
        crate::physics::car_store::CAR
            .splits_passed
            .get()
            .wrapping_add(1),
    );
    Ok(())
}
