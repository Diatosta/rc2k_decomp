//! Command 7: every playing voice to the stopping list: the playing list, if not empty, is put in
//! front of the stopping list and emptied, and no channel is counted used. The channels keep their
//! `used` marks: the turn's update frees them as it stops the voices. Refuses a playing list that
//! does not end within 0x200 voices.

use crate::audio::own::AudioThread;
use crate::audio::voices::{GamePool, lists, store_lists};
use crate::errors::Refused;

pub fn stop_all_voices(audio: &AudioThread) -> Result<(), Refused> {
    let mut l = lists(&audio.state);
    rc2k_core::audio::voices::stop_all(&mut GamePool::new(audio), &mut l)?;
    store_lists(&audio.state, &l);
    Ok(())
}
