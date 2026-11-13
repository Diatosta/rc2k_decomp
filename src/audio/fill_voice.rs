//! `audio::fill_voice` (0x40B06E): a new voice filled for its sample record.
//!
//! Takes: the voice it shares data with (0: none), the record, the voice
//!
//! Without a shared voice: data, length, format, frames and rate from `g_sampleFile`, no
//! source, no next copy, the record's `data_key`. With one: the same five from it, it as
//! the source, the voice added at the end of the record's chain (unless it is on it
//! already), no next copy and no `data_key`. Then from the record: flags 1 with
//! DirectSound's buffers (`g_soundMode` bit 0x10000) else 2, sound, kind, scale,
//! priority, volume, the pan's low byte (signed), pan sign 0, the frequency (or the
//! voice's own for `OWN_FREQUENCY`), the place, the minimum distance from its own, the volume
//! factor (0x40, or what the record's points at); state 0, dirty bits 0xF more, no
//! channel, no list link. One caller, command 3 (`audio::load_sounds`).
//! Refuses a shared voice for a record without voices, a chain without its end.

use crate::audio::own::AudioThread;
use crate::audio::samples::fill_voice;
use crate::errors::Refused;

pub fn fill_voice_in_game(
    audio: &AudioThread,
    at: u32,
    record: crate::audio::voices::SampleRecord,
    shared: u32,
) -> Result<(), Refused> {
    let state = &audio.state;
    let file = state.sample_file.get();
    fill_voice(audio, at, record, shared, &file, state.sound_mode.get())
}
