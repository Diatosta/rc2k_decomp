//! `physics::track_checkpoints` (0x44157B): the work car's checkpoints followed.
//! `physics::find_car_checkpoint` into +0x98 and `line_place`;
//! `physics::checkpoint_offset`; `physics::off_line_warnings`; at the last checkpoint
//! of the current split (`g_checkCounts[+0x98]` - 1 equal to +0x90) +0x94 = +0x98; then,
//! below `g_splitColourCount` and when `physics::passed_split_point` gives CF, +0x94 and
//! +0x98 the split's checkpoint (`SplitTables.checkpoints[1][+0x90]`) and +0x90 up by one.
//!
//! Caller: `physics::car_physics_step` (0x43B90F), every physics step. Refuses (before writing) a
//! checkpoint past 0x1418 or a split past 0x40, and what the calls refuse.

use crate::errors::Refused;
use crate::physics::step_families::CheckpointInputs;
use rc2k_core::physics::splits::{at_split_end, check_tracked, split_ahead};

/// The steps.
// Converted: `rc2k_core::physics::splits::{check_tracked, at_split_end, split_ahead}`.
pub fn track_checkpoints(
    c: &CheckpointInputs<'_>,
    scene: &crate::scene::owner::Scene,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    let physics_store = c.physics;
    let checkpoint = {
        crate::physics::find_car_checkpoint::find_car_checkpoint(
            c.physics, c.course, c.shared, scene,
        )
    }?;
    check_tracked(checkpoint, c.car.splits_passed.get())?;
    c.car.checkpoint.set(checkpoint);
    scene.lights.state.line_place.set(checkpoint);
    {
        crate::physics::checkpoint_offset::checkpoint_offset(c.course, c.vars, c.checkpoints, c.car)
    }?;
    crate::physics::off_line_warnings::off_line_warnings(
        c.frame_work,
        c.work,
        &c.shared.replay_file,
        c.tables,
        c.checkpoints,
        c.car,
        sound,
    )?;
    let next =
        crate::race::checkpoints_store::check_count(c.checkpoints, c.car.checkpoint.get() as usize)
            .get()
            .wrapping_sub(1);
    if at_split_end(next.wrapping_add(1), c.car.splits_passed.get()) {
        c.car.reset_checkpoint.set(c.car.checkpoint.get());
    }
    if !split_ahead(
        c.car.splits_passed.get(),
        physics_store.split_tables.split_colour_count.get(),
    ) {
        return Ok(());
    }
    if !{ crate::physics::passed_split_point::passed_split_point(c, scene) }? {
        return Ok(());
    }
    let split =
        physics_store.split_tables.checkpoints[1 + c.car.splits_passed.get() as usize].get();
    c.car.reset_checkpoint.set(split);
    c.car.checkpoint.set(split);
    c.car
        .splits_passed
        .set(c.car.splits_passed.get().wrapping_add(1));
    Ok(())
}
