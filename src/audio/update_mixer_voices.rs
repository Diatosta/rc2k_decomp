//! `audio::update_mixer_voices` (0x40BC05, a chunk of 0x40AC77 entered by `jnz`): the
//! turn's update of the voices on the game's own mixer.
//!
//! The playing list: a voice with dirty bits gets, on its channel, as marked: the step
//! (frequency `>>` the halvings, over `g_mixRate`, as 32.32) for 4; the left and right
//! volumes for 1 or 2 (volume x factor, times 64 - pan and 64 for a pan of 0 and up, 64 and
//! 64 + pan below, each `>> 6`; with mode bit 0x2000000 and a negative pan sign the right
//! negated); for a restart (0x20) or a voice not yet sounding, the channel's position and
//! fraction 0 and its format, data, length (the voice's frames) and kind, the voice's
//! state 2; dirty bits 0. A voice that does not loop whose channel is no longer used (the
//! mixer ended it) has its channel freed and 0, goes to the stopping list's front, and
//! `g_channelsUsed` is one less. The stopping list: each voice's dirty bits + 0xF, state
//! 0, channel freed and 0, and the list emptied. `audio::mix`. The playing list again for
//! the voices the mix ended.
//! Refuses, before anything: lists without their end; a playing voice without a channel; no
//! mixing rate with a frequency to set (the original divides by 0); and a voice whose
//! volume or pan changes while its sound has ended: the original then mistakes the pan for
//! the list's link and writes the link through it (section 16).

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

/// `mix`: `audio::mix`.
pub fn update_mixer_voices(
    audio: &AudioThread,
    mix: &mut dyn FnMut() -> Result<(), Refused>,
) -> Result<(), Refused> {
    let state = &audio.state;
    // Converted: `rc2k_core::audio::mixer_voices`; the lists stored for the mix, which
    // reads them, and read again after it.
    use rc2k_core::audio::mixer_voices::{after_mix, before_mix};
    let rate = state.mix_rate.get();
    let mode = state.sound_mode.get();
    let mut pool = GamePool::new(audio);
    let mut l = lists(state);
    before_mix(&mut pool, &mut l, rate, mode)?;
    store(state, &l);
    mix()?;
    let mut l = lists(state);
    after_mix(&mut pool, &mut l, rate, mode)?;
    store(state, &l);
    Ok(())
}

/// The voices' turn of the game's own mixer, the mix (`audio::mix`) in it.
pub fn update_mixer_voices_in_game(audio: &AudioThread) -> Result<(), Refused> {
    let mut mix = || -> Result<(), Refused> {
        crate::audio::mix::mix(&audio.state)?;
        Ok(())
    };
    update_mixer_voices(audio, &mut mix)
}
