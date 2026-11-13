//! The replay's frame and tenth (`frame`,
//! `tenth`, 0 when playing, `replay_mode` 2; else the first block's +8 / 100 and its
//! remainder); at a whole frame, while recording (`replay_mode` 1),
//! `drive::save_replay_cars`. (The original also kept the four car blocks in slot frame & 7 of a
//! store nothing read; that copy is gone.)

use crate::errors::BadInput;
use crate::race::owner::Race;
use rc2k_core::drive::replay_frames::{PLAYING, RECORDING, frame_and_tenth};

pub fn keep_replay_frame(race: &Race) -> Result<(), BadInput> {
    race.replay.replay_frames().frame.set(0);
    race.replay.replay_frames().tenth.set(0);
    if race.course.numbers.get().replay_mode.get() != PLAYING {
        let (frame, tenth) = frame_and_tenth(race.car_blocks().cars[0].step_clock.get());
        race.replay.replay_frames().frame.set(frame);
        race.replay.replay_frames().tenth.set(tenth);
    }
    if race.replay.replay_frames().tenth.get() == 0
        && race.course.numbers.get().replay_mode.get() == RECORDING
    {
        crate::drive::save_replay_cars::save_replay_cars(race);
    }
    Ok(())
}
