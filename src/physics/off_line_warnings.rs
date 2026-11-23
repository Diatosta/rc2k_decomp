//! Unless the car has finished (`finish_flag`) or `go_stand_timer` is set, two counters with their
//! next cue times: `off_line_steps` counts the steps the car is far from the checkpoints
//! (`nearest_checkpoint_distance` not below `off_line_distance`, or `checkpoint` four or more from
//! `reset_checkpoint`), else 0 with its time `off_line_cue_time` = 300; `turned_steps` counts the
//! steps its heading is `off_line_word` or more (unsigned) from the checkpoint's heading, else 0
//! with `turned_cue_time` = 300. Then, unless `hold_car_revs` is set, each counter at or past its
//! time moves the time on by 3000 and cues a sound (`physics::cue_listed_car_sound`: 0x50 turned,
//! 0x52 off the line).
//!
//! Refuses (before anything) a checkpoint past 0x1418.

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::Car;
use crate::physics::figures::{OFF_LINE_DISTANCE, OFF_LINE_WORD};
use crate::physics::tables_store::PhysicsTables;
use crate::race::frame_store::FrameWorkStore;
use crate::race::replay_file_store::ReplayFileBlock;
use crate::race::work_store::RaceWork;
use rc2k_core::physics::progress::{
    LAST_CHECKPOINT, OFF_LINE_CUE, TURNED_CUE, Warnings, cue_due, off_line, spread,
};
use rc2k_core::physics::x87::below as x87_below;

fn load(car: &Car) -> Warnings {
    Warnings {
        off: car.off_line_steps,
        turned: car.turned_steps,
        off_time: car.off_line_cue_time,
        turned_time: car.turned_cue_time,
    }
}

fn store(car: &mut Car, w: &Warnings) {
    car.off_line_steps = w.off;
    car.turned_steps = w.turned;
    car.off_line_cue_time = w.off_time;
    car.turned_cue_time = w.turned_time;
}

/// The steps.
// Core: `rc2k_core::physics::progress::{Warnings, off_line, cue_due}`.
#[allow(
    clippy::too_many_arguments,
    reason = "the race's tables the warnings read, and the owners"
)]
pub fn off_line_warnings(
    frame_work: &FrameWorkStore,
    work: &RaceWork,
    replay_file: &ReplayFileBlock,
    physics_tables: &PhysicsTables,
    checkpoints: crate::race::checkpoints_store::CheckpointsRef,
    car: &mut Car,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    if car.finish_flag != 0 || car.go_stand_timer != 0 {
        return Ok(());
    }
    let cp = car.checkpoint;
    if cp > LAST_CHECKPOINT {
        return Err(BadInput("a checkpoint past 0x1418").into());
    }
    let near = x87_below(
        physics_tables.nearest_checkpoint_distance as f64,
        OFF_LINE_DISTANCE as f64,
    );
    let mut w = load(car);
    w.count_off(off_line(near, || spread(cp, car.reset_checkpoint)));
    w.count_turned(
        spread(
            car.car_heading,
            crate::race::checkpoints_store::angle_run_on(checkpoints, cp as usize),
        ),
        OFF_LINE_WORD,
    );
    store(car, &w);
    if replay_file.hold_car_revs.get() != 0 {
        return Ok(());
    }
    for (count, time, cue) in [
        (car.turned_steps, &mut car.turned_cue_time, TURNED_CUE),
        (car.off_line_steps, &mut car.off_line_cue_time, OFF_LINE_CUE),
    ] {
        if let Some(next) = cue_due(count, *time) {
            *time = next;
            crate::physics::cue_listed_car_sound::cue_listed_car_sound(
                frame_work, work, cue, sound, audio,
            )?;
        }
    }
    Ok(())
}
