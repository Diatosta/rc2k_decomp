//! Follows the work car's checkpoints. `physics::find_car_checkpoint` into the car's checkpoint and
//! `line_place`; `physics::checkpoint_offset`; `physics::off_line_warnings`; at the last checkpoint
//! of the current split (the checkpoint's check count - 1 equal to `splits_passed`)
//! `reset_checkpoint` = `checkpoint`; then, below `split_colour_count` and when
//! `physics::passed_split_point` is true, `reset_checkpoint` and `checkpoint` become the split's
//! checkpoint (`SplitTables.checkpoints[1][splits_passed]`) and `splits_passed` goes up by one.
//!
//! Refuses (before writing) a checkpoint past 0x1418 or a split past 0x40, and what the calls
//! refuse.

use crate::errors::Refused;
use crate::physics::step_families::CheckpointInputs;
use rc2k_core::physics::splits::{at_split_end, check_tracked, split_ahead};

/// The steps.
// Core: `rc2k_core::physics::splits::{check_tracked, at_split_end, split_ahead}`.
pub fn track_checkpoints(
    c: &CheckpointInputs<'_>,
    scene: &crate::scene::owner::Scene,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    let physics_store = c.physics;
    let checkpoint = {
        crate::physics::find_car_checkpoint::find_car_checkpoint(
            c.physics, c.course, c.shared, scene,
        )
    }?;
    check_tracked(checkpoint, c.car.splits_passed.get())?;
    c.car.checkpoint.set(checkpoint);
    scene.line_and_shake_mut().line_place = checkpoint;
    {
        crate::physics::checkpoint_offset::checkpoint_offset(
            &c.shared.buffers.reads.get().engine_tables,
            c.course,
            c.vars,
            c.checkpoints,
            c.car,
        )
    }?;
    crate::physics::off_line_warnings::off_line_warnings(
        c.frame_work,
        c.work,
        c.shared.replay_file(),
        c.tables,
        c.checkpoints,
        c.car,
        sound,
        audio,
    )?;
    let next = crate::race::checkpoints_store::check_count_run_on(
        c.checkpoints,
        c.car.checkpoint.get() as usize,
    )
    .wrapping_sub(1);
    if at_split_end(next.wrapping_add(1), c.car.splits_passed.get()) {
        c.car.reset_checkpoint.set(c.car.checkpoint.get());
    }
    if !split_ahead(
        c.car.splits_passed.get(),
        physics_store.split_tables().split_colour_count,
    ) {
        return Ok(());
    }
    if !{ crate::physics::passed_split_point::passed_split_point(c) }? {
        return Ok(());
    }
    let split = physics_store.split_tables().checkpoints[1 + c.car.splits_passed.get() as usize];
    c.car.reset_checkpoint.set(split);
    c.car.checkpoint.set(split);
    c.car
        .splits_passed
        .set(c.car.splits_passed.get().wrapping_add(1));
    Ok(())
}
