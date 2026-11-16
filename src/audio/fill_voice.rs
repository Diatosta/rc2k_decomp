//! Fills a new voice for its sample record. A voice that shares data with another takes its data,
//! length, format, frames and rate from it and joins the end of the record's chain; otherwise they
//! come from the sample file. The rest comes from the record: flags, sound, kind, scale,
//! priority, volume, pan, frequency, place, minimum distance and volume factor. Refuses a shared
//! voice for a record without voices, and a chain without its end.

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
