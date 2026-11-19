//! `race::play_replay_range`: the viewer plays from the block read: its cameras into the block
//! (`drive::replay_frame_marks`), the race's sounds started unless the frames are being saved as
//! pictures (`race::start_race_sounds`, not with `replay_recording` 1), the replay played
//! (`race::play_movie`), the sounds reset (`race::reset_race_sounds`), the key that ended it
//! (`input::take_key`) into `key_after_play`, `wait_key_release` 1 (keys held), the block shown
//! held to the last (the index's `block_count` at most, unsigned; then frame 0), `viewer_stopped` 1
//! (stopped).

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn play_replay_range(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let sound = o.sound;
    let replay = &race.replay;
    use crate::race::race_end;
    crate::drive::replay_frame_marks::replay_frame_marks(race)?;
    if race.viewer_panel().replay_recording.get() != 1 {
        crate::race::sounds::start_race_sounds(sound, o.audio, &o.system.session.picture_clock)?;
    }
    crate::race::play_movie::play_movie(race, o)?;
    race_end::reset_race_sounds(&race_end::SoundVars::game(sound), o)?;
    // The key taken: its low byte only.
    let key = crate::input::take_key::take_key(o.input) as u32 & 0xFF;
    replay.replay_view().key_after_play.set(key);
    replay.replay_view().wait_key_release.set(1);
    if race.shared.replay_file().index.block_count.get() <= replay.replay_view().replay_block.get()
    {
        replay
            .replay_view()
            .replay_block
            .set(race.shared.replay_file().index.block_count.get());
        replay.replay_view().replay_frame.set(0);
    }
    race.shared.replay_file().viewer_stopped.set(true);
    Ok(())
}
