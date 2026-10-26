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

use crate::audio::store::{AUDIO, SAMPLE_RECORDS};

pub use rc2k_core::audio::voices::{SAMPLES, VOICES};

// The records, voices and channels and their constants: `rc2k_core::audio::voices`.
pub use rc2k_core::audio::voices::{LOADED, MODE_3D, MixerChannel, SampleRecord, Voice};

/// A dword at an address (the pointer a record or a COM call holds).
pub fn read(at: u32) -> u32 {
    crate::pointers::read_or_end::<u32>(at, "audio::voices's operand")
}

/// A record of `T` at `at`, once the address is known to hold one (inside one region,
/// aligned); the program ends with the address otherwise.
fn record_at<'a, T>(at: u32, what: &str) -> &'a mut T {
    crate::pointers::check_or_end(at, size_of::<T>(), what);
    if !(at as usize).is_multiple_of(align_of::<T>()) {
        crate::fatal::raise::end(
            "audio::voices",
            crate::errors::BadInput("a record at an address out of line"),
        );
    }
    // Safety: `at` .. `at + size_of::<T>()` lies in one live region and is aligned; the
    // records are the audio thread's (its lock held), as the original's are.
    unsafe { &mut *(crate::pointers::host(at) as *mut T) }
}

pub fn voice<'a>(at: u32) -> &'a mut Voice {
    record_at(at, "a voice")
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
    record_at(crate::pointers::game_address(r.as_ptr()), "a sample record")
}

pub fn channel<'a>(at: u32) -> &'a mut MixerChannel {
    record_at(at, "a mixer channel")
}

/// The core's `Pool` over the game's memory: voices and channels by their addresses.
pub struct GamePool;

impl rc2k_core::audio::voices::Pool for GamePool {
    fn voice(&mut self, at: u32) -> &mut Voice {
        voice(at)
    }
    fn channel(&mut self, at: u32) -> &mut MixerChannel {
        channel(at)
    }
    fn channel_at(&self, n: u32) -> u32 {
        AUDIO.channels[0].addr() + n * std::mem::size_of::<MixerChannel>() as u32
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
