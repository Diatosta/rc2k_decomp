//! The voices of the race's shared sound spots (converted 2026-10-20 from
//! `sound::match_spot_voices` 0x40E642, `sound::exclusive_spot_voices` 0x40EAF3 and
//! `sound::spot_voice_distances` 0x40E733): 0x48 voices the race sound keeps in step with
//! the spots the frame shares (a waterfall, the crowd: kinds 1 ..= 0x15).

use crate::Invalid;

/// How many voices there are.
pub const VOICES: usize = 0x48;
/// The id of a free voice.
pub const FREE: u32 = 0x1888_FFFF;
/// The most spots a frame shares.
pub const MOST_SPOTS: usize = 36;
/// The last kind.
pub const MOST_KIND: u32 = 0x15;

/// A spot the frame shares.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Spot {
    pub id: u32,
    /// x, y, z (floats' bits, copied as they are).
    pub place: [u32; 3],
    pub kind: u32,
}

/// A voice, its twenty dwords in the game's order.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Voice {
    /// The spot's id (`FREE`: none).
    pub id: u32,
    /// 1 when heard this turn.
    pub heard: u32,
    pub place: [u32; 3],
    /// The kind's sound (-2 once freed).
    pub sound: u32,
    pub kind: u32,
    /// 1 << kind.
    pub bit: u32,
    /// 1 until its first turn.
    pub new: u32,
    /// 1 to (re)start it.
    pub restart: u32,
    pub playing: u32,
    pub repeats: u32,
    /// The delay before the next start.
    pub delay: u32,
    /// When it ends.
    pub ends: u32,
    pub rate: f32,
    pub volume: f32,
    pub distance: f32,
    /// The distance's change a second.
    pub change: f32,
    pub pitch: f32,
    pub loudness: f32,
}

impl Voice {
    /// From the game's twenty dwords.
    pub fn from_words(w: [u32; 20]) -> Self {
        let f = f32::from_bits;
        Voice {
            id: w[0],
            heard: w[1],
            place: [w[2], w[3], w[4]],
            sound: w[5],
            kind: w[6],
            bit: w[7],
            new: w[8],
            restart: w[9],
            playing: w[10],
            repeats: w[11],
            delay: w[12],
            ends: w[13],
            rate: f(w[14]),
            volume: f(w[15]),
            distance: f(w[16]),
            change: f(w[17]),
            pitch: f(w[18]),
            loudness: f(w[19]),
        }
    }

    /// As the game's twenty dwords.
    pub fn to_words(&self) -> [u32; 20] {
        [
            self.id,
            self.heard,
            self.place[0],
            self.place[1],
            self.place[2],
            self.sound,
            self.kind,
            self.bit,
            self.new,
            self.restart,
            self.playing,
            self.repeats,
            self.delay,
            self.ends,
            self.rate.to_bits(),
            self.volume.to_bits(),
            self.distance.to_bits(),
            self.change.to_bits(),
            self.pitch.to_bits(),
            self.loudness.to_bits(),
        ]
    }
}

/// Each spot of a kind 1 ..= 0x15 given its voice (the one with its id, else the first
/// free one, made its: new, the kind's sound, kind and bit, not repeating, playing or
/// restarting; none free: unheard), its place and heard. Refused (before anything): more
/// than 36 spots.
pub fn match_spots(
    spots: &[Spot],
    voices: &mut [Voice; VOICES],
    sound_of_kind: &[u32; MOST_KIND as usize + 1],
) -> Result<(), Invalid> {
    if spots.len() > MOST_SPOTS {
        return Err(Invalid("more than 36 shared spots"));
    }
    for spot in spots {
        if !(1..=MOST_KIND).contains(&spot.kind) {
            continue;
        }
        let at = match voices.iter().position(|v| v.id == spot.id) {
            Some(at) => at,
            None => match voices.iter().position(|v| v.id == FREE) {
                Some(at) => {
                    let v = &mut voices[at];
                    v.id = spot.id;
                    v.new = 1;
                    v.sound = sound_of_kind[spot.kind as usize];
                    v.bit = 1 << spot.kind;
                    v.kind = spot.kind;
                    v.repeats = 0;
                    v.playing = 0;
                    v.restart = 0;
                    at
                }
                None => continue,
            },
        };
        voices[at].place = spot.place;
        voices[at].heard = 1;
    }
    Ok(())
}

/// At most one voice of a kind starts: a kind playing, or started by an earlier voice in
/// the table, leaves the others unheard.
pub fn keep_exclusive(voices: &mut [Voice; VOICES]) {
    let mut bits = voices
        .iter()
        .filter(|v| v.heard == 1 && v.playing == 1)
        .fold(0, |bits, v| bits | v.bit);
    for v in voices.iter_mut().filter(|v| v.heard == 1 && v.playing != 1) {
        if v.bit & bits != 0 {
            v.heard = 0;
        }
        bits |= v.bit;
    }
}

/// The constants the distances use: the turns a second, and the pitch's offset (337.52),
/// factor (0.00296) and easing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hearing {
    pub turns_a_second: f32,
    pub offset: f32,
    pub factor: f32,
    pub ease: f32,
}

/// Each heard voice's distance (the root of its place's squares), the distance's change a
/// second, the pitch (1, or for a voice not new eased from 1 toward (change + offset) x
/// factor) and loudness (1, or 1 / the distance when that is over 1, compared as the
/// floats' bits), and new cleared. What the x87 keeps on its stack is an `f64`.
pub fn hear(voices: &mut [Voice; VOICES], k: &Hearing) {
    for v in voices.iter_mut().filter(|v| v.heard == 1) {
        let [x, y, z] = v.place.map(|w| f32::from_bits(w) as f64);
        let distance = (x * x + (y * y + z * z)).sqrt();
        let before = v.distance as f64;
        v.distance = distance as f32;
        v.change = ((before - distance) * k.turns_a_second as f64) as f32;
        v.loudness = 0.0;
        v.pitch = 1.0;
        if v.new != 1 {
            let pitch = v.pitch as f64;
            let toward = (v.change as f64 + k.offset as f64) * k.factor as f64;
            v.pitch = ((toward - pitch) * k.ease as f64 + pitch) as f32;
        }
        v.loudness = 1.0;
        if v.distance.to_bits() as i32 > 1.0f32.to_bits() as i32 {
            v.loudness = (1.0 / v.distance as f64) as f32;
        }
        v.new = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spots_and_voices() {
        let mut voices = [Voice {
            id: FREE,
            ..Voice::default()
        }; VOICES];
        voices[3].id = 77;
        let mut kinds = [0u32; 22];
        kinds[2] = 0x40;
        let spots = [
            Spot {
                id: 77,
                place: [1, 2, 3],
                kind: 2,
            },
            Spot {
                id: 5,
                place: [4, 5, 6],
                kind: 2,
            },
            Spot {
                id: 6,
                place: [0; 3],
                kind: 0,
            },
        ];
        match_spots(&spots, &mut voices, &kinds).unwrap();
        assert_eq!(
            (voices[3].place, voices[3].heard, voices[3].new),
            ([1, 2, 3], 1, 0)
        );
        assert_eq!(
            (voices[0].id, voices[0].sound, voices[0].bit, voices[0].new),
            (5, 0x40, 4, 1)
        );
        assert_eq!(voices[1].id, FREE);
        assert!(match_spots(&[Spot::default(); 37], &mut voices, &kinds).is_err());
        // Both kind 2: the second unheard.
        voices[0].playing = 0;
        voices[3].playing = 0;
        voices[3].bit = 4;
        keep_exclusive(&mut voices);
        assert_eq!((voices[0].heard, voices[3].heard), (1, 0));
        // Distances: 3, 4, 0 is 5 away; was 6: change 1 x 30.
        voices[0].place = [3.0f32.to_bits(), 4.0f32.to_bits(), 0];
        voices[0].distance = 6.0;
        voices[0].new = 0;
        let k = Hearing {
            turns_a_second: 30.0,
            offset: 337.52,
            factor: 0.00296,
            ease: 0.5,
        };
        hear(&mut voices, &k);
        assert_eq!(
            (voices[0].distance, voices[0].change, voices[0].loudness),
            (5.0, 30.0, 0.2)
        );
        assert!((voices[0].pitch - 1.0439).abs() < 1e-4);
    }
}
