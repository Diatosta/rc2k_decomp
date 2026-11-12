//! `physics::off_line_warnings` (0x44148F): unless the car has finished (`finish_flag`) or
//! `go_stand_timer` is set, two counters with their next cue times: `off_line_steps` counts the
//! steps the car is far from the checkpoints (`g_nearestCheckpointDistance` not below
//! `off_line_distance`, or `checkpoint` four or more from `reset_checkpoint`), else 0 with its
//! time `off_line_cue_time` 300; `turned_steps` counts the steps its heading is `off_line_word` or
//! more (unsigned) from the checkpoint's heading, else 0 with `turned_cue_time` 300. Then, unless
//! `hold_car_revs` is set, each counter at or past its time (by the sign) moves the time on 3000
//! and cues a sound (`physics::cue_listed_car_sound`: 0x50 turned, 0x52 off the line).
//!
//! Caller: `physics::track_checkpoints`. Refuses (before anything) a checkpoint past 0x1418.

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::race::frame_store::FrameWork;
use crate::race::replay_file_store::ReplayFileBlock;
use crate::race::work_store::RaceWork;
use rc2k_core::physics::progress::{
    LAST_CHECKPOINT, OFF_LINE_CUE, TURNED_CUE, Warnings, cue_due, off_line, spread,
};
use rc2k_core::physics::x87::below as x87_below;

/// The counters: off, turned, their times.
fn counters(car: &Car) -> [&crate::session::PlainDword; 4] {
    [
        &car.off_line_steps,
        &car.turned_steps,
        &car.off_line_cue_time,
        &car.turned_cue_time,
    ]
}

fn load(car: &Car) -> Warnings {
    let [off, turned, off_time, turned_time] = counters(car).map(crate::session::PlainDword::get);
    Warnings {
        off,
        turned,
        off_time,
        turned_time,
    }
}

fn store(car: &Car, w: &Warnings) {
    for (c, v) in counters(car)
        .into_iter()
        .zip([w.off, w.turned, w.off_time, w.turned_time])
    {
        c.set(v);
    }
}

/// The steps.
// Converted: `rc2k_core::physics::progress::{Warnings, off_line, cue_due}`.
#[allow(
    clippy::too_many_arguments,
    reason = "the race's tables the warnings read, and the owners"
)]
pub fn off_line_warnings(
    frame_work: &FrameWork,
    work: &RaceWork,
    replay_file: &ReplayFileBlock,
    physics_tables: &PhysicsTables,
    checkpoints: &crate::race::checkpoints_store::Checkpoints,
    car: &Car,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    if car.finish_flag.get() != 0 || car.go_stand_timer.get() != 0 {
        return Ok(());
    }
    let cp = car.checkpoint.get();
    if cp > LAST_CHECKPOINT {
        return Err(BadInput("a checkpoint past 0x1418").into());
    }
    let near = x87_below(
        physics_tables.nearest_checkpoint_distance.get() as f64,
        physics_tables.off_line_distance.get() as f64,
    );
    let mut w = load(car);
    w.count_off(off_line(near, || spread(cp, car.reset_checkpoint.get())));
    w.count_turned(
        spread(
            car.car_heading.get(),
            crate::race::checkpoints_store::angle(checkpoints, cp as usize).get(),
        ),
        physics_tables.off_line_word.get(),
    );
    store(car, &w);
    if replay_file.hold_car_revs.get() != 0 {
        return Ok(());
    }
    for (count, time, cue) in [
        (&car.turned_steps, &car.turned_cue_time, TURNED_CUE),
        (&car.off_line_steps, &car.off_line_cue_time, OFF_LINE_CUE),
    ] {
        if let Some(next) = cue_due(count.get(), time.get()) {
            time.set(next);
            crate::physics::cue_listed_car_sound::cue_listed_car_sound(
                frame_work, work, cue, sound, audio,
            )?;
        }
    }
    Ok(())
}
