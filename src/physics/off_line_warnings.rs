//! `physics::off_line_warnings` (0x44148F): unless finished (+0xA8) or +0xA4 set, two
//! counters with their next cue times: +0x270 (`unk_71A248`) counts the steps the car is
//! far from the checkpoints (`g_nearestCheckpointDistance` not below `flt_614C6C`, or
//! +0x98 four or more from +0x94), else 0 with its time +0x278 300; +0x274 counts the
//! steps its heading is `dword_614C74` or more (unsigned) from the checkpoint's
//! (`dword_9ED808[+0x98]`), else 0 with +0x27C 300. Then, unless `dword_5BEC94` is set,
//! each counter at or past its time (by the sign) moves the time on 3000 and cues a sound
//! (`physics::cue_listed_car_sound`, ebx 0x50 turned, 0x52 off the line).
//!
//! Caller: `physics::track_checkpoints`. Refuses (before anything) a checkpoint (+0x98) past
//! 0x1418.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use rc2k_core::physics::progress::{
    LAST_CHECKPOINT, OFF_LINE_CUE, TURNED_CUE, Warnings, cue_due, off_line, spread,
};
use rc2k_core::physics::x87::below as x87_below;

/// The counters: off, turned, their times.
fn counters(shared: &RaceShared) -> [&crate::session::Dword; 4] {
    [
        &shared.car.off_line_steps,
        &shared.car.turned_steps,
        &shared.car.off_line_cue_time,
        &shared.car.turned_cue_time,
    ]
}

fn load(shared: &RaceShared) -> Warnings {
    let [off, turned, off_time, turned_time] =
        counters(shared).map(super::super::session::Dword::get);
    Warnings {
        off,
        turned,
        off_time,
        turned_time,
    }
}

fn store(shared: &RaceShared, w: &Warnings) {
    for (c, v) in counters(shared)
        .into_iter()
        .zip([w.off, w.turned, w.off_time, w.turned_time])
    {
        c.set(v);
    }
}

/// The steps.
// Converted: `rc2k_core::physics::progress::{Warnings, off_line, cue_due}`.
pub fn off_line_warnings(race: &Race, sound: &crate::sound::owner::Sound) -> Result<(), Refused> {
    if race.shared.car.finish_flag.get() != 0 || race.shared.car.go_stand_timer.get() != 0 {
        return Ok(());
    }
    let cp = race.shared.car.checkpoint.get();
    if cp > LAST_CHECKPOINT {
        return Err(BadInput("a checkpoint past 0x1418").into());
    }
    let near = x87_below(
        f32::from_bits(
            race.shared
                .physics_tables
                .nearest_checkpoint_distance
                .bits(),
        ) as f64,
        f32::from_bits(race.shared.physics_tables.off_line_distance.bits()) as f64,
    );
    let mut w = load(&race.shared);
    w.count_off(off_line(near, || {
        spread(cp, race.shared.car.reset_checkpoint.get())
    }));
    w.count_turned(
        spread(
            race.shared.car.car_heading.get(),
            crate::race::checkpoints_store::angle(cp as usize).get(),
        ),
        race.shared.physics_tables.off_line_word.get(),
    );
    store(&race.shared, &w);
    if race.shared.replay_file.hold_car_revs.get() != 0 {
        return Ok(());
    }
    for (count, time, cue) in [
        (
            &race.shared.car.turned_steps,
            &race.shared.car.turned_cue_time,
            TURNED_CUE,
        ),
        (
            &race.shared.car.off_line_steps,
            &race.shared.car.off_line_cue_time,
            OFF_LINE_CUE,
        ),
    ] {
        if let Some(next) = cue_due(count.get(), time.get()) {
            time.set(next);
            crate::physics::cue_listed_car_sound::cue_listed_car_sound(race, cue, sound)?;
        }
    }
    Ok(())
}
