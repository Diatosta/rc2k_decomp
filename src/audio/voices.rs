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

use crate::audio::store::SampleSlot;
use crate::audio::store::Slot;
use crate::store::addr32 as a;

pub use rc2k_core::audio::voices::{SAMPLES, VOICES};

pub const SAMPLES_RVA: SampleSlot = SampleSlot::at(0x50A7A4); // g_sampleRecords
pub const VOICES_RVA: Slot = Slot::at(0x4F834C); // g_voices
pub const VOICE_COUNT_RVA: Slot = Slot::at(0x4F8348); // g_voiceCount
pub const PLAYING_RVA: Slot = Slot::at(0x4F8340); // g_playingVoices
pub const STOPPING_RVA: Slot = Slot::at(0x4F8344); // g_stoppingVoices
pub const CHANNELS_USED_RVA: Slot = Slot::at(0x506350); // g_channelsUsed
pub const CHANNEL_COUNT_RVA: Slot = Slot::at(0x50634C); // g_channelCount
pub const CHANNELS_RVA: Slot = Slot::at(0x506354); // g_channels
pub const SOUND_MODE_RVA: Slot = Slot::at(0x4F82BC); // g_soundMode
pub const A3D_RVA: Slot = Slot::at(0x506D60); // g_useA3d
pub const PRIMARY_RVA: Slot = Slot::at(0x506A8C); // g_primaryBuffer
pub const VOLUMES_RVA: Slot = Slot::at(0x506B4C); // g_volumeTable: 65 volumes, -10000 .. 0
pub const MASTER_VOLUME_RVA: Slot = Slot::at(0x4F82D8); // g_masterVolume
pub const VOICE_MANAGER_RVA: Slot = Slot::at(0x506C58); // g_voiceManager: the voice manager's IKsPropertySet
pub const EAX_RVA: Slot = Slot::at(0x506C74); // g_eaxProperties
pub const EAX_ROOM_RVA: Slot = Slot::at(0x506C78); // g_eaxRoom: the room asked for
pub const EAX_ROOM_SET_RVA: Slot = Slot::at(0x506C7C); // g_eaxRoomSet: the room set
pub const BUFFER_LOST_RVA: Slot = Slot::at(0x4F82CC); // g_soundBufferLost
pub const COMMIT_RVA: Slot = Slot::at(0x4F82D0); // g_commit3d: 3D settings to commit
pub const SAMPLE_MEMORY_RVA: Slot = Slot::at(0x4F8300); // g_sampleMemory (a MemoryList)
// The records, voices and channels and their constants: `rc2k_core::audio::voices`.
pub use rc2k_core::audio::voices::{LOADED, MODE_3D, MixerChannel, SampleRecord, Voice};

/// Where the audio thread's tables and lists are: the game's, or a test's.
#[derive(Clone, Copy, Debug)]
pub struct World {
    pub samples: u32,
    pub playing: u32,
    pub stopping: u32,
    pub channels_used: u32,
    pub channel_count: u32,
    pub channels: u32,
    pub sound_mode: u32,
}

impl World {
    pub fn game() -> Self {
        Self {
            samples: a(SAMPLES_RVA),
            playing: a(PLAYING_RVA),
            stopping: a(STOPPING_RVA),
            channels_used: a(CHANNELS_USED_RVA),
            channel_count: a(CHANNEL_COUNT_RVA),
            channels: a(CHANNELS_RVA),
            sound_mode: a(SOUND_MODE_RVA),
        }
    }
}

/// A dword of the world.
pub unsafe fn read(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

pub unsafe fn write(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) }
}

pub unsafe fn voice<'a>(at: u32) -> &'a mut Voice {
    unsafe { &mut *(at as *mut Voice) }
}

pub unsafe fn sample<'a>(w: &World, number: u32) -> &'a mut SampleRecord {
    unsafe { &mut *((w.samples as *mut SampleRecord).add(number as usize)) }
}

pub unsafe fn channel<'a>(at: u32) -> &'a mut MixerChannel {
    unsafe { &mut *(at as *mut MixerChannel) }
}

/// The core's `Pool` over a world's memory: voices and channels by their addresses.
pub struct GamePool {
    samples: u32,
    channels: u32,
}

impl GamePool {
    /// `w`'s tables; every handle the core is given must be a voice or channel of the
    /// world (the lists and records hold nothing else).
    pub unsafe fn new(w: &World) -> Self {
        GamePool {
            samples: w.samples,
            channels: w.channels,
        }
    }
}

impl rc2k_core::audio::voices::Pool for GamePool {
    fn voice(&mut self, at: u32) -> &mut Voice {
        unsafe { voice(at) }
    }
    fn channel(&mut self, at: u32) -> &mut MixerChannel {
        unsafe { channel(at) }
    }
    fn channel_at(&self, n: u32) -> u32 {
        self.channels + n * std::mem::size_of::<MixerChannel>() as u32
    }
    fn record(&mut self, n: u32) -> &mut SampleRecord {
        unsafe { &mut *((self.samples as *mut SampleRecord).add(n as usize)) }
    }
}

/// A world's lists, read.
pub unsafe fn lists(w: &World) -> rc2k_core::audio::voices::Lists {
    unsafe {
        rc2k_core::audio::voices::Lists {
            playing: read(w.playing),
            stopping: read(w.stopping),
            channels_used: read(w.channels_used),
            channel_count: read(w.channel_count),
        }
    }
}

/// A world's lists stored back (the channel count is not written).
pub unsafe fn store_lists(w: &World, l: &rc2k_core::audio::voices::Lists) {
    unsafe {
        write(w.playing, l.playing);
        write(w.stopping, l.stopping);
        write(w.channels_used, l.channels_used);
    }
}
