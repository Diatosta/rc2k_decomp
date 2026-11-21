//! The turn's update of the voices on the game's own mixer. Each voice of the playing list with
//! dirty bits gets, on its channel, as marked: the step (frequency over the mix rate, as 32.32),
//! the left and right volumes (volume x factor, panned), or for a restart or a voice not yet
//! sounding the channel's position, format, data, length and kind. A voice that does not loop
//! whose channel the mixer ended has its channel freed and goes to the stopping list; the
//! stopping list's voices are stopped and the list emptied. Then `audio::mix`, and the playing
//! list again for the voices the mix ended. Refuses, before anything: lists without their end; a
//! playing voice without a channel; no mixing rate with a frequency to set (a division by 0); and
//! a voice whose volume or pan changes while its sound has ended (the link would be written through
//! the pan).

use crate::audio::own::{AudioOwn, AudioThread};
use crate::audio::voices::GamePool;
use crate::errors::Refused;
use rc2k_core::audio::voices::Lists;

/// The lists as the mixer's turn reads them (no channel count).
fn lists(state: &AudioOwn) -> Lists {
    Lists {
        playing: state.playing_voices.get(),
        stopping: state.stopping_voices.get(),
        channels_used: state.channels_used.get(),
        channel_count: 0,
    }
}

fn store(state: &AudioOwn, l: &Lists) {
    state.playing_voices.set(l.playing);
    state.stopping_voices.set(l.stopping);
    state.channels_used.set(l.channels_used);
}

/// The voices' turn of the mixer, the mix (`audio::mix`) between the two halves.
pub(super) fn update_mixer_voices(audio: &AudioThread) -> Result<(), Refused> {
    let state = &audio.state;
    use rc2k_core::audio::mixer_voices::{after_mix, before_mix};
    let rate = state.mix_rate.get();
    let mode = state.sound_mode.get();
    let mut pool = GamePool::new(audio);
    let mut l = lists(state);
    before_mix(&mut pool, &mut l, rate, mode)?;
    store(state, &l);
    crate::audio::mix::mix(&audio.state)?;
    let mut l = lists(state);
    after_mix(&mut pool, &mut l, rate, mode)?;
    store(state, &l);
    Ok(())
}
