//! `audio::stop_all_voices` (0x40AFD9): command 7: every playing voice to the stopping list.
//!
//! The playing list, if not empty, is put in front of the stopping list (its last voice's
//! `next` the stopping list's head) and emptied, and `g_channelsUsed` is 0. The channels
//! keep their `used` marks: the turn's update frees them as it stops the voices.
//! Refuses a playing list that does not end within 0x200 voices.

use crate::audio::own::AudioThread;
use crate::audio::voices::{GamePool, lists, store_lists};
use crate::errors::Refused;

pub fn stop_all_voices(audio: &AudioThread) -> Result<(), Refused> {
    // Converted: `rc2k_core::audio::voices::stop_all`.
    let mut l = lists(&audio.state);
    rc2k_core::audio::voices::stop_all(&mut GamePool::new(audio), &mut l)?;
    store_lists(&audio.state, &l);
    Ok(())
}
