//! The shared spots' voices (stream B, 2026-10-18): `unk_5168C0`, 0x48 voices of 0x50
//! bytes, which `sound::spot_sounds` (0x40E611) keeps in step with the spots the race
//! frame shares each frame (`g_sharedSoundSpots`, 0x14 bytes: +0 an id, +4 .. +0xC the
//! place, +0x10 the kind 1 .. 0x15; `race::share_sound_spots`).
//!
//! A voice by offset (names by use): +0 the spot's id (0x1888FFFF: free), +4 heard this
//! turn, +8 .. +0x10 the place, +0x14 the sound (`dword_517F54[kind]`; -2 freed), +0x18
//! the kind, +0x1C 1 << kind, +0x20 new, +0x24 (re)start it, +0x28 playing, +0x2C the
//! repeats, +0x30 the delay before the next, +0x34 when it ends, +0x38 the rate and
//! +0x3C the volume (floats), +0x40 the distance, +0x44 its change, +0x48 and +0x4C the
//! pitch and volume factors.

use crate::errors::BadInput;
use crate::session::Dword;
use crate::system::owner::system;

pub const VOICE_LEN: u32 = 0x50;
pub const VOICE_COUNT: u32 = 0x48;
pub const MOST_SPOTS: u32 = 36;
pub const MOST_KIND: u32 = 0x15;

/// A voice's twenty dwords.
pub type VoiceWords = [Dword; 20];

/// Voice `n` (under `VOICE_COUNT`).
pub fn voice(n: u32, sound: &crate::sound::owner::Sound) -> &VoiceWords {
    &sound.race.spot_voices.voices[n as usize]
}

/// The address of voice `n` (the pickers' work holds addresses).
fn voice_address(n: u32, sound: &crate::sound::owner::Sound) -> u32 {
    crate::session::address_of(&sound.race.spot_voices.voices) + n * VOICE_LEN
}

/// The dword at byte `at` of voice `v`.
fn word(v: &VoiceWords, at: u32) -> Result<&Dword, BadInput> {
    if !at.is_multiple_of(4) {
        return Err(BadInput("a spot voice's dword out of line"));
    }
    v.get((at / 4) as usize)
        .ok_or(BadInput("a dword past a spot voice"))
}

/// The dword at byte `at` of voice `v`.
pub fn vw(v: &VoiceWords, at: u32) -> Result<u32, BadInput> {
    word(v, at).map(Dword::get)
}

/// The dword at byte `at` of voice `v` set to `value`.
pub fn set(v: &VoiceWords, at: u32, value: u32) -> Result<(), BadInput> {
    word(v, at).map(|w| w.set(value))
}

/// The voices as the core's (`rc2k_core::sound::spot_voices`).
pub fn load_voices(
    sound: &crate::sound::owner::Sound,
) -> [rc2k_core::sound::spot_voices::Voice; VOICE_COUNT as usize] {
    let words: [[u32; 20]; VOICE_COUNT as usize] =
        std::array::from_fn(|n| std::array::from_fn(|k| sound.race.spot_voices.voices[n][k].get()));
    words.map(rc2k_core::sound::spot_voices::Voice::from_words)
}

/// The voices stored back.
pub fn store_voices(
    sound: &crate::sound::owner::Sound,
    voices: &[rc2k_core::sound::spot_voices::Voice; VOICE_COUNT as usize],
) {
    let words = voices.map(|v| v.to_words());
    for (voice, words) in sound.race.spot_voices.voices.iter().zip(words) {
        for (cell, word) in voice.iter().zip(words) {
            cell.set(word);
        }
    }
}

/// A voice's address, or 0 for none (the pickers' work holds addresses).
pub fn voice_or_0(n: Option<usize>, sound: &crate::sound::owner::Sound) -> u32 {
    n.map_or(0, |n| voice_address(n as u32, sound))
}

/// The pickers' work stored as the original leaves it, and the scratch dword (1000.0,
/// then the last distance x 1000) when a distance was worked out.
pub fn store_picks(
    sound: &crate::sound::owner::Sound,
    picks: &rc2k_core::sound::spot_voices::Picks,
    trail: &rc2k_core::sound::spot_voices::Trail,
) {
    let work = &sound.race.spot_picks;
    work.pick_playing.set(voice_or_0(picks.playing, sound));
    work.pick_nearest_distance.set(picks.nearest_distance);
    work.pick_nearest.set(voice_or_0(picks.nearest, sound));
    work.pick_next_distance.set(picks.next_distance);
    work.pick_next.set(voice_or_0(picks.next, sound));
    if let Some(d) = trail.last_distance {
        system().session.machine.scratch.set(d);
    }
}
