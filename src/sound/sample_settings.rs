//! `sound::sample_settings` (0x40BA71): a sound's volume and rate, the sample's own where
//! asked for and held to what the mixer takes.
//!
//! Returns: the volume, the rate
//! A sample without a buffer (+8 null): volume 0, rate 44100. Else a volume of -0x6F (the
//! sample's own) takes the record's +0x28, then is held to 0 .. 0x40 (signed); a rate of -0x6F takes the record's
//! +0x30, then (so a record's -0x70 chains) -0x70 the buffer's +0x24, then is held to
//! 100 .. 200000 (signed).
//! Five callers (the race sounds' turn). Refuses a sample past the 0x1CD records (the
//! original reads past them, into the next table; refused).

use crate::audio::owner::Audio;
use crate::audio::voices::{SampleRecord, voice_of};
use crate::errors::BadInput;

// ---- game data ----

pub const RECORDS: u32 = 0x1CD;
/// "The sample's own".
pub const OWN: u32 = 0xFFFF_FF91;
/// A copy of the record of sample `n`, or a refusal past the records.
pub fn record(audio: &Audio, n: u32) -> Result<SampleRecord, BadInput> {
    if n >= RECORDS {
        return Err(BadInput("a sample past the 0x1CD records"));
    }
    Ok(audio.sample_records.records[n as usize].get())
}

/// The rate as both leaves take it: -0x6F the record's +0x30, then -0x70 the buffer's
/// +0x24; held to 100 .. 200000.
/// The record's and its first voice's figures (`None`: no voice, not loaded).
pub fn loaded(audio: &Audio, record: &SampleRecord) -> Option<rc2k_core::audio::samples::Loaded> {
    if record.first_voice == 0 {
        return None;
    }
    let first = voice_of(&audio.state, record.first_voice);
    Some(rc2k_core::audio::samples::Loaded {
        voice_kind: record.voice_kind,
        volume: record.volume,
        frequency: record.frequency,
        own_frequency: first.own_frequency,
        format: first.format,
        frames: first.frames,
    })
}

/// (volume, rate).
pub fn sample_settings(
    audio: &Audio,
    sample: u32,
    volume: u32,
    rate: u32,
) -> Result<(u32, u32), BadInput> {
    // Converted: `rc2k_core::audio::samples::settings`.
    let record = record(audio, sample)?;
    Ok(rc2k_core::audio::samples::settings(
        volume,
        rate,
        loaded(audio, &record).as_ref(),
    ))
}
