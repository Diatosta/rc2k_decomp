//! Command 1 on the audio thread: a sound started, changed or stopped (see `audio/voices.rs`).
//! Nothing for a negative number, a sound not loaded, or one whose first voice is not.
//!
//! With a volume and a voice of kind 0 (or, for another kind or no volume, a voice not playing,
//! unless the command is only_if_playing): the first of its copies that is free gets a channel, a
//! free one while channels are left, else the channel of the playing voice of the lowest
//! priority below its own, which goes to the stopping list; the voice goes to the front of the
//! playing list. A playing voice with no volume is stopped. A voice started or playing then gets
//! its volume (held to 0x40), pan (-0x40 .. 0x40), frequency (100 .. 200,000) and, in 3D, its
//! place, each marked dirty if it changed; kind 3 marks a restart; and the play bit is set.
//!
//! Refuses, before anything is written: a number of 0x1CD or more; no channels left and no voice
//! playing; a playing list that does not end within `VOICES` voices; a playing voice to stop without
//! its channel.

use crate::audio::own::AudioThread;
use crate::audio::voices::{GamePool, lists, store_lists};
use crate::errors::Refused;
use rc2k_core::audio::commands::Start;

pub fn start_voice(audio: &AudioThread, sound: i32, cmd: &Start) -> Result<(), Refused> {
    let mut l = lists(&audio.state);
    let mode = audio.state.sound_mode.get();
    rc2k_core::audio::voices::start_voice(&mut GamePool::new(audio), &mut l, mode, sound, cmd)?;
    store_lists(&audio.state, &l);
    Ok(())
}
