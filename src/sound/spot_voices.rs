//! The shared spots' voices: `SpotVoices.voices[0][0]`, 0x48 voices of 0x50 bytes, which
//! `sound::spot_sounds` keeps in step with the spots the race frame shares each frame (0x14 bytes
//! each: an id, the place, the kind 1 .. 0x15; `race::share_sound_spots`).
//!
//! A voice's fields are named in `SpotVoice`.

pub(super) const VOICE_COUNT: u32 = 0x48;
pub const MOST_SPOTS: u32 = 36;
pub const MOST_KIND: u32 = 0x15;

/// A voice: the fields in the game's order. The whole numbers (ids,
/// flags, counts, the clock's turns) are dwords; the place, the rate, the volume, the distance, its
/// change and the two factors are floats.
#[derive(Clone, Copy)]
pub struct SpotVoice {
    /// The spot's id (`FREE_ID`: free).
    pub id: u32,
    /// 1 when heard this turn.
    pub heard: u32,
    /// Where the spot is.
    pub place: [f32; 3],
    /// The kind's sound (-2: freed).
    pub sound: u32,
    pub kind: u32,
    /// 1 << kind.
    pub kind_bit: u32,
    /// 1 until its first turn.
    pub new: u32,
    /// 1 to (re)start it.
    pub restart: u32,
    pub playing: u32,
    pub repeats: u32,
    /// The delay before the next start.
    pub delay: u32,
    /// When it ends (the clock's turns).
    pub ends: u32,
    pub rate: f32,
    pub volume: f32,
    pub distance: f32,
    /// The distance's change a second.
    pub distance_change: f32,
    /// The pitch factor.
    pub pitch_factor: f32,
    /// The volume factor.
    pub volume_factor: f32,
}

impl SpotVoice {
    /// The id of a free voice.
    pub const FREE_ID: u32 = 0x1888_FFFF;

    /// A free voice, as the game starts: the id free, the sound -2, the pitch factor 1.0.
    pub const fn free() -> SpotVoice {
        SpotVoice {
            id: Self::FREE_ID,
            heard: 0,
            place: [0.0; 3],
            sound: 0xFFFF_FFFE,
            kind: 0,
            kind_bit: 0,
            new: 0,
            restart: 0,
            playing: 0,
            repeats: 0,
            delay: 0,
            ends: 0,
            rate: 0.0,
            volume: 0.0,
            distance: 0.0,
            distance_change: 0.0,
            pitch_factor: 1.0,
            volume_factor: 0.0,
        }
    }

    /// The twenty dwords in the game's order, as the core's `Voice` takes them.
    pub(super) fn to_words(self) -> [u32; 20] {
        [
            self.id,
            self.heard,
            self.place[0].to_bits(),
            self.place[1].to_bits(),
            self.place[2].to_bits(),
            self.sound,
            self.kind,
            self.kind_bit,
            self.new,
            self.restart,
            self.playing,
            self.repeats,
            self.delay,
            self.ends,
            self.rate.to_bits(),
            self.volume.to_bits(),
            self.distance.to_bits(),
            self.distance_change.to_bits(),
            self.pitch_factor.to_bits(),
            self.volume_factor.to_bits(),
        ]
    }

    /// A voice from its twenty dwords.
    pub(super) fn from_words(w: [u32; 20]) -> SpotVoice {
        let f = f32::from_bits;
        SpotVoice {
            id: w[0],
            heard: w[1],
            place: [f(w[2]), f(w[3]), f(w[4])],
            sound: w[5],
            kind: w[6],
            kind_bit: w[7],
            new: w[8],
            restart: w[9],
            playing: w[10],
            repeats: w[11],
            delay: w[12],
            ends: w[13],
            rate: f(w[14]),
            volume: f(w[15]),
            distance: f(w[16]),
            distance_change: f(w[17]),
            pitch_factor: f(w[18]),
            volume_factor: f(w[19]),
        }
    }
}

/// The voices as the core's (`rc2k_core::sound::spot_voices`).
pub(super) fn load_voices(
    sound: &crate::sound::owner::Sound,
) -> [rc2k_core::sound::spot_voices::Voice; VOICE_COUNT as usize] {
    let voices = sound.race.spot_voices.get().voices.borrow();
    std::array::from_fn(|n| rc2k_core::sound::spot_voices::Voice::from_words(voices[n].to_words()))
}

/// The voices stored back.
pub(super) fn store_voices(
    sound: &crate::sound::owner::Sound,
    voices: &[rc2k_core::sound::spot_voices::Voice; VOICE_COUNT as usize],
) {
    let mut held = sound.race.spot_voices.get().voices.borrow_mut();
    for (voice, new) in held.iter_mut().zip(voices) {
        *voice = SpotVoice::from_words(new.to_words());
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
