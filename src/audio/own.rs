//! The audio thread's own variables: what only the audio thread touches (its mixer, voice
//! lists, device objects, sample work buffers and set-up state), plain cells in a record the
//! thread's loop owns (`audio::thread::audio_thread` makes it, a test makes its own), so that
//! the game's thread cannot reach it. What both threads touch is `store::AudioShared`.
//!
//! Which thread touches which field (read by grep of every access, round 56; T0 = the game's
//! thread, T1 = the audio thread's loop and what it calls; the SDL callback of
//! `platform_sound` touches none of them):
//! - T1 only, here: the mixer (`channels`, `channel_count`, `mix_*`, the cursors), the
//!   voice lists (`playing_voices`, `stopping_voices`), the sample work buffers and files,
//!   the stream files' header, the DirectSound / A3D / EAX objects and ids, the buffer and
//!   wave format descriptions, `volume_table`, `sound_buffer_lost`, `commit_3d`,
//!   `audio_turn_tick`, `opened_sound_mode`, `sound_list_loaded`, `next_tune`, `music_path`.
//! - Both, in `AudioShared`: written by T0 (settings, menus) and read by T1 (`no_audio`,
//!   `menu_sound_mode`, `race_sound_mode`, `master_volume`, `audio_options`, `tune_turn`, the
//!   levels), written by T1 and read by T0 (`sound_mode`, `has_3d`, `has_eax`, `has_a3d2`,
//!   `sound_caps`, `voice_manager_on`, the two streams' players, `voices` (`sample_settings`),
//!   `channels_used` and `voice_count` (the debug window)), and the set-up values T0 reads
//!   from the settings file before the audio thread starts and T1 reads after it
//!   (`mix_rate_setting`, `sample_buffer_kb`, `no_music`, `no_listener`, the `ignore*`
//!   settings): shared, the "before the thread starts" order is not enforced anywhere.
//!   `sounds_held_back` is T0-only today (the census found a T1 read; there is none), kept
//!   shared with the others.
//! - `AudioOwn` reaches the shared part as a reference and derefs to it, so the audio
//!   thread's code reads both through one `state`.
//!
//! The records (`ListCells`, `Shared<MixerChannel>`, ...) keep their `Shared` cells: the
//! memory and file helpers take them as such.

use crate::audio::describe_wave_format::WaveFormat;
use crate::audio::make_buffer::BufferDescription;
use crate::audio::owner::Audio;
use crate::audio::samples::SampleFile;
use crate::audio::store::{AudioShared, CHANNEL_IDLE};
use crate::memory::{ListCells, MemoryList};
use crate::session::{Plain, PlainBytes, PlainDword};
use rc2k_core::audio::voices::MixerChannel;

/// The variables only the audio thread touches (`AudioThread`'s, made by the thread's own loop).
pub struct AudioOwn {
    /// The shared part, which `AudioOwn` reaches as a reference (and derefs to).
    shared: &'static AudioShared,
    /// `g_audioTurnTick`: the clock reading of the audio thread's last turn. (IDA 0x4F82B8)
    pub audio_turn_tick: PlainDword,
    /// `g_openedSoundMode`: the mode DirectSound was opened with. (IDA 0x4F82C0)
    pub opened_sound_mode: PlainDword,
    /// `g_soundBufferLost`: 1 when a DirectSound buffer was lost (the thread restores them). (IDA 0x4F82CC)
    pub sound_buffer_lost: PlainDword,
    /// `g_commit3d`: 1 when 3D settings are waiting for the listener's commit. (IDA 0x4F82D0)
    pub commit_3d: PlainDword,
    /// `g_primaryVolumeKept`: the primary buffer's volume kept at start (1: none). (IDA 0x4F82DC)
    pub primary_volume_kept: PlainDword,
    /// `g_sampleBitsFlags`: {format flag, bits} twice: 0x10 eight bits, 0x20 sixteen. (IDA 0x4F82E0)
    pub sample_bits_flags: [PlainDword; 4],
    /// `g_sampleChannelFlags`: {format flag, channels} twice: 0x100 mono, 0x200 stereo. (IDA 0x4F82F0)
    pub sample_channel_flags: [PlainDword; 4],
    /// `g_sampleMemory`: the samples' memory (a `MemoryList`). (IDA 0x4F8300)
    pub sample_memory: ListCells,
    /// `g_playingVoices`: the first voice on the playing list (an address; 0: none). (IDA 0x4F8340)
    pub playing_voices: PlainDword,
    /// `g_stoppingVoices`: the first voice on the stopping list. (IDA 0x4F8344)
    pub stopping_voices: PlainDword,
    /// `g_channelCount`: the mixer's channels (8 at start). (IDA 0x50634C)
    pub channel_count: PlainDword,
    /// `g_channels`: the mixer's channels. (IDA 0x506354)
    pub channels: [Plain<MixerChannel>; 16],
    /// `g_mixRate`: the mixer's rate (the primary buffer's). (IDA 0x5068D4)
    pub mix_rate: PlainDword,
    /// `g_primarySize`: the primary buffer's bytes. (IDA 0x5068D8)
    pub primary_size: PlainDword,
    /// `g_mixMost`: the most a turn mixes, 40 ms of bytes. (IDA 0x5068DC)
    pub mix_most: PlainDword,
    /// `g_mixShift`: bytes to frames (a shift). (IDA 0x5068E0)
    pub mix_shift: PlainDword,
    /// `g_playCursor`: the primary buffer's play cursor. (IDA 0x5068E4)
    pub play_cursor: PlainDword,
    /// `g_writeCursor`: the mixer's write place. (IDA 0x5068E8)
    pub write_cursor: PlainDword,
    /// `g_sampleBufferMemory`: the work buffer's memory (a `MemoryList`). (IDA 0x5068F0)
    pub sample_buffer_memory: ListCells,
    /// `g_sampleBuffer`: the work buffer for reading samples (an address). (IDA 0x506930)
    pub sample_buffer: PlainDword,
    /// `g_sampleBufferSize`: its bytes. (IDA 0x506934)
    pub sample_buffer_size: PlainDword,
    /// `g_sampleFile`: what the last sample file read held. (IDA 0x506938)
    pub sample_file: Plain<SampleFile>,
    /// `dword_506958`: 0x40, the volume factor a sample record's `volume_factor` points at. (IDA 0x506958)
    pub unit_volume_factor: PlainDword,
    /// `g_soundListLoaded`: the sound list loaded (command 3), 0 after it is freed. (IDA 0x50695C)
    pub sound_list_loaded: PlainDword,
    /// `g_nextTune`: the tune played when the music stops. (IDA 0x50698C)
    pub next_tune: PlainDword,
    /// `g_musicPath`: `;3files\music\00.wav`, the tune's two digits are put at +0xE. (IDA 0x5069C0)
    pub music_path: PlainBytes<24>,
    /// `g_streamHeader`: the 0xC00 bytes a stream's file head is read into (an address; 0: not taken yet). (IDA 0x5069D8)
    pub stream_header: PlainDword,
    /// `g_streamHeaderMemory`: the header's memory (a `MemoryList`). (IDA 0x5069DC)
    pub stream_header_memory: ListCells,
    /// `riid`: A3D's interface id. (IDA 0x506A28)
    pub a3d_iid: PlainBytes<16>,
    /// `g_iidListener3d`: the 3D listener's interface id. (IDA 0x506A48)
    pub iid_listener: PlainBytes<16>,
    /// `g_iidSound3dBuffer`: the 3D buffer's interface id. (IDA 0x506A58)
    pub iid_sound_3d_buffer: PlainBytes<16>,
    /// `g_iidKsPropertySet`: the property set's interface id. (IDA 0x506A78)
    pub iid_property_set: PlainBytes<16>,
    /// `g_directSound`: the IDirectSound object (an address; 0: none). (IDA 0x506A88)
    pub direct_sound: PlainDword,
    /// `g_primaryBuffer`: the primary buffer. (IDA 0x506A8C)
    pub primary_buffer: PlainDword,
    /// `g_soundListener`: the 3D listener. (IDA 0x506A90)
    pub sound_listener: PlainDword,
    /// `g_lockedParts`: what `Lock` gives: two pointers and their sizes. (IDA 0x506A98)
    pub locked_parts: [PlainDword; 4],
    /// `g_primaryCaps`: the primary buffer's `DSBCAPS` (its bytes at +8). (IDA 0x506B08)
    pub primary_caps: PlainBytes<20>,
    /// `g_bufferDescription`: the `DSBUFFERDESC` a buffer is made with. (IDA 0x506B1C)
    pub buffer_description: Plain<BufferDescription>,
    /// `g_waveFormat`: the `WAVEFORMATEX` a buffer is made with. (IDA 0x506B30)
    pub wave_format: Plain<WaveFormat>,
    /// `g_volumeTable`: 65 volumes, -10000 .. 0 (signed). (IDA 0x506B4C)
    pub volume_table: [PlainDword; 65],
    /// `g_voiceManager`: the voice manager's IKsPropertySet. (IDA 0x506C58)
    pub voice_manager: PlainDword,
    /// `g_a3dSplashKept`: 1 = the splash pair acts. (IDA 0x506C60)
    pub a3d_splash_kept: PlainDword,
    /// `g_a3dReady`. (IDA 0x506C64)
    pub a3d_ready: PlainDword,
    /// `g_a3dObject`. (IDA 0x506C68)
    pub a3d_object: PlainDword,
    /// `g_eaxProperties`: EAX's IKsPropertySet. (IDA 0x506C74)
    pub eax_properties: PlainDword,
    /// `g_eaxRoom`: the room asked for (6 at start). (IDA 0x506C78)
    pub eax_room: PlainDword,
    /// `g_eaxRoomSet`: the room set (6 at start). (IDA 0x506C7C)
    pub eax_room_set: PlainDword,
    /// `g_eaxEnvironmentSet`: the race's environment (`ENVIRONMENT`) last set (1.0 at start). (IDA 0x506C80)
    pub environment_kept: PlainDword,
    /// `rclsid`: A3D's class id. (IDA 0x506CD8)
    pub a3d_clsid: PlainBytes<16>,
    /// `g_a3d2Interface`. (IDA 0x506D58)
    pub a3d2_interface: PlainDword,
    /// `g_useA3d`: 1 = A3D is the sound. (IDA 0x506D60)
    pub use_a3d: PlainDword,
    /// A3D2's class id. (IDA 0x506D6C)
    pub a3d2_clsid: PlainBytes<16>,
    /// A3D2's interface id. (IDA 0x506D8C)
    pub a3d2_iid: PlainBytes<16>,
}

impl AudioOwn {
    pub fn new(shared: &'static AudioShared) -> Self {
        AudioOwn {
            shared,
            audio_turn_tick: PlainDword::new(0),
            opened_sound_mode: PlainDword::new(0),
            sound_buffer_lost: PlainDword::new(0),
            commit_3d: PlainDword::new(0),
            primary_volume_kept: PlainDword::new(0x1),
            sample_bits_flags: [
                PlainDword::new(0x10),
                PlainDword::new(0x8),
                PlainDword::new(0x20),
                PlainDword::new(0x10),
            ],
            sample_channel_flags: [
                PlainDword::new(0x100),
                PlainDword::new(0x1),
                PlainDword::new(0x200),
                PlainDword::new(0x2),
            ],
            sample_memory: ListCells::new(MemoryList {
                head: 0x0,
                total: 0x0,
                cursor: 0x0,
                remaining: 0x0,
            }),
            playing_voices: PlainDword::new(0),
            stopping_voices: PlainDword::new(0),
            channel_count: PlainDword::new(0x8),
            channels: [const { Plain::new(CHANNEL_IDLE) }; 16],
            mix_rate: PlainDword::new(0),
            primary_size: PlainDword::new(0x2000),
            mix_most: PlainDword::new(0x1000),
            mix_shift: PlainDword::new(0),
            play_cursor: PlainDword::new(0),
            write_cursor: PlainDword::new(0),
            sample_buffer_memory: ListCells::new(MemoryList {
                head: 0x0,
                total: 0x0,
                cursor: 0x0,
                remaining: 0x0,
            }),
            sample_buffer: PlainDword::new(0),
            sample_buffer_size: PlainDword::new(0),
            sample_file: Plain::new(SampleFile {
                data: 0x0,
                length: 0x0,
                format: 0x0,
                channels: 0x0,
                rate: 0x0,
                bits: 0x0,
                frame_bytes: 0x0,
                frames: 0x0,
            }),
            unit_volume_factor: PlainDword::new(0x40),
            sound_list_loaded: PlainDword::new(0),
            next_tune: PlainDword::new(0),
            music_path: PlainBytes::new([
                0x3b, 0x33, 0x66, 0x69, 0x6c, 0x65, 0x73, 0x5c, 0x6d, 0x75, 0x73, 0x69, 0x63, 0x5c,
                0x30, 0x30, 0x2e, 0x77, 0x61, 0x76, 0, 0, 0, 0,
            ]),
            stream_header: PlainDword::new(0),
            stream_header_memory: ListCells::new(MemoryList {
                head: 0x0,
                total: 0x0,
                cursor: 0x0,
                remaining: 0x0,
            }),
            a3d_iid: PlainBytes::new([
                0x83, 0xfa, 0x9a, 0x27, 0x81, 0x49, 0xce, 0x11, 0xa5, 0x21, 0, 0x20, 0xaf, 0xb,
                0xe5, 0x60,
            ]),
            iid_listener: PlainBytes::new([
                0x84, 0xfa, 0x9a, 0x27, 0x81, 0x49, 0xce, 0x11, 0xa5, 0x21, 0, 0x20, 0xaf, 0xb,
                0xe5, 0x60,
            ]),
            iid_sound_3d_buffer: PlainBytes::new([
                0x86, 0xfa, 0x9a, 0x27, 0x81, 0x49, 0xce, 0x11, 0xa5, 0x21, 0, 0x20, 0xaf, 0xb,
                0xe5, 0x60,
            ]),
            iid_property_set: PlainBytes::new([
                0x30, 0xac, 0xef, 0x31, 0x5c, 0x51, 0xd0, 0x11, 0xa9, 0xaa, 0, 0xaa, 0, 0x61, 0xbe,
                0x93,
            ]),
            direct_sound: PlainDword::new(0),
            primary_buffer: PlainDword::new(0),
            sound_listener: PlainDword::new(0),
            locked_parts: [const { PlainDword::new(0) }; 4],
            primary_caps: PlainBytes::new([0; 20]),
            buffer_description: Plain::new(BufferDescription {
                size: 0x0,
                flags: 0x0,
                bytes: 0x0,
                reserved: 0x0,
                format: 0x0,
            }),
            wave_format: Plain::new(WaveFormat {
                tag: 0x0,
                channels: 0x0,
                samples_per_second: 0x0,
                bytes_per_second: 0x0,
                block_align: 0x0,
                bits: 0x0,
                extra: 0x0,
            }),
            volume_table: [
                PlainDword::new(0xffffd8f0),
                PlainDword::new(0xfffff1e4),
                PlainDword::new(0xfffff43e),
                PlainDword::new(0xfffff59e),
                PlainDword::new(0xfffff698),
                PlainDword::new(0xfffff75a),
                PlainDword::new(0xfffff7f8),
                PlainDword::new(0xfffff87e),
                PlainDword::new(0xfffff8f2),
                PlainDword::new(0xfffff958),
                PlainDword::new(0xfffff9b4),
                PlainDword::new(0xfffffa06),
                PlainDword::new(0xfffffa52),
                PlainDword::new(0xfffffa98),
                PlainDword::new(0xfffffad8),
                PlainDword::new(0xfffffb14),
                PlainDword::new(0xfffffb4c),
                PlainDword::new(0xfffffb81),
                PlainDword::new(0xfffffbb2),
                PlainDword::new(0xfffffbe1),
                PlainDword::new(0xfffffc0e),
                PlainDword::new(0xfffffc38),
                PlainDword::new(0xfffffc62),
                PlainDword::new(0xfffffc87),
                PlainDword::new(0xfffffcac),
                PlainDword::new(0xfffffcd0),
                PlainDword::new(0xfffffcf2),
                PlainDword::new(0xfffffd12),
                PlainDword::new(0xfffffd32),
                PlainDword::new(0xfffffd50),
                PlainDword::new(0xfffffd6e),
                PlainDword::new(0xfffffd8a),
                PlainDword::new(0xfffffda6),
                PlainDword::new(0xfffffdc1),
                PlainDword::new(0xfffffddb),
                PlainDword::new(0xfffffdf4),
                PlainDword::new(0xfffffe0c),
                PlainDword::new(0xfffffe24),
                PlainDword::new(0xfffffe3b),
                PlainDword::new(0xfffffe52),
                PlainDword::new(0xfffffe68),
                PlainDword::new(0xfffffe7d),
                PlainDword::new(0xfffffe92),
                PlainDword::new(0xfffffea7),
                PlainDword::new(0xfffffebb),
                PlainDword::new(0xfffffece),
                PlainDword::new(0xfffffee0),
                PlainDword::new(0xfffffef4),
                PlainDword::new(0xffffff06),
                PlainDword::new(0xffffff18),
                PlainDword::new(0xffffff2a),
                PlainDword::new(0xffffff3b),
                PlainDword::new(0xffffff4c),
                PlainDword::new(0xffffff5c),
                PlainDword::new(0xffffff6c),
                PlainDword::new(0xffffff7c),
                PlainDword::new(0xffffff8c),
                PlainDword::new(0xffffff9b),
                PlainDword::new(0xffffffaa),
                PlainDword::new(0xffffffb9),
                PlainDword::new(0xffffffc8),
                PlainDword::new(0xffffffd6),
                PlainDword::new(0xffffffe4),
                PlainDword::new(0xfffffff2),
                PlainDword::new(0x0),
            ],
            voice_manager: PlainDword::new(0),
            a3d_splash_kept: PlainDword::new(0),
            a3d_ready: PlainDword::new(0),
            a3d_object: PlainDword::new(0),
            eax_properties: PlainDword::new(0),
            eax_room: PlainDword::new(0x6),
            eax_room_set: PlainDword::new(0x6),
            environment_kept: PlainDword::new(0x3f800000),
            a3d_clsid: PlainBytes::new([
                0xe0, 0xee, 0xf1, 0xd8, 0x34, 0xf6, 0xcf, 0x11, 0x87, 0, 0, 0xa0, 0x24, 0x5d, 0x91,
                0x8b,
            ]),
            a3d2_interface: PlainDword::new(0),
            use_a3d: PlainDword::new(0),
            a3d2_clsid: PlainBytes::new([
                0x24, 0x2c, 0xfa, 0x92, 0x3c, 0x25, 0xd2, 0x11, 0x90, 0xfb, 0, 0x60, 0x8, 0xa1,
                0xf4, 0x41,
            ]),
            a3d2_iid: PlainBytes::new([
                0x80, 0x2, 0xc4, 0xe4, 0xba, 0xcc, 0xd2, 0x11, 0x9d, 0xcf, 0, 0x50, 0x4, 0x11,
                0x58, 0x2f,
            ]),
        }
    }
}

impl std::ops::Deref for AudioOwn {
    type Target = AudioShared;

    fn deref(&self) -> &AudioShared {
        self.shared
    }
}

/// What the audio thread works with: its own variables (`state`, which derefs to the shared
/// ones) and the rest of the audio owner (the commands, the sample records and names, the
/// levels), which `AudioThread` derefs to. Made by the thread's loop (`audio::thread`).
pub struct AudioThread {
    pub state: AudioOwn,
    owner: &'static Audio,
}

impl AudioThread {
    pub fn new(owner: &'static Audio) -> Self {
        AudioThread {
            state: AudioOwn::new(&owner.state),
            owner,
        }
    }
}

impl std::ops::Deref for AudioThread {
    type Target = Audio;

    fn deref(&self) -> &Audio {
        self.owner
    }
}
