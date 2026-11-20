//! The audio thread's sounds: sample records, voices and mixer channels. A sound number (what
//! `sound::play_sound` sends) picks a sample record (`SampleRecord`, 0x1CD of them). A record that
//! is loaded points at its first voice (`Voice`, a pool of 0x200); a sound that can sound more than
//! once at a time has more voices, linked by `next_copy`. A voice that sounds is on the playing
//! list (linked by `next`) and holds a mixer channel (`MixerChannel`); a voice put out of use goes
//! to the stopping list for the next turn to stop. What a command changes in a voice is marked in
//! its `dirty` bits for the turn to carry out: 1 volume, 2 pan, 4 frequency, 8 place, 0x10 play,
//! 0x20 restart.

use crate::audio::own::{AudioOwn, AudioThread};
use crate::audio::sample_records::SampleRecords;
use crate::audio::store::AudioShared;
use crate::memory::session::{Plain, Shared};

pub use rc2k_core::audio::voices::{SAMPLES, VOICES};

// The records, voices and channels and their constants: `rc2k_core::audio::voices`.
pub use rc2k_core::audio::voices::{LOADED, MODE_3D, MixerChannel, SampleRecord, Voice};

/// An id's place in a pool of `len`: ids are the place + 1 (0 is none). 0, or one past the pool,
/// ends the program.
fn place_of(id: u32, len: usize, what: &'static str) -> usize {
    match (id as usize).checked_sub(1) {
        Some(n) if n < len => n,
        _ => crate::fatal::raise::end("audio::voices", crate::errors::BadInput(what)),
    }
}

/// The voice's place in the pool for id `id`.
fn voice_place(state: &AudioShared, id: u32) -> &Shared<Voice> {
    &state.voices[place_of(id, state.voices.len(), "a voice id past the pool")]
}

/// The mixer channel's place for id `id` (its place + 1).
fn channel_place(state: &AudioOwn, id: u32) -> &Plain<MixerChannel> {
    &state.channels[place_of(
        id,
        state.channels.len(),
        "a mixer channel id past the channels",
    )]
}

/// The sample record of sound `number`; a number past the table ends the program.
fn sample_of(records: &SampleRecords, number: u32) -> SampleRecord {
    let Some(r) = records.record(number) else {
        crate::fatal::raise::end(
            "audio::voices",
            crate::errors::BadInput("a sound number past the sample records"),
        );
    };
    r
}

/// The voice of id `id`, as it is now.
pub fn voice_of(state: &AudioShared, id: u32) -> Voice {
    voice_place(state, id).get()
}

/// The voice of id `id` changed (read, changed, written back): `change` must not change it by
/// another way.
pub fn update_voice<R>(state: &AudioShared, id: u32, change: impl FnOnce(&mut Voice) -> R) -> R {
    let place = voice_place(state, id);
    let mut v = place.get();
    let answer = change(&mut v);
    place.set(v);
    answer
}

/// The voice being changed stored now, for the property sets' openings that read its `buffer_3d`
/// from the pool.
pub(super) fn stored_buffer_3d(
    state: &AudioOwn,
    id: u32,
    v: &Voice,
) -> crate::audio::property_sets::Holder {
    voice_place(state, id).set(*v);
    crate::audio::property_sets::Holder::Buffer3d(id)
}

/// The id of the voice at place `n`.
pub(super) fn voice_id(n: u32) -> u32 {
    n + 1
}

/// The mixer channel of id `id`, as it is now.
pub fn channel(state: &AudioOwn, id: u32) -> MixerChannel {
    channel_place(state, id).get()
}

/// The mixer channel of id `id` changed, as `update_voice`.
pub fn update_channel(state: &AudioOwn, id: u32, change: impl FnOnce(&mut MixerChannel)) {
    channel_place(state, id).update(|mut channel| {
        change(&mut channel);
        channel
    });
}

/// The sample record of sound `number`, as it is now.
pub fn sample(records: &SampleRecords, number: u32) -> SampleRecord {
    sample_of(records, number)
}

/// The core's `Pool` over the stores: voices and channels by their ids (place + 1).
pub(super) struct GamePool<'a> {
    pub state: &'a AudioOwn,
    pub records: &'a SampleRecords,
}

impl<'a> GamePool<'a> {
    /// The pool over the audio's stores.
    pub fn new(audio: &'a AudioThread) -> Self {
        GamePool {
            state: &audio.state,
            records: &audio.sample_records,
        }
    }
}

impl rc2k_core::audio::voices::Pool for GamePool<'_> {
    fn voice(&self, at: u32) -> Voice {
        voice_place(self.state, at).get()
    }
    fn set_voice(&mut self, at: u32, v: Voice) {
        voice_place(self.state, at).set(v);
    }
    fn channel(&self, at: u32) -> MixerChannel {
        channel_place(self.state, at).get()
    }
    fn set_channel(&mut self, at: u32, c: MixerChannel) {
        channel_place(self.state, at).set(c);
    }
    fn channel_at(&self, n: u32) -> u32 {
        n + 1
    }
    fn record(&self, n: u32) -> SampleRecord {
        sample_of(self.records, n)
    }
}

/// The playing and stopping lists, the channels held and their count, read.
pub fn lists(state: &AudioOwn) -> rc2k_core::audio::voices::Lists {
    rc2k_core::audio::voices::Lists {
        playing: state.playing_voices.get(),
        stopping: state.stopping_voices.get(),
        channels_used: state.channels_used.get(),
        channel_count: state.channel_count.get(),
    }
}

/// The lists stored back (the channel count is not written).
pub(super) fn store_lists(state: &AudioOwn, l: &rc2k_core::audio::voices::Lists) {
    state.playing_voices.set(l.playing);
    state.stopping_voices.set(l.stopping);
    state.channels_used.set(l.channels_used);
}
