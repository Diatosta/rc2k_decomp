//! The sample records: sound number -> its record, as the audio thread and the game's thread read
//! them. `SAMPLE_CONSTANTS` holds the twelve fields that never change and `SampleRecords` the two
//! that do, `data_key` and `first_voice`; the names and volume levels the records point at are
//! handles (a name's key into `names_store`, a `VolumeLevel`). The table the constants are built
//! from is `audio::sample_table`.
//!
//! `data_key` is written by the game's thread when a race is set up (`sound::load_race_sounds`),
//! before the Load command, and read by the audio thread when it loads (the command queue's lock
//! orders them); `first_voice` is written by the audio thread (`audio::load_sounds`,
//! `free_voices`) and read by both; the rest is read by both, written by none.

use crate::audio::sample_table::first_records;
use crate::memory::session::Dword;
use rc2k_core::audio::voices::{SAMPLES, SampleRecord};

const SAMPLE_COUNT: usize = SAMPLES as usize;

/// The sample records: sound number -> its record. Of a record's 14 fields only
/// two change after the start, and these are all that is shared here: `data_key` (the game's thread
/// sets the race voices' file names, then sends the load command, which the locked queue orders
/// before the audio thread's read) and `first_voice` (the audio thread's loads and `free_voices`
/// set it, `sample_settings` of the game's thread reads it). The other twelve are constants
/// (`SAMPLE_CONSTANTS`, read by both threads), so `record` gives a whole `SampleRecord` made of
/// both.
#[derive(rc2k_derive::Cells)]
pub struct SampleRecords {
    data_keys: [Dword; SAMPLE_COUNT],
    first_voices: [Dword; SAMPLE_COUNT],
}

/// The sample records' fields that never change (`data_key` and `first_voice` are 0 here: they are
/// `SampleRecords`'), by sound number.
pub static SAMPLE_CONSTANTS: [SampleRecord; SAMPLE_COUNT] = constants();

/// The records as the image holds them, without the two fields that change.
const fn constants() -> [SampleRecord; SAMPLE_COUNT] {
    let mut out = first_records();
    let mut n = 0;
    while n < SAMPLE_COUNT {
        out[n].data_key = 0;
        n += 1;
    }
    out
}

/// The name of `key` in `names`, read to its NUL inside the list that holds it: the last name of
/// every list ends in a NUL, so a name never runs on into the next list.
pub fn name_in(
    names: &crate::audio::names_store::SampleNames,
    key: u32,
) -> Result<Vec<u8>, crate::errors::BadInput> {
    use crate::audio::names_store::NameList;
    let key = key as usize;
    let Some((list, start)) = NameList::ALL.into_iter().find_map(|list| {
        let start = list.first_key() as usize;
        (start..start + list.len())
            .contains(&key)
            .then_some((list, start))
    }) else {
        return Err(crate::errors::BadInput(
            "a sample record's name key off the names",
        ));
    };
    let len = list.len();
    let text: Vec<u8> = (key - start..len)
        .map(|at| names.byte(list, at))
        .take_while(|&b| b != 0)
        .collect();
    if key - start + text.len() == len {
        return Err(crate::errors::BadInput(
            "a sample name with no end in its field",
        ));
    }
    Ok(text)
}

/// A record's `volume_factor`: which level its voices take (0: none, 0x40).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub(super) enum VolumeLevel {
    /// `AudioOwn::unit_volume_factor`.
    Unit = 1,
    /// `Levels::race_effects_volume`.
    RaceEffects = 2,
    /// `Levels::codriver_voice_volume`.
    CodriverVoice = 3,
    /// `Levels::engine_volume`.
    Engine = 4,
}

/// The level a record's `volume_factor` names, now (a `VolumeLevel`; 0 is no level).
pub(super) fn volume_level(
    audio: &crate::audio::own::AudioThread,
    code: u32,
) -> Result<u32, crate::errors::BadInput> {
    let state = &audio.state;
    let levels = &audio.levels;
    Ok(match code {
        1 => state.unit_volume_factor.get(),
        2 => levels.race_effects_volume.get(),
        3 => levels.codriver_voice_volume.get(),
        4 => levels.engine_volume.get(),
        _ => {
            return Err(crate::errors::BadInput(
                "a sample record's volume level unknown",
            ));
        }
    })
}

impl SampleRecords {
    pub const fn new() -> Self {
        let first = first_records();
        let mut data_keys = [const { Dword::new(0) }; SAMPLE_COUNT];
        let mut n = 0;
        while n < SAMPLE_COUNT {
            data_keys[n] = Dword::new(first[n].data_key);
            n += 1;
        }
        SampleRecords {
            data_keys,
            first_voices: [const { Dword::new(0) }; SAMPLE_COUNT],
        }
    }

    /// Sound `number`'s record as it is now, or `None` past the records.
    pub fn record(&self, number: u32) -> Option<SampleRecord> {
        let n = number as usize;
        let mut record = *SAMPLE_CONSTANTS.get(n)?;
        record.data_key = self.data_keys[n].get();
        record.first_voice = self.first_voices[n].get();
        Some(record)
    }

    /// Sound `number`'s file name key set (a race's voices, by the game's thread).
    pub fn set_data_key(&self, number: usize, key: u32) {
        self.data_keys[number].set(key);
    }

    /// Sound `number`'s first voice set (by the audio thread's loads and `free_voices`).
    pub fn set_first_voice(&self, number: usize, voice: u32) {
        self.first_voices[number].set(voice);
    }
}
