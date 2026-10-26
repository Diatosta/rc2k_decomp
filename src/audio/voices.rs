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

use crate::audio::store::{AUDIO, SAMPLE_RECORDS};

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

/// The voice of id `id` (its place in `g_voices` + 1).
pub fn voice<'a>(id: u32) -> &'a mut Voice {
    let n = place_of(id, AUDIO.voices.len(), "a voice id past the pool");
    // SAFETY: the audio thread's records (its lock held), one borrowed at a time, as the
    // original's are.
    unsafe { AUDIO.voices[n].borrow() }
}

/// The id of the voice at place `n`.
pub fn voice_id(n: u32) -> u32 {
    n + 1
}

/// The mixer channel of id `id` (its place in `g_channels` + 1).
pub fn channel<'a>(id: u32) -> &'a mut MixerChannel {
    let n = place_of(
        id,
        AUDIO.channels.len(),
        "a mixer channel id past the channels",
    );
    // SAFETY: as `voice`.
    unsafe { AUDIO.channels[n].borrow() }
}

/// The sample record of sound `number` (the original indexes the table unchecked; a
/// number past it ends the program).
pub fn sample<'a>(number: u32) -> &'a mut SampleRecord {
    let Some(r) = SAMPLE_RECORDS.records.get(number as usize) else {
        crate::fatal::raise::end(
            "audio::voices",
            crate::errors::BadInput("a sound number past the sample records"),
        );
    };
    // SAFETY: as `voice`.
    unsafe { r.borrow() }
}

/// The core's `Pool` over the stores: voices and channels by their ids (place + 1).
pub struct GamePool;

impl rc2k_core::audio::voices::Pool for GamePool {
    fn voice(&mut self, at: u32) -> &mut Voice {
        voice(at)
    }
    fn channel(&mut self, at: u32) -> &mut MixerChannel {
        channel(at)
    }
    fn channel_at(&self, n: u32) -> u32 {
        n + 1
    }
    fn record(&mut self, n: u32) -> &mut SampleRecord {
        sample(n)
    }
}

/// The playing and stopping lists, the channels held and their count, read.
pub fn lists() -> rc2k_core::audio::voices::Lists {
    rc2k_core::audio::voices::Lists {
        playing: AUDIO.playing_voices.get(),
        stopping: AUDIO.stopping_voices.get(),
        channels_used: AUDIO.channels_used.get(),
        channel_count: AUDIO.channel_count.get(),
    }
}

/// The lists stored back (the channel count is not written).
pub fn store_lists(l: &rc2k_core::audio::voices::Lists) {
    AUDIO.playing_voices.set(l.playing);
    AUDIO.stopping_voices.set(l.stopping);
    AUDIO.channels_used.set(l.channels_used);
}

#[cfg(test)]
mod id_tests {
    /// The voices and channels are named by ids since phase 12: no pointer of the image's
    /// may point into them (it would be taken for an id).
    #[test]
    fn no_pointer_of_the_image_points_at_a_voice_or_a_channel() {
        let voices = 0x4F834C..0x4F834C + 0x200 * 0x70;
        let channels = 0x506354..0x506354 + 16 * 0x58;
        for &place in crate::relocations::PLACES.iter() {
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
