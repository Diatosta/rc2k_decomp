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

use crate::framework::{game_addr, rva};

pub const SAMPLES: u32 = 0x1CD;
pub const VOICES: u32 = 0x200;

pub const SAMPLES_RVA: usize = rva(0x50A7A4); // g_sampleRecords
pub const VOICES_RVA: usize = rva(0x4F834C); // g_voices
pub const VOICE_COUNT_RVA: usize = rva(0x4F8348); // g_voiceCount
pub const PLAYING_RVA: usize = rva(0x4F8340); // g_playingVoices
pub const STOPPING_RVA: usize = rva(0x4F8344); // g_stoppingVoices
pub const CHANNELS_USED_RVA: usize = rva(0x506350); // g_channelsUsed
pub const CHANNEL_COUNT_RVA: usize = rva(0x50634C); // g_channelCount
pub const CHANNELS_RVA: usize = rva(0x506354); // g_channels
pub const SOUND_MODE_RVA: usize = rva(0x4F82BC); // g_soundMode
pub const A3D_RVA: usize = rva(0x506D60); // g_useA3d
pub const PRIMARY_RVA: usize = rva(0x506A8C); // g_primaryBuffer
pub const VOLUMES_RVA: usize = rva(0x506B4C); // g_volumeTable: 65 volumes, -10000 .. 0
pub const MASTER_VOLUME_RVA: usize = rva(0x4F82D8); // g_masterVolume
pub const LISTENER_RVA: usize = rva(0x506C58); // g_listener3d
pub const EAX_RVA: usize = rva(0x506C74); // g_eaxProperties
pub const EAX_ROOM_RVA: usize = rva(0x506C78); // g_eaxRoom: the room asked for
pub const EAX_ROOM_SET_RVA: usize = rva(0x506C7C); // g_eaxRoomSet: the room set
pub const BUFFER_LOST_RVA: usize = rva(0x4F82CC); // g_soundBufferLost
pub const COMMIT_RVA: usize = rva(0x4F82D0); // g_commit3d: 3D settings to commit
pub const SAMPLE_MEMORY_RVA: usize = rva(0x4F8300); // g_sampleMemory (a MemoryList)
/// The mixer channels that are cleared (`audio::clear_channels`).
pub const CHANNELS_CLEARED: u32 = 0x10;

/// "As the sample record says" in a command.
pub const DEFAULT: u32 = 0xFFFF_FF91;
/// "The voice's own frequency".
pub const OWN_FREQUENCY: u32 = 0xFFFF_FF90;
/// "As the sample record says", for a place's float.
pub const DEFAULT_PLACE: u32 = 0xFF80_0000;
/// `g_soundMode`: places in 3D.
pub const MODE_3D: u32 = 0x10_0000;

pub const DIRTY_VOLUME: u32 = 1;
pub const DIRTY_PAN: u32 = 2;
pub const DIRTY_FREQUENCY: u32 = 4;
pub const DIRTY_PLACE: u32 = 8;
pub const DIRTY_PLAY: u32 = 0x10;
pub const DIRTY_RESTART: u32 = 0x20;
/// `Voice::flags`: the voice's sample is loaded.
pub const LOADED: u32 = 0x10;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SampleRecord {
    pub number: u32,
    pub group: u32,
    /// The first voice (0: not loaded).
    pub first_voice: u32,
    pub data_key: u32,
    pub priority: u32,
    pub voice_kind: u32,
    /// How many voices to make for it.
    pub copies: i32,
    /// The sample's volume scale out of 0x40 (`audio::prepare_sample`).
    pub scale: u32,
    /// Points at a volume factor (0: 0x40).
    pub volume_factor: u32,
    pub field_24: u32,
    pub volume: u32,
    pub pan: u32,
    pub frequency: u32,
    pub place: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Voice {
    /// 1 a buffer of its own, 2 the software mixer's; `LOADED`.
    pub flags: u32,
    /// Its sample record's number.
    pub sound: u32,
    pub buffer: u32,
    pub buffer_3d: u32,
    pub data_key: u32,
    /// The voice whose sample data this one shares.
    pub source: u32,
    pub next_copy: u32,
    pub data: u32,
    pub length: u32,
    pub own_frequency: u32,
    /// Low four bits: the sample's shift.
    pub format: u32,
    /// The sample's frames.
    pub frames: u32,
    /// The record's `scale`.
    pub scale: u32,
    pub priority: i32,
    /// 0 once, 1 looping, 3 restarted each time.
    pub kind: u32,
    /// 0 free, 1 started, 2 sounding.
    pub state: u32,
    pub dirty: u32,
    /// Its mixer channel (0: none).
    pub channel: u32,
    /// The playing or stopping list's link.
    pub next: u32,
    pub volume_factor: u32,
    pub field_50: u32,
    pub volume: u32,
    pub pan: i32,
    pub pan_sign: i32,
    pub frequency: u32,
    pub place: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MixerChannel {
    pub used: u32,
    pub gap_4: [u32; 2],
    pub voice: u32,
    pub format: u32,
    pub data: u32,
    pub length: u32,
    pub kind: u32,
    pub gap_20: [u32; 3],
    pub position: u32,
    pub position_fraction: u32,
    pub step: u32,
    pub step_fraction: u32,
    pub left: i32,
    pub right: i32,
    pub gap_44: [u32; 5],
}

const _: () = assert!(std::mem::size_of::<SampleRecord>() == 0x40);
const _: () = assert!(std::mem::size_of::<Voice>() == 0x70);
const _: () = assert!(std::mem::size_of::<MixerChannel>() == 0x58);

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
        let a = |r| game_addr(r) as u32;
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

    /// The game's address of each, as the original's code holds it, with this world's.
    pub fn pairs(&self) -> [(u32, u32); 7] {
        let game = World::game();
        [
            (game.samples, self.samples),
            (game.playing, self.playing),
            (game.stopping, self.stopping),
            (game.channels_used, self.channels_used),
            (game.channel_count, self.channel_count),
            (game.channels, self.channels),
            (game.sound_mode, self.sound_mode),
        ]
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
