//! The shared spots' voices: `SpotVoices.voices[0][0]`, 0x48 voices of 0x50
//! bytes, which `sound::spot_sounds` (0x40E611) keeps in step with the spots the race
//! frame shares each frame (`g_sharedSoundSpots`, 0x14 bytes: +0 an id, +4 .. +0xC the
//! place, +0x10 the kind 1 .. 0x15; `race::share_sound_spots`).
//!
//! A voice by offset (names by use): +0 the spot's id (0x1888FFFF: free), +4 heard this
//! turn, +8 .. +0x10 the place, +0x14 the sound (`SpotPicks.kind_sounds[0][kind]`; -2 freed), +0x18
//! the kind, +0x1C 1 << kind, +0x20 new, +0x24 (re)start it, +0x28 playing, +0x2C the
//! repeats, +0x30 the delay before the next, +0x34 when it ends, +0x38 the rate and
//! +0x3C the volume (floats), +0x40 the distance, +0x44 its change, +0x48 and +0x4C the
//! pitch and volume factors.

use crate::session::{Cell32, PlainDword, PlainFloat};

pub const VOICE_COUNT: u32 = 0x48;
pub const MOST_SPOTS: u32 = 36;
pub const MOST_KIND: u32 = 0x15;

/// A voice: 0x50 bytes, the fields in the game's order (the offsets are in the header). The
/// whole numbers (ids, flags, counts, the clock's turns) are dwords; the place, the rate, the
/// volume, the distance, its change and the two factors are floats.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SpotVoice {
    /// +0: the spot's id (`FREE_ID`: free).
    pub id: PlainDword,
    /// +4: 1 when heard this turn.
    pub heard: PlainDword,
    /// +8 .. +0x10: where the spot is.
    pub place: [PlainFloat; 3],
    /// +0x14: the kind's sound (-2: freed).
    pub sound: PlainDword,
    /// +0x18
    pub kind: PlainDword,
    /// +0x1C: 1 << kind.
    pub kind_bit: PlainDword,
    /// +0x20: 1 until its first turn.
    pub new: PlainDword,
    /// +0x24: 1 to (re)start it.
    pub restart: PlainDword,
    /// +0x28
    pub playing: PlainDword,
    /// +0x2C
    pub repeats: PlainDword,
    /// +0x30: the delay before the next start.
    pub delay: PlainDword,
    /// +0x34: when it ends (the clock's turns).
    pub ends: PlainDword,
    /// +0x38
    pub rate: PlainFloat,
    /// +0x3C
    pub volume: PlainFloat,
    /// +0x40
    pub distance: PlainFloat,
    /// +0x44: the distance's change a second.
    pub distance_change: PlainFloat,
    /// +0x48: the pitch factor.
    pub pitch_factor: PlainFloat,
    /// +0x4C: the volume factor.
    pub volume_factor: PlainFloat,
}

impl SpotVoice {
    /// The id of a free voice.
    pub const FREE_ID: u32 = 0x1888_FFFF;

    /// A free voice, as the image starts: the id free, the sound -2, the pitch factor 1.0.
    pub const fn free() -> SpotVoice {
        SpotVoice {
            id: PlainDword::new(Self::FREE_ID),
            heard: PlainDword::new(0),
            place: [const { PlainFloat::new(0.0) }; 3],
            sound: PlainDword::new(0xFFFF_FFFE),
            kind: PlainDword::new(0),
            kind_bit: PlainDword::new(0),
            new: PlainDword::new(0),
            restart: PlainDword::new(0),
            playing: PlainDword::new(0),
            repeats: PlainDword::new(0),
            delay: PlainDword::new(0),
            ends: PlainDword::new(0),
            rate: PlainFloat::new(0.0),
            volume: PlainFloat::new(0.0),
            distance: PlainFloat::new(0.0),
            distance_change: PlainFloat::new(0.0),
            pitch_factor: PlainFloat::new(1.0),
            volume_factor: PlainFloat::new(0.0),
        }
    }

    /// The twenty dwords in the game's order, as the core's `Voice` takes them (a whole-record
    /// transfer: each cell by its bits).
    fn cells(&self) -> [&dyn Cell32; 20] {
        [
            &self.id,
            &self.heard,
            &self.place[0],
            &self.place[1],
            &self.place[2],
            &self.sound,
            &self.kind,
            &self.kind_bit,
            &self.new,
            &self.restart,
            &self.playing,
            &self.repeats,
            &self.delay,
            &self.ends,
            &self.rate,
            &self.volume,
            &self.distance,
            &self.distance_change,
            &self.pitch_factor,
            &self.volume_factor,
        ]
    }
}

/// Voice `n` (under `VOICE_COUNT`).
pub fn voice(n: u32, sound: &crate::sound::owner::Sound) -> &SpotVoice {
    &sound.race.spot_voices.get().voices[n as usize]
}

/// The voices as the core's (`rc2k_core::sound::spot_voices`).
pub fn load_voices(
    sound: &crate::sound::owner::Sound,
) -> [rc2k_core::sound::spot_voices::Voice; VOICE_COUNT as usize] {
    let words: [[u32; 20]; VOICE_COUNT as usize] = std::array::from_fn(|n| {
        let cells = sound.race.spot_voices.get().voices[n].cells();
        std::array::from_fn(|k| cells[k].bits())
    });
    words.map(rc2k_core::sound::spot_voices::Voice::from_words)
}

/// The voices stored back.
pub fn store_voices(
    sound: &crate::sound::owner::Sound,
    voices: &[rc2k_core::sound::spot_voices::Voice; VOICE_COUNT as usize],
) {
    let words = voices.map(|v| v.to_words());
    for (voice, words) in sound.race.spot_voices.get().voices.iter().zip(words) {
        for (cell, word) in voice.cells().into_iter().zip(words) {
            cell.set_bits(word);
        }
    }
}

/// A voice as the pickers' work holds it: its number + 1, or 0 for none (the original held
/// the voice's address; nothing reads it back).
pub fn voice_or_0(n: Option<usize>) -> u32 {
    n.map_or(0, |n| n as u32 + 1)
}

/// The pickers' work stored as the original leaves it, and the scratch dword (1000.0,
/// then the last distance x 1000) when a distance was worked out.
pub fn store_picks(
    sound: &crate::sound::owner::Sound,
    picks: &rc2k_core::sound::spot_voices::Picks,
    trail: &rc2k_core::sound::spot_voices::Trail,
    system: &crate::system::owner::System,
) {
    let work = sound.race.spot_picks.get();
    work.pick_playing.set(voice_or_0(picks.playing));
    work.pick_nearest_distance.set(picks.nearest_distance);
    work.pick_nearest.set(voice_or_0(picks.nearest));
    work.pick_next_distance.set(picks.next_distance);
    work.pick_next.set(voice_or_0(picks.next));
    if let Some(d) = trail.last_distance {
        system.machine().scratch.set(d);
    }
}
