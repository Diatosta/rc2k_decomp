//! `sound::load_race_sounds` (0x40EED8): names the co-driver's speech files and tells the
//! audio thread which sets of sounds a race needs.
//!
//! In:  ebp goes into the commands as it is
//! Out: every register preserved
//! The two cars' sound objects are `off_512A10[dword_514F70]` and
//! `off_512A10[dword_514F74]`. The first's dword +0x298 says whose voice it has: 0 the
//! list of 95 file names at 0x507C1A (`files\%speech\m\..`), else the list at 0x508914
//! (`..\f\..`); the second car has the other list, or the same one if its +0x298 is the
//! first's. The sample records 0xED .. 0x1AA of `unk_50A7A4` (0x40 bytes each) get the
//! names at their +0xC: the odd ones the first car's, the even ones the second's.
//! Then the sets, each a command 3 with the set's address in eax: in a replay
//! (`dword_75B108` not 0) only `unk_512340`; else the first car's kind
//! (`off_512360[its +4 * 8]`), `unk_511CA8`, `unk_512268`, `unk_511D04`; unless
//! `dword_514F78` is 1 the second car's kind (`off_512364[its +4 * 8]`), `unk_512014`,
//! `unk_512070`; and `unk_5121EC` unless `dword_514F78` is `dword_514F7C`.
//! The commands carry the loop's leftovers in the other registers, reproduced.
//! What the two counts and "a replay" are is a reading.
//! One caller, `sound::set_up_race_sounds`. Refuses a car number of 8 or more, a missing
//! object, a kind of 0x22 or more and a list of names that does not hold 95 of at most 0x100
//! bytes.

use crate::audio::names_store::SampleNames;
use crate::errors::BadInput;
use crate::pointers::GamePtr;
use crate::race::state_store::RACE_STATE;
use crate::session::Dword;
use crate::sound::queue::START_3;
use crate::sound::sets_store::RACE_SOUND_SETS;

// ---- game data ----

const FIRST_SAMPLE: usize = 0xED;
const NAMES: usize = 95;
const OBJECTS: u32 = 8;
/// The kinds `off_512360` has sets for: 0x22, the cars' kinds (0x20 and 0x21, the Clio's,
/// share kind 0xD's sets). The original reads it unchecked.
const KINDS: u32 = 0x22;

/// The lengths of a list's 95 names.
fn lengths(list: GamePtr) -> Result<Vec<u32>, BadInput> {
    use rc2k_core::strings::bytes::Bytes as _;
    let bytes = list.bytes("a speech file's names")?;
    let mut at = 0usize;
    let mut lengths = Vec::with_capacity(NAMES);
    for _ in 0..NAMES {
        let length = (0..0x100usize)
            .find(|&n| bytes.byte(at + n) == Some(0))
            .ok_or(BadInput("a speech file's name without its end"))?;
        lengths.push(length as u32);
        at += length + 1;
    }
    Ok(lengths)
}

// ---- Rust version ----

/// `send` gets each command 3's list (its eax, all the audio thread reads of it).
/// The variables (`crate::sound::store::RACE_SOUNDS`): dword_514F70 and dword_514F74 the
/// two cars' numbers, dword_514F78, dword_514F7C the counts, off_512A10 the eight sound
/// objects; dword_75B108 the replay switch.
pub fn load_race_sounds(send: &mut dyn FnMut(u32)) -> Result<(), BadInput> {
    use crate::sound::store::RACE_SOUNDS as R;
    let object = |number: &Dword| -> Result<usize, BadInput> {
        let number = number.get();
        if number >= OBJECTS {
            return Err(BadInput("a car's number past the sound objects"));
        }
        match R.table[number as usize].get() {
            0 => Err(BadInput("no sound object for a car")),
            object => Ok(object as usize),
        }
    };
    let (first, second) = (object(&R.player_cars[0])?, object(&R.player_cars[1])?);
    let kind = |object: usize| match crate::sound::store::SoundObject::at(object as u32)?
        .kind
        .get()
    {
        kind if kind < KINDS => Ok(kind as usize),
        _ => Err(BadInput("a car's kind past the table of sound sets")),
    };
    let replay = RACE_STATE.results.finished.get() != 0;
    let (count, other_count) = (R.one_player.get(), R.count.get());
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
    // Converted: `rc2k_core::sound::race::voices` and `sets`.
    use rc2k_core::sound::race::{Set, Voice, sets, voices};
    // The lists by name key (`audio::store::name_key`: their places in the names).
    let list = |v: Voice| {
        (match v {
            Voice::Male => core::mem::offset_of!(SampleNames, male_speech),
            Voice::Female => core::mem::offset_of!(SampleNames, female_speech),
        }) as u32
    };
    let (a, b) = voices(
        crate::sound::store::SoundObject::at(first as u32)?
            .cue_queue
            .first
            .get(),
        crate::sound::store::SoundObject::at(second as u32)?
            .cue_queue
            .first
            .get(),
    );
    let (mut ebx, mut edx) = (list(a), list(b));
    // Unbounded: the original reads the names on as far as they go.
    let name = |key| crate::audio::store::name_at(key).map(GamePtr::unbounded);
    let (of_first, of_second) = (lengths(name(ebx)?)?, lengths(name(edx)?)?);
    // Nothing has been written or sent up to here.
    for n in 0..NAMES {
        let records = &crate::audio::store::SAMPLE_RECORDS.records;
        records[FIRST_SAMPLE + n * 2].update(|r| r.data_key = ebx);
        records[FIRST_SAMPLE + n * 2 + 1].update(|r| r.data_key = edx);
        ebx += (of_first[n] as usize + 1) as u32;
        edx += (of_second[n] as usize + 1) as u32;
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
            Set::Replay => crate::session::address_of(&RACE_SOUND_SETS.replay_set),
            Set::FirstKind(k) => {
                crate::sound::sets_store::RaceSoundSets::kind_set_of(2 * k as usize)?
            }
            Set::First(n) => [
                crate::session::address_of(&RACE_SOUND_SETS.first_set_a),
                crate::session::address_of(&RACE_SOUND_SETS.first_set_b),
                crate::session::address_of(&RACE_SOUND_SETS.first_set_c),
            ][n],
            Set::SecondKind(k) => {
                crate::sound::sets_store::RaceSoundSets::kind_set_of(2 * k as usize + 1)?
            }
            Set::Second(n) => [
                crate::session::address_of(&RACE_SOUND_SETS.second_set_a),
                crate::session::address_of(&RACE_SOUND_SETS.second_set_b),
            ][n],
            Set::Last => crate::session::address_of(&RACE_SOUND_SETS.last_set),
        });
    }
    Ok(())
}

/// In the game: each list sent as command 3.
pub fn load_race_sounds_in_game() -> Result<(), BadInput> {
    load_race_sounds(&mut |list| crate::sound::senders::send(START_3, [list, 0, 0, 0]))
}
