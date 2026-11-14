//! `sound::load_race_sounds`: names the co-driver's speech files and tells the audio thread which
//! sets of sounds a race needs.
//!
//! The two cars' sound objects are `RaceSounds.table[0][RaceSounds.player_cars[0]]` and
//! `RaceSounds.table[0][RaceSounds.player_cars[1]]`. The first car's voice field says whose voice
//! it has: 0 the list of 95 male file names (`files\%speech\m\..`), else the female list
//! (`..\f\..`); the second car has the other list, or the same one if its voice field equals the
//! first's. The sample records 0xED .. 0x1AA of `SampleRecords.records[0]` get the names: the odd
//! ones the first car's, the even ones the second's. Then the sets, each a command 3 (load a list
//! of sounds) naming the set: in a replay (`RaceState.results.finished` not 0) only
//! `SoundSets.replay_set[0]`; else the first car's kind set (`SoundSets.kinds[0]`),
//! `first_set_a[0]`, `first_set_b[0]` and `first_set_c[0]`; unless `RaceSounds.one_player` is 1 the
//! second car's kind set (`SoundSets.kinds[1]`), `second_set_a[0]` and `second_set_b[0]`; and
//! `last_set[0]` unless `RaceSounds.one_player` is `RaceSounds.count`. The commands also carry the
//! loop's leftover values in their unused words; they are reproduced because the sound hash of a
//! held race records them. What the two counts and "a replay" are is a reading. One caller,
//! `sound::set_up_race_sounds`. Refuses a car number of 8 or more, a missing object, a kind of 0x22
//! or more and a list of names that does not hold 95 of at most 0x100 bytes.

use crate::audio::names_store::{NameList, SampleNames};
use crate::errors::BadInput;
use crate::race::owner::Race;
use crate::session::PlainDword;
use crate::sound::sets_store::NamedSet;
use rc2k_core::audio::commands::{Command, SoundList};

// ---- game data ----

const FIRST_SAMPLE: usize = 0xED;
const NAMES: usize = 95;
const OBJECTS: u32 = 8;
/// The kinds `SoundSets.kinds[0]` has sets for: 0x22, the cars' kinds (0x20 and 0x21, the
/// Clio's, share kind 0xD's sets).
const KINDS: u32 = 0x22;

/// The lengths of a list's 95 names, the list starting at the name key `first` (each name
/// ends in a NUL inside its field).
fn lengths(names: &SampleNames, first: u32) -> Result<Vec<u32>, BadInput> {
    let mut key = first;
    let mut lengths = Vec::with_capacity(NAMES);
    for _ in 0..NAMES {
        let length = crate::audio::store::name_in(names, key)?.len();
        if length >= 0x100 {
            return Err(BadInput("a speech file's name without its end"));
        }
        lengths.push(length as u32);
        key += length as u32 + 1;
    }
    Ok(lengths)
}

/// `send` gets each command 3's list (by handle: the audio thread resolves it). The variables
/// (`sound::owner::Sound::race`'s `sounds`): `RaceSounds.player_cars` the two cars' numbers,
/// `RaceSounds.one_player` and `RaceSounds.count` the counts, `RaceSounds.table[0]` the eight sound
/// objects; `RaceState.results.finished` the replay switch.
pub fn load_race_sounds(
    send: &mut dyn FnMut(SoundList),
    race: &Race,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), BadInput> {
    let r = sound.race.sounds.get();
    let object = |number: &PlainDword| -> Result<u32, BadInput> {
        let number = number.get();
        if number >= OBJECTS {
            return Err(BadInput("a car's number past the sound objects"));
        }
        Ok(number)
    };
    let (first, second) = (object(&r.player_cars[0])?, object(&r.player_cars[1])?);
    let kind = |object: u32| match crate::sound::store::SoundObject::of(sound, object)?
        .kind
        .get()
    {
        kind if kind < KINDS => Ok(kind as usize),
        _ => Err(BadInput("a car's kind past the table of sound sets")),
    };
    let replay = race.shared.state.results.finished.is_set();
    let (count, other_count) = (
        sound.race.shared.one_player.get(),
        sound.race.shared.count.get(),
    );
    let kinds = if replay {
        None
    } else {
        let second = if count != 1 {
            Some(kind(second)?)
        } else {
            None
        };
        Some((kind(first)?, second))
    };
    // The logic is `rc2k_core::sound::race::voices` and `sets`.
    use rc2k_core::sound::race::{Set, Voice, sets, voices};
    // The lists by name key (`audio::store::name_key`: their places in the names).
    let list = |v: Voice| {
        match v {
            Voice::Male => NameList::MaleSpeech,
            Voice::Female => NameList::FemaleSpeech,
        }
        .first_key()
    };
    let (a, b) = voices(
        crate::sound::store::SoundObject::of(sound, first)?
            .cue_queue
            .first
            .get(),
        crate::sound::store::SoundObject::of(sound, second)?
            .cue_queue
            .first
            .get(),
    );
    let (mut first_name, mut second_name) = (list(a), list(b));
    let (of_first, of_second) = (
        lengths(&audio.sample_names, first_name)?,
        lengths(&audio.sample_names, second_name)?,
    );
    // Nothing has been written or sent up to here.
    for n in 0..NAMES {
        let records = &audio.sample_records;
        records.set_data_key(FIRST_SAMPLE + n * 2, first_name);
        records.set_data_key(FIRST_SAMPLE + n * 2 + 1, second_name);
        first_name += (of_first[n] as usize + 1) as u32;
        second_name += (of_second[n] as usize + 1) as u32;
    }
    let (first_kind, second_kind) = kinds.unwrap_or((0, None));
    for set in sets(
        kinds.is_none(),
        first_kind as u32,
        second_kind.unwrap_or(0) as u32,
        count,
        other_count,
    ) {
        send(match set {
            Set::Replay => NamedSet::Replay.list(),
            Set::FirstKind(k) => sound.race.sets.kind_list(2 * k as usize)?,
            Set::First(n) => [NamedSet::FirstA, NamedSet::FirstB, NamedSet::FirstC][n].list(),
            Set::SecondKind(k) => sound.race.sets.kind_list(2 * k as usize + 1)?,
            Set::Second(n) => [NamedSet::SecondA, NamedSet::SecondB][n].list(),
            Set::Last => NamedSet::Last.list(),
        });
    }
    Ok(())
}

/// In the game: each list sent as command 3.
pub fn load_race_sounds_in_game(
    race: &Race,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), BadInput> {
    load_race_sounds(
        &mut |list| crate::sound::senders::send(audio, Command::Load(Some(list))),
        race,
        sound,
        audio,
    )
}
