//! The audio thread's sounds: sample records, voices and mixer channels (IDA, 2026-10-14).
//!
//! A sound number (what `sound::play_sound` sends) picks a sample record (`SampleRecord`,
//! 0x1CD of them at `g_sampleRecords`, 0x50A7A4). A record that is loaded points at its
//! first voice (`Voice`, a pool of 0x200 at `g_voices`, 0x4F834C, `g_voiceCount` used); a
//! sound that can sound more than once at a time has more voices, linked by `next_copy`.
//! A voice that sounds is on the playing list (`g_playingVoices`, 0x4F8340, linked by
//! `next`) and holds a mixer channel (`MixerChannel`, `g_channelCount` of them at
//! `g_channels`, 0x506354; `g_channelsUsed` held); a voice put out of use goes to the
//! stopping list (`g_stoppingVoices`, 0x4F8344) for the next turn to stop. What a command
//! changes in a voice is marked in its `dirty` bits for the turn to carry out: 1 volume,
//! 2 pan, 4 frequency, 8 place, 0x10 play, 0x20 restart.
//!
//! Since phase 12 a voice or a channel is named by its id, its place + 1 (the original's
//! pointers: `first_voice`, `source`, `next_copy`, `next`, a channel's `voice`, the lists),
//! never by an address.

use crate::audio::owner::audio;
use crate::session::Shared;

pub use rc2k_core::audio::voices::{SAMPLES, VOICES};

// The records, voices and channels and their constants: `rc2k_core::audio::voices`.
pub use rc2k_core::audio::voices::{LOADED, MODE_3D, MixerChannel, SampleRecord, Voice};

/// A dword at an address (the pointer a record or a COM call holds).
pub fn read(at: u32) -> u32 {
    crate::pointers::read_or_end::<u32>(at, "audio::voices's operand")
}

/// An id's place in a pool of `len`: ids are the place + 1 (phase 12: no addresses; 0 is
/// none, as a null pointer was). 0, or one past the pool, ends the program.
fn place_of(id: u32, len: usize, what: &'static str) -> usize {
    match (id as usize).checked_sub(1) {
        Some(n) if n < len => n,
        _ => crate::fatal::raise::end("audio::voices", crate::errors::BadInput(what)),
    }
}

/// The voice's place in the pool for id `id`.
fn voice_place(id: u32) -> &'static Shared<Voice> {
    let state = &audio().state;
    &state.voices[place_of(id, state.voices.len(), "a voice id past the pool")]
}

/// The mixer channel's place for id `id` (its place in `g_channels` + 1).
fn channel_place(id: u32) -> &'static Shared<MixerChannel> {
    let state = &audio().state;
    &state.channels[place_of(
        id,
        state.channels.len(),
        "a mixer channel id past the channels",
    )]
}

/// The sample record's place (the original indexes the table unchecked; a number past it ends
/// the program).
fn sample_place(number: u32) -> &'static Shared<SampleRecord> {
    let Some(r) = audio().sample_records.records.get(number as usize) else {
        crate::fatal::raise::end(
            "audio::voices",
            crate::errors::BadInput("a sound number past the sample records"),
        );
    };
    r
}

/// The voice of id `id` (its place in `g_voices` + 1), as it is now.
pub fn voice(id: u32) -> Voice {
    voice_place(id).get()
}

/// The voice of id `id` changed (read, changed, written back): `change` must not change it by
/// another way.
pub fn update_voice<R>(id: u32, change: impl FnOnce(&mut Voice) -> R) -> R {
    let place = voice_place(id);
    let mut v = place.get();
    let answer = change(&mut v);
    place.set(v);
    answer
}

/// The voice being changed stored now, for what reads it by address; its `buffer_3d`'s address.
pub fn stored_buffer_3d(id: u32, v: &Voice) -> u32 {
    let place = voice_place(id);
    place.set(*v);
    place.addr() + core::mem::offset_of!(Voice, buffer_3d) as u32
}

/// The id of the voice at place `n`.
pub fn voice_id(n: u32) -> u32 {
    n + 1
}

/// The mixer channel of id `id`, as it is now.
pub fn channel(id: u32) -> MixerChannel {
    channel_place(id).get()
}

/// The mixer channel of id `id` changed, as `update_voice`.
pub fn update_channel(id: u32, change: impl FnOnce(&mut MixerChannel)) {
    channel_place(id).update(change);
}

/// The sample record of sound `number`, as it is now.
pub fn sample(number: u32) -> SampleRecord {
    sample_place(number).get()
}

/// The core's `Pool` over the stores: voices and channels by their ids (place + 1).
pub struct GamePool;

impl rc2k_core::audio::voices::Pool for GamePool {
    fn voice(&self, at: u32) -> Voice {
        voice_place(at).get()
    }
    fn set_voice(&mut self, at: u32, v: Voice) {
        voice_place(at).set(v);
    }
    fn channel(&self, at: u32) -> MixerChannel {
        channel_place(at).get()
    }
    fn set_channel(&mut self, at: u32, c: MixerChannel) {
        channel_place(at).set(c);
    }
    fn channel_at(&self, n: u32) -> u32 {
        n + 1
    }
    fn record(&self, n: u32) -> SampleRecord {
        sample_place(n).get()
    }
}

/// The playing and stopping lists, the channels held and their count, read.
pub fn lists() -> rc2k_core::audio::voices::Lists {
    let state = &audio().state;
    rc2k_core::audio::voices::Lists {
        playing: state.playing_voices.get(),
        stopping: state.stopping_voices.get(),
        channels_used: state.channels_used.get(),
        channel_count: state.channel_count.get(),
    }
}

/// The lists stored back (the channel count is not written).
pub fn store_lists(l: &rc2k_core::audio::voices::Lists) {
    let audio = audio();
    audio.state.playing_voices.set(l.playing);
    audio.state.stopping_voices.set(l.stopping);
    audio.state.channels_used.set(l.channels_used);
}

#[cfg(test)]
mod id_tests {
    /// The voices and channels are named by ids since phase 12: no pointer of the image's
    /// may point into them (it would be taken for an id).
    #[test]
    fn no_pointer_of_the_image_points_at_a_voice_or_a_channel() {
        let voices = 0x4F834C..0x4F834C + 0x200 * 0x70;
        let channels = 0x506354..0x506354 + 16 * 0x58;
        for place in crate::moved::image_relocations() {
            let target = u32::from_le_bytes(
                crate::moved::image_bytes(place as usize, 4)
                    .try_into()
                    .unwrap(),
            ) as usize;
            assert!(
                !voices.contains(&target) && !channels.contains(&target),
                "{place:#x} points at {target:#x}"
            );
        }
    }
}
