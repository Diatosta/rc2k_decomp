//! The audio variables both the game's thread and the audio thread touch (`AudioShared`): the
//! sound mode and volumes, the voices, the two stream players, the quality modes and the audio
//! options the advanced page edits, the facts of the sound device the menus show, the settings
//! read from the file, `tune_turn`. The rest (the mixer's channels and cursors, sample file and
//! buffers, the device objects and ids, the voice lists) is the audio thread's own,
//! `audio::own::AudioOwn`. The sample records are in `audio::sample_records`.
//!
//! The records the core works on (`Voice`, `MixerChannel`, `AudioStream`, `MemoryList`,
//! `AudioOptions`, `SampleFile`) are plain structs held in `Shared`, copied out and in or borrowed
//! whole; the scalars of `AudioShared` are atomic cells with relaxed loads and stores. Gaps (bytes
//! the code never names) are kept as `_gap_XXXX` buffers. The commands the game sends the audio
//! thread wait in a locked queue (`sound::queue`). The race's EAX environment
//! (`Audio::environment`) is written by the game's thread and read by the audio thread.

use crate::memory::session::{Bytes, Dword, Shared};
use rc2k_core::audio::modes::AudioOptions;
use rc2k_core::audio::streams::AudioStream;
use rc2k_core::audio::voices::{MixerChannel, VOICES, Voice};

/// The audio variables both the game's thread and the audio thread touch.
#[derive(rc2k_derive::Cells)]
pub struct AudioShared {
    /// The sound mode of each quality (0x212, 0x222, 0x221, 0x220), the advanced page's first
    /// choice picks one.
    pub quality_modes: [Dword; 4],
    /// 1 = no sound (the `noaudio` / `nosound` settings; the audio thread sets it when it ends).
    pub no_audio: Dword,
    /// The mode's bits (0x10000 own buffers, 0x40000 the game's mixer, 0x100000 3D ..) the thread
    /// works in.
    pub sound_mode: Dword,
    /// The mode for the menus (saved in `var\ral.dat`).
    pub menu_sound_mode: Dword,
    /// The mode for the races (saved in `var\ral.dat`).
    pub race_sound_mode: Dword,
    /// The mixing rate (`audiobasefreq`, 0xAC44 = 44100).
    pub mix_rate_setting: Dword,
    /// The master volume, 0 ..= 0x40 (saved in `var\ral.dat`).
    pub master_volume: Dword,
    /// The voices made so far.
    pub voice_count: Dword,
    /// The pool of voices (`audio::voices::Voice`).
    pub voices: [Shared<Voice>; VOICES as usize],
    /// The channels held by voices.
    pub channels_used: Dword,
    /// The work buffer's KB (`wavheapsize`).
    pub sample_buffer_kb: Dword,
    /// The sound stream's player (command 9).
    pub sound_stream: Shared<AudioStream>,
    /// The music's player (command 0xB).
    pub music_stream: Shared<AudioStream>,
    /// 1 = no music (`nomusic`).
    pub no_music: Dword,
    /// DirectSound's `DSCAPS` (flags at +4, free hardware buffers at +0x14, free 3D ones at +0x3C).
    pub sound_caps: Bytes<96>,
    /// The `ignoredsv` setting.
    pub ignore_dsv_setting: Dword,
    /// The voice manager is open.
    pub voice_manager_on: Dword,
    /// The settings file's `ignorea3d`: still read so that old files load; nothing uses it.
    pub ignore_a3d_setting: Dword,
    /// The `ignoreeax` setting.
    pub ignore_eax_setting: Dword,
    /// EAX is open.
    pub has_eax: Dword,
    /// Sounds held back.
    pub sounds_held_back: Dword,
    /// The advanced page's choices.
    pub audio_options: Shared<AudioOptions>,
    /// Which of tunes 10 .. 15 the next tune 0xA is (0 .. 5), kept in `var\ral.dat`; the audio
    /// thread's music command steps it, the game's thread loads and saves it.
    pub tune_turn: Dword,
    /// `ignoreds3d`: 1 = no 3D listener.
    pub no_listener: Dword,
    /// 3D is there.
    pub has_3d: Dword,
}

/// A voice as the image holds all of them: free (sound -2), 44100 Hz, scale 0x40.
const VOICE_FREE: Voice = Voice {
    flags: 0x0,
    sound: 0xfffffffe,
    buffer: 0x0,
    buffer_3d: 0x0,
    data_key: 0x0,
    source: 0x0,
    next_copy: 0x0,
    data: 0x0,
    length: 0x0,
    own_frequency: 0xac44,
    format: 0x0,
    frames: 0x0,
    scale: 0x40,
    priority: 0x0,
    kind: 0x0,
    state: 0x0,
    dirty: 0x0,
    channel: 0x0,
    next: 0x0,
    volume_factor: 0x40,
    min_distance: 0x3f800000,
    volume: 0x40,
    pan: 0x0,
    pan_sign: 0x0,
    frequency: 0xac44,
    place: [0x0, 0x0, 0x0],
};
/// A mixer channel as the image holds all of them (a 1.0 at +0x20).
pub(super) const CHANNEL_IDLE: MixerChannel = MixerChannel {
    used: 0x0,
    gap_4: [0x0, 0x0],
    voice: 0x0,
    format: 0x0,
    data: 0x0,
    length: 0x0,
    kind: 0x0,
    gap_20: [0x3f800000, 0x0, 0x0],
    position: 0x0,
    position_fraction: 0x0,
    step: 0x0,
    step_fraction: 0x0,
    left: 0x0,
    right: 0x0,
    unused_44: [0x0, 0x0, 0x0, 0x0, 0x0],
};

impl AudioShared {
    pub const fn new() -> Self {
        AudioShared {
            quality_modes: [
                Dword::new(0x212),
                Dword::new(0x222),
                Dword::new(0x221),
                Dword::new(0x220),
            ],
            no_audio: Dword::new(0),
            sound_mode: Dword::new(0),
            menu_sound_mode: Dword::new(0),
            race_sound_mode: Dword::new(0),
            mix_rate_setting: Dword::new(0xac44),
            master_volume: Dword::new(0x41),
            voice_count: Dword::new(0),
            voices: [const { Shared::new(VOICE_FREE) }; VOICES as usize],
            channels_used: Dword::new(0),
            sample_buffer_kb: Dword::new(0x400),
            sound_stream: Shared::new(AudioStream {
                playing: 0x0,
                volume: 0x40,
                buffer: 0x0,
                open: 0x0,
                file: 0x0,
                play_cursor: 0x0,
                write_at: 0x0,
                format: 0x0,
                bytes: 0x10000,
                step: 0x8000,
                tail: 0x8000,
            }),
            music_stream: Shared::new(AudioStream {
                playing: 0x0,
                volume: 0x40,
                buffer: 0x0,
                open: 0x0,
                file: 0x0,
                play_cursor: 0x0,
                write_at: 0x0,
                format: 0x0,
                bytes: 0x10000,
                step: 0x8000,
                tail: 0x8000,
            }),
            no_music: Dword::new(0),
            sound_caps: Bytes::new([0; 96]),
            ignore_dsv_setting: Dword::new(0),
            voice_manager_on: Dword::new(0),
            ignore_a3d_setting: Dword::new(0x1),
            ignore_eax_setting: Dword::new(0),
            has_eax: Dword::new(0),
            sounds_held_back: Dword::new(0),
            audio_options: Shared::new(AudioOptions {
                kept: 0x0,
                choice: [0x2, 0x0, 0x0, 0x0, 0x0],
                applied: [0x2, 0x0, 0x0, 0x0, 0x0],
                has_3d: 0x0,
                has_eax: 0x0,
            }),
            tune_turn: Dword::new(0),
            no_listener: Dword::new(0),
            has_3d: Dword::new(0),
        }
    }
}
