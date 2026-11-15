//! The audio thread's own variables: what only the audio thread touches (its mixer, voice lists,
//! device objects, sample work buffers and set-up state), plain cells in a record the thread's
//! loop owns (`audio::thread::audio_thread` makes it, a test makes its own), so that the game's
//! thread cannot reach it. What both threads touch is `store::AudioShared`: settings and menus
//! written by the game thread and read by the audio thread, results written by the audio thread
//! and read by the game thread, and the set-up values read from the settings file before the audio
//! thread starts (that order is not enforced). `AudioOwn` reaches the shared part as a reference
//! and derefs to it.

use crate::audio::describe_wave_format::WaveFormat;
use crate::audio::make_buffer::BufferDescription;
use crate::audio::mixer::MIX_BUFFER_LEN;
use crate::audio::owner::Audio;
use crate::audio::sample_data::SampleData;
use crate::audio::samples::SampleFile;
use crate::audio::store::{AudioShared, CHANNEL_IDLE};
use crate::memory::{ListCells, MemoryList};
use crate::session::{Plain, PlainBytes, PlainDword};
use rc2k_core::audio::commands::SoundList;
use rc2k_core::audio::voices::MixerChannel;

/// The variables only the audio thread touches (`AudioThread`'s, made by the thread's own loop).
pub struct AudioOwn {
    /// The shared part, which `AudioOwn` reaches as a reference (and derefs to).
    shared: &'static AudioShared,
    /// The clock reading of the audio thread's last turn.
    pub audio_turn_tick: PlainDword,
    /// The mode DirectSound was opened with.
    pub opened_sound_mode: PlainDword,
    /// 1 when a DirectSound buffer was lost (the thread restores them).
    pub sound_buffer_lost: PlainDword,
    /// 1 when 3D settings are waiting for the listener's commit.
    pub commit_3d: PlainDword,
    /// The primary buffer's volume kept at start (1: none).
    pub primary_volume_kept: PlainDword,
    /// {format flag, bits} twice: 0x10 eight bits, 0x20 sixteen.
    pub sample_bits_flags: [PlainDword; 4],
    /// {format flag, channels} twice: 0x100 mono, 0x200 stereo.
    pub sample_channel_flags: [PlainDword; 4],
    /// The samples' memory (a `MemoryList`).
    pub sample_memory: ListCells,
    /// The first voice on the playing list (an address; 0: none).
    pub playing_voices: PlainDword,
    /// The first voice on the stopping list.
    pub stopping_voices: PlainDword,
    /// The mixer's channels (8 at start).
    pub channel_count: PlainDword,
    /// The mixer's channels.
    pub channels: [Plain<MixerChannel>; 16],
    /// The mixer's rate (the primary buffer's).
    pub mix_rate: PlainDword,
    /// The primary buffer's bytes.
    pub primary_size: PlainDword,
    /// The most a turn mixes, 40 ms of bytes.
    pub mix_most: PlainDword,
    /// Bytes to frames (a shift).
    pub mix_shift: PlainDword,
    /// The primary buffer's play cursor.
    pub play_cursor: PlainDword,
    /// The mixer's write place.
    pub write_cursor: PlainDword,
    /// The work buffer's memory (a `MemoryList`).
    pub sample_buffer_memory: ListCells,
    /// The work buffer for reading samples (an address).
    pub sample_buffer: PlainDword,
    /// Its bytes.
    pub sample_buffer_size: PlainDword,
    /// What the last sample file read held.
    pub sample_file: Plain<SampleFile>,
    /// 0x40, the volume factor a sample record's `volume_factor` points at.
    pub unit_volume_factor: PlainDword,
    /// The sound list loaded (command 3), none after it is freed.
    pub sound_list_loaded: Plain<Option<SoundList>>,
    /// The tune played when the music stops.
    pub next_tune: PlainDword,
    /// `;3files\music\00.wav`, the tune's two digits are put at +0xE.
    pub music_path: PlainBytes<24>,
    /// The 0xC00 bytes a stream's file head is read into (an address; 0: not taken yet).
    pub stream_header: PlainDword,
    /// The header's memory (a `MemoryList`).
    pub stream_header_memory: ListCells,
    /// A3D's interface id.
    pub a3d_iid: PlainBytes<16>,
    /// The 3D listener's interface id.
    pub iid_listener: PlainBytes<16>,
    /// The 3D buffer's interface id.
    pub iid_sound_3d_buffer: PlainBytes<16>,
    /// The property set's interface id.
    pub iid_property_set: PlainBytes<16>,
    /// The IDirectSound object (an address; 0: none).
    pub direct_sound: PlainDword,
    /// The primary buffer.
    pub primary_buffer: PlainDword,
    /// The 3D listener.
    pub sound_listener: PlainDword,
    /// What `Lock` gives: two pointers and their sizes.
    pub locked_parts: [PlainDword; 4],
    /// The primary buffer's `DSBCAPS` (its bytes at +8).
    pub primary_caps: PlainBytes<20>,
    /// The `DSBUFFERDESC` a buffer is made with.
    pub buffer_description: Plain<BufferDescription>,
    /// The `WAVEFORMATEX` a buffer is made with.
    pub wave_format: Plain<WaveFormat>,
    /// 65 volumes, -10000 .. 0 (signed).
    pub volume_table: [PlainDword; 65],
    /// The voice manager's IKsPropertySet.
    pub voice_manager: PlainDword,
    /// 1 = the splash pair acts.
    pub a3d_splash_kept: PlainDword,
    pub a3d_ready: PlainDword,
    pub a3d_object: PlainDword,
    /// EAX's IKsPropertySet.
    pub eax_properties: PlainDword,
    /// The room asked for (6 at start).
    pub eax_room: PlainDword,
    /// The room set (6 at start).
    pub eax_room_set: PlainDword,
    /// The race's environment (`ENVIRONMENT`) last set (1.0 at start).
    pub environment_kept: PlainDword,
    /// A3D's class id.
    pub a3d_clsid: PlainBytes<16>,
    pub a3d2_interface: PlainDword,
    /// 1 = A3D is the sound.
    pub use_a3d: PlainDword,
    /// A3D2's class id.
    pub a3d2_clsid: PlainBytes<16>,
    /// A3D2's interface id.
    pub a3d2_iid: PlainBytes<16>,
    /// The sounds' sample bytes: the work buffer, the stream's header and the kept samples.
    pub sample_data: SampleData,
    /// The mixing buffer: a turn's left and right dwords, from `mix_channels` to `write_mix`.
    pub mix: std::sync::Mutex<[i32; MIX_BUFFER_LEN as usize / 4]>,
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
            sound_list_loaded: Plain::new(None),
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
            sample_data: SampleData::new(),
            mix: std::sync::Mutex::new([0; MIX_BUFFER_LEN as usize / 4]),
        }
    }
}

impl std::ops::Deref for AudioOwn {
    type Target = AudioShared;

    fn deref(&self) -> &AudioShared {
        self.shared
    }
}

/// What the audio thread works with: its own variables (`state`, which derefs to the shared ones)
/// and the rest of the audio owner (the commands, the sample records and names, the levels), which
/// `AudioThread` derefs to. Made by the thread's loop (`audio::thread`).
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
