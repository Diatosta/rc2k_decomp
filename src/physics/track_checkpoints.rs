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
use crate::physics::car_store::Car;
use crate::physics::step_families::CheckpointInputs;
use rc2k_core::physics::splits::{at_split_end, check_tracked, split_ahead};

/// The steps.
// Core: `rc2k_core::physics::splits::{check_tracked, at_split_end, split_ahead}`.
pub fn track_checkpoints(
    c: &CheckpointInputs<'_>,
    scene: &crate::scene::owner::Scene,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
    car: &mut Car,
) -> Result<(), Refused> {
    let physics_store = c.physics;
    let checkpoint = {
        crate::physics::find_car_checkpoint::find_car_checkpoint(
            c.physics, c.course, c.shared, car, scene,
        )
    }?;
    check_tracked(checkpoint, car.splits_passed)?;
    car.checkpoint = checkpoint;
    scene.line_and_shake_mut().line_place = checkpoint;
    {
        crate::physics::checkpoint_offset::checkpoint_offset(
            c.shared.buffers.engine_tables(),
            c.course,
            c.vars,
            c.checkpoints,
            car,
        )
    }?;
    crate::physics::off_line_warnings::off_line_warnings(
        c.frame_work,
        &c.work.borrow(),
        c.shared.replay_file(),
        &c.tables.borrow(),
        c.checkpoints,
        car,
        sound,
        audio,
    )?;
    let next =
        crate::race::checkpoints_store::check_count_run_on(c.checkpoints, car.checkpoint as usize)
            .wrapping_sub(1);
    if at_split_end(next.wrapping_add(1), car.splits_passed) {
        car.reset_checkpoint = car.checkpoint;
    }
    if !split_ahead(
        car.splits_passed,
        physics_store.split_tables().split_colour_count,
    ) {
        return Ok(());
    }
    if !{ crate::physics::passed_split_point::passed_split_point(c, car) }? {
        return Ok(());
    }
    let split = physics_store.split_tables().checkpoints[1 + car.splits_passed as usize];
    car.reset_checkpoint = split;
    car.checkpoint = split;
    car.splits_passed = car.splits_passed.wrapping_add(1);
    Ok(())
}
