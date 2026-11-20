//! The shared spots' voices: `SpotVoices.voices[0][0]`, 0x48 voices of 0x50 bytes, which
//! `sound::spot_sounds` keeps in step with the spots the race frame shares each frame (0x14 bytes
//! each: an id, the place, the kind 1 .. 0x15; `race::share_sound_spots`).
//!
//! A voice's fields are named in `SpotVoice`.

use crate::memory::session::{Cell32, PlainDword, PlainFloat};

pub(super) const VOICE_COUNT: u32 = 0x48;
pub const MOST_SPOTS: u32 = 36;
pub const MOST_KIND: u32 = 0x15;

/// A voice: the fields in the game's order. The whole numbers (ids,
/// flags, counts, the clock's turns) are dwords; the place, the rate, the volume, the distance, its
/// change and the two factors are floats.
#[derive(rc2k_derive::Cells)]
pub struct SpotVoice {
    /// The spot's id (`FREE_ID`: free).
    pub id: PlainDword,
    /// 1 when heard this turn.
    pub heard: PlainDword,
    /// Where the spot is.
    pub place: [PlainFloat; 3],
    /// The kind's sound (-2: freed).
    pub sound: PlainDword,
    pub kind: PlainDword,
    /// 1 << kind.
    pub kind_bit: PlainDword,
    /// 1 until its first turn.
    pub new: PlainDword,
    /// 1 to (re)start it.
    pub restart: PlainDword,
    pub playing: PlainDword,
    pub repeats: PlainDword,
    /// The delay before the next start.
    pub delay: PlainDword,
    /// When it ends (the clock's turns).
    pub ends: PlainDword,
    pub rate: PlainFloat,
    pub volume: PlainFloat,
    pub distance: PlainFloat,
    /// The distance's change a second.
    pub distance_change: PlainFloat,
    /// The pitch factor.
    pub pitch_factor: PlainFloat,
    /// The volume factor.
    pub volume_factor: PlainFloat,
}

impl SpotVoice {
    /// The id of a free voice.
    pub const FREE_ID: u32 = 0x1888_FFFF;

    /// A free voice, as the game starts: the id free, the sound -2, the pitch factor 1.0.
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
pub(super) fn load_voices(
    sound: &crate::sound::owner::Sound,
) -> [rc2k_core::sound::spot_voices::Voice; VOICE_COUNT as usize] {
    let words: [[u32; 20]; VOICE_COUNT as usize] = std::array::from_fn(|n| {
        let cells = sound.race.spot_voices.get().voices[n].cells();
        std::array::from_fn(|k| cells[k].bits())
    });
    words.map(rc2k_core::sound::spot_voices::Voice::from_words)
}

/// The voices stored back.
pub(super) fn store_voices(
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

/// The scratch dword (1000.0, then the last distance x 1000) when a distance was worked out.
pub(super) fn store_scratch(
    trail: &rc2k_core::sound::spot_voices::Trail,
    system: &crate::system::owner::System,
) {
    if let Some(d) = trail.last_distance {
        system.machine().scratch.set(d);
    }
}
