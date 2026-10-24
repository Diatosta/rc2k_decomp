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

use crate::framework::rva;
use crate::session::MACHINE;
use crate::sound::store::EffectsSlot;
use crate::sound::store::StageSlot;
use crate::store::{game_addr, game_read, game_write};

/// unk_5168C0.
pub const VOICES: usize = 0x5168C0;
pub const VOICE_LEN: u32 = 0x50;
pub const VOICE_COUNT: u32 = 0x48;
/// g_sharedSoundSpots, g_sharedSoundSpotCount.
pub const SPOTS: StageSlot = StageSlot::at(0x5165F0);
pub const SPOT_COUNT: StageSlot = StageSlot::at(0x5165EC);
pub const SPOT_LEN: u32 = 0x14;
pub const MOST_SPOTS: u32 = 36;
/// dword_517F54: a kind's sound, 22 of them.
pub const SOUND_OF_KIND: EffectsSlot = EffectsSlot::at(0x517F54);
/// unk_517FAC: a kind's repeats (8 bytes, kind 1 first: +2 the most, +4 and +6 the
/// least and most delay, words).
pub const REPEATS: EffectsSlot = EffectsSlot::at(0x517FAC);
pub const MOST_KIND: u32 = 0x15;
/// dword_517F40 .. dword_517F50: the work of the two pickers (the playing one, the
/// nearest's distance and voice, the next's distance and voice).
pub const WORK: EffectsSlot = EffectsSlot::at(0x517F40);

/// The address of voice `n`.
pub fn voice(n: u32) -> u32 {
    game_addr(rva(VOICES)) as u32 + n * VOICE_LEN
}

/// The dword at `at` of the voice at `v`.
pub fn vw(v: u32, at: u32) -> u32 {
    unsafe { ((v + at) as *const u32).read_unaligned() }
}

pub fn set(v: u32, at: u32, value: u32) {
    unsafe { ((v + at) as *mut u32).write_unaligned(value) }
}

pub fn g(x: impl crate::store::Ida) -> u32 {
    unsafe { crate::store::ida_read(x) }
}

pub fn p(x: impl crate::store::Ida, v: u32) {
    unsafe { crate::store::ida_write(x, v) }
}

/// The voices as the core's (`rc2k_core::sound::spot_voices`).
pub fn load_voices() -> [rc2k_core::sound::spot_voices::Voice; VOICE_COUNT as usize] {
    let words: [[u32; 20]; VOICE_COUNT as usize] = unsafe { game_read(rva(VOICES)) };
    words.map(rc2k_core::sound::spot_voices::Voice::from_words)
}

/// The voices stored back.
pub fn store_voices(voices: &[rc2k_core::sound::spot_voices::Voice; VOICE_COUNT as usize]) {
    let words = voices.map(|v| v.to_words());
    unsafe { game_write(rva(VOICES), words) };
}

/// A voice's address, or 0 for none (the pickers' work holds addresses).
pub fn voice_or_0(n: Option<usize>) -> u32 {
    n.map_or(0, |n| voice(n as u32))
}

/// The pickers' work stored as the original leaves it, and the scratch dword (1000.0,
/// then the last distance x 1000) when a distance was worked out.
pub fn store_picks(
    picks: &rc2k_core::sound::spot_voices::Picks,
    trail: &rc2k_core::sound::spot_voices::Trail,
) {
    p(WORK, voice_or_0(picks.playing));
    p(WORK + 4, picks.nearest_distance);
    p(WORK + 8, voice_or_0(picks.nearest));
    p(WORK + 12, picks.next_distance);
    p(WORK + 16, voice_or_0(picks.next));
    if let Some(d) = trail.last_distance {
        MACHINE.scratch.set(d);
    }
}
