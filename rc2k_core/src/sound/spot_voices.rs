//! The voices of the race's shared sound spots (from
//! `sound::match_spot_voices` 0x40E642, `sound::exclusive_spot_voices` 0x40EAF3 and
//! `sound::spot_voice_distances` 0x40E733; also the pickers `sound::merge_spot_voices`
//! 0x40E809 and `thin_spot_voices` 0x40E9F8, `repeat_spot_voices` 0x40EB44 and
//! `free_spot_voices` 0x40E6F4, and `play_spot_voices` 0x40EB9E's choices): 0x48 voices
//! the race sound keeps in step with the spots the frame shares (a waterfall, the crowd:
//! kinds 1 ..= 0x15).

use crate::Invalid;

/// How many voices there are.
pub const VOICES: usize = 0x48;
/// The id of a free voice.
pub const FREE: u32 = 0x1888_FFFF;
/// The most spots a frame shares.
pub const MOST_SPOTS: usize = 36;
/// The last kind.
pub const MOST_KIND: u32 = 0x15;

/// The sounds whose voices are merged each turn, in order (from
/// `sound::merge_spot_voices_all` 0x40E7CA).
pub const MERGED_SOUNDS: [u32; 6] = [0x1B7, 0x1B8, 0x1BB, 0x1BC, 0x1C1, 0x1BF];
/// The sounds whose voices are thinned each turn, in order (from
/// `sound::thin_spot_voices_all` 0x40E97D).
pub const THINNED_SOUNDS: [u32; 12] = [
    0x1B1, 0x1B2, 0x1B3, 0x1B4, 0x1B5, 0x1B6, 0x1B9, 0x1BA, 0x1B0, 0x1BE, 0x1C0, 0x1C3,
];

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

/// A voice's distance x 1000 as a whole number, as `fistp` makes it: the nearest, halves to
/// even (the product is exact in the x87's precision); out of range or not a number the
/// x87's 0x80000000.
pub fn distance_1000(distance: f32) -> u32 {
    let product = (distance as f64 * 1000.0).round_ties_even();
    if product.is_nan() || product < i32::MIN as f64 || product > i32::MAX as f64 {
        0x8000_0000
    } else {
        product as i32 as u32
    }
}

/// The pickers' work (`dword_517F40` ..): voices by index, distances x 1000 (unsigned).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Picks {
    /// The last heard voice found playing (merge only).
    pub playing: Option<usize>,
    pub nearest_distance: u32,
    pub nearest: Option<usize>,
    pub next_distance: u32,
    pub next: Option<usize>,
}

impl Default for Picks {
    fn default() -> Self {
        Picks {
            playing: None,
            nearest_distance: u32::MAX,
            nearest: None,
            next_distance: u32::MAX,
            next: None,
        }
    }
}

/// What a pick leaves behind besides its work, for the registers the original leaves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Trail {
    /// The last distance worked out (also the game's scratch dword).
    pub last_distance: Option<u32>,
    /// When a new nearest came, the next it pushed out: the last time.
    pub pushed_out: Option<Option<usize>>,
    /// Thin: the playing voice the search ended at.
    pub ended_at: Option<usize>,
}

/// One voice weighed: the nearest and the next move down; true when it is the new nearest.
fn weigh(picks: &mut Picks, trail: &mut Trail, n: usize, distance: f32) -> bool {
    let d = distance_1000(distance);
    trail.last_distance = Some(d);
    if d < picks.nearest_distance {
        picks.next_distance = picks.nearest_distance;
        trail.pushed_out = Some(picks.next);
        picks.next = picks.nearest;
        picks.nearest_distance = d;
        picks.nearest = Some(n);
        true
    } else {
        if d < picks.next_distance {
            picks.next_distance = d;
            picks.next = Some(n);
        }
        false
    }
}

/// Of the voices heard with `sound`, the nearest and the next found and every one
/// silenced (unheard, not playing); the nearest then heard again and restarted. The
/// caller moves it between the two with `merge_between` when there is a next.
pub fn merge(voices: &mut [Voice; VOICES], sound: u32) -> (Picks, Trail) {
    let (mut picks, mut trail) = (Picks::default(), Trail::default());
    for (n, v) in voices.iter_mut().enumerate() {
        if v.heard != 1 || v.sound != sound {
            continue;
        }
        if v.playing == 1 {
            picks.playing = Some(n);
        }
        weigh(&mut picks, &mut trail, n, v.distance);
        v.heard = 0;
        v.playing = 0;
    }
    if let Some(n) = picks.nearest {
        voices[n].restart = 1;
        voices[n].heard = 1;
    }
    (picks, trail)
}

/// The nearest's new place: the point between it and the next nearest the listener.
pub fn merge_between(voices: &mut [Voice; VOICES], picks: &Picks, point: [u32; 3]) {
    if let Some(n) = picks.nearest {
        voices[n].place = point;
    }
}

/// Of the voices heard with `sound`, the nearest two found (a playing one ends the search
/// at once, nothing more changed); the others silenced (a nearest pushed out stays heard:
/// the original's silencing of it is never reached). The caller chooses one of the two by
/// a random number with `thin_choose` when there is a next.
pub fn thin(voices: &mut [Voice; VOICES], sound: u32) -> (Picks, Trail) {
    let (mut picks, mut trail) = (Picks::default(), Trail::default());
    for (n, v) in voices.iter_mut().enumerate() {
        if v.heard != 1 || v.sound != sound {
            continue;
        }
        if v.playing == 1 {
            trail.ended_at = Some(n);
            return (picks, trail);
        }
        if !weigh(&mut picks, &mut trail, n, v.distance) {
            v.heard = 0;
        }
    }
    (picks, trail)
}

/// The nearest heard by the random number's low bit, the next by the other.
pub fn thin_choose(voices: &mut [Voice; VOICES], picks: &Picks, random: u32) {
    if let (Some(a), Some(b)) = (picks.nearest, picks.next) {
        voices[a].heard = random & 1;
        voices[b].heard = (random & 1) ^ 1;
    }
}

/// A kind's repeats (`unk_517FAC`, 8 bytes from kind 1; words).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Repeats {
    /// The most starts (0: no limit).
    pub most: u16,
    pub least_delay: u16,
    pub most_delay: u16,
}

impl Repeats {
    /// From the record's eight bytes.
    pub fn from_bytes(b: [u8; 8]) -> Self {
        let w = |n: usize| u16::from_le_bytes([b[n], b[n + 1]]);
        Repeats {
            most: w(2),
            least_delay: w(4),
            most_delay: w(6),
        }
    }
}

/// Each voice heard and not playing: a delay of its kind (the least plus `below(n, range)` when the
/// range is not 0, the random number's upper half kept in the low 16 bits), and the restart flag
/// while its repeats last (counted). Refused (before anything): such a voice of kind 0 or past
/// 0x15.
pub fn repeat<E: From<Invalid>>(
    voices: &mut [Voice; VOICES],
    table: &[Repeats; MOST_KIND as usize],
    mut below: impl FnMut(usize, u32) -> Result<u32, E>,
) -> Result<(), E> {
    let waiting = |v: &Voice| v.heard == 1 && v.playing != 1;
    if voices
        .iter()
        .any(|v| waiting(v) && !(1..=MOST_KIND).contains(&v.kind))
    {
        return Err(Invalid("a waiting spot voice of a kind past the table").into());
    }
    for (n, v) in voices.iter_mut().enumerate() {
        if !waiting(v) {
            continue;
        }
        let k = table[v.kind as usize - 1];
        let range = k.most_delay.wrapping_sub(k.least_delay) as u32;
        let r = if range != 0 { below(n, range)? } else { range };
        v.delay = (r & 0xFFFF_0000) | (r as u16).wrapping_add(k.least_delay) as u32;
        if k.most != 0 {
            if k.most as u32 == v.repeats {
                continue;
            }
            v.repeats = v.repeats.wrapping_add(1);
        }
        v.restart = 1;
    }
    Ok(())
}

/// Each voice not heard this turn freed (id `FREE`, sound -2): a playing one first has
/// its sound stopped (`stop(n, sound)`), unless it is already free (then it is left).
pub fn free_unheard<E>(
    voices: &mut [Voice; VOICES],
    mut stop: impl FnMut(usize, u32) -> Result<(), E>,
) -> Result<(), E> {
    for (n, v) in voices.iter_mut().enumerate() {
        if v.heard == 1 {
            continue;
        }
        if v.playing == 1 {
            if v.id == FREE {
                continue;
            }
            stop(n, v.sound)?;
        }
        v.id = FREE;
        v.sound = 0xFFFF_FFFE;
    }
    Ok(())
}

/// What `sound::play_spot_voices` does with a voice heard this turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Played {
    /// Told to start (+0x24 1): its settings and end worked out, playing.
    Start,
    /// Playing (+0x28 1): it goes on.
    GoOn,
    /// Neither: nothing sent.
    Silent,
}

/// A heard voice's play this turn: `start` (+0x24) first, `playing` (+0x28) asked only
/// when it is not told to start.
pub fn played(start: u32, playing: impl FnOnce() -> u32) -> Played {
    if start == 1 {
        Played::Start
    } else if playing() == 1 {
        Played::GoOn
    } else {
        Played::Silent
    }
}

/// A voice going on: its sound with bit 31 set (sent again without restarting it)
/// unless its sample loops (the sample record's +0x14 is 1).
pub fn going_on(sound: u32, looping: u32) -> u32 {
    if looping != 1 {
        sound | 0x8000_0000
    } else {
        sound
    }
}

/// A started voice's end: its sample's duration from now, after its delay (wrapping).
pub fn end_time(duration: u32, clock: u32, delay: u32) -> u32 {
    duration.wrapping_add(clock).wrapping_add(delay)
}

/// Whether a voice's time is up: the clock at or past its end (`js`: a signed
/// difference).
pub fn time_up(clock: u32, end: u32) -> bool {
    clock.wrapping_sub(end) as i32 >= 0
}

/// The request's dwords a voice fills, with the voice's offsets they come from, in order:
/// the volume, the rate, the two factors, the place.
pub const REQUEST_FROM: [(usize, u32); 7] = [
    (2, 0x3C),
    (1, 0x38),
    (8, 0x48),
    (9, 0x4C),
    (3, 8),
    (4, 0xC),
    (5, 0x10),
];

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

    #[test]
    fn pickers() {
        let mut voices = [Voice::default(); VOICES];
        for (n, d) in [(2, 5.0), (5, 3.0), (9, 4.0), (11, 9.0)] {
            voices[n].heard = 1;
            voices[n].sound = 7;
            voices[n].distance = d;
        }
        voices[11].playing = 1;
        let mut merged = voices;
        let (picks, trail) = merge(&mut merged, 7);
        assert_eq!(
            (picks.nearest, picks.next, picks.playing),
            (Some(5), Some(9), Some(11))
        );
        assert_eq!(
            (trail.last_distance, trail.pushed_out),
            (Some(9000), Some(None))
        );
        assert_eq!(
            (merged[5].heard, merged[5].restart, merged[2].heard),
            (1, 1, 0)
        );
        merge_between(&mut merged, &picks, [1, 2, 3]);
        assert_eq!(merged[5].place, [1, 2, 3]);
        let mut thinned = voices;
        let (picks, trail) = thin(&mut thinned, 7);
        assert_eq!(
            (picks.nearest, picks.next, trail.ended_at),
            (Some(5), Some(9), Some(11))
        );
        // The playing one ended it: 2 pushed out stays heard, 9 silenced.
        assert_eq!((thinned[2].heard, thinned[9].heard), (1, 0));
        thin_choose(&mut thinned, &picks, 3);
        assert_eq!((thinned[5].heard, thinned[9].heard), (1, 0));
        assert_eq!(distance_1000(2.0005), 2000);
        assert_eq!(distance_1000(f32::INFINITY), 0x8000_0000);
        // Repeats: kind 1, 2 starts at most, delays 10 .. 14.
        let mut table = [Repeats::default(); 21];
        table[0] = Repeats {
            most: 2,
            least_delay: 10,
            most_delay: 14,
        };
        let mut r = [Voice::default(); VOICES];
        r[0] = Voice {
            heard: 1,
            kind: 1,
            repeats: 1,
            ..Voice::default()
        };
        repeat::<Invalid>(&mut r, &table, |_, range| Ok(range - 1)).unwrap();
        assert_eq!((r[0].delay, r[0].repeats, r[0].restart), (13, 2, 1));
        r[0].restart = 0;
        repeat::<Invalid>(&mut r, &table, |_, _| Ok(0)).unwrap();
        assert_eq!((r[0].delay, r[0].restart), (10, 0));
        r[1] = Voice {
            heard: 1,
            kind: 0,
            ..Voice::default()
        };
        assert!(repeat::<Invalid>(&mut r, &table, |_, _| Ok(0)).is_err());
        // Freeing: 3 playing and stopped, 4 playing and free left, 5 not playing freed.
        let mut f = [Voice {
            heard: 1,
            ..Voice::default()
        }; VOICES];
        f[3] = Voice {
            playing: 1,
            sound: 8,
            id: 1,
            ..Voice::default()
        };
        f[4] = Voice {
            playing: 1,
            id: FREE,
            sound: 9,
            ..Voice::default()
        };
        f[5] = Voice {
            id: 2,
            ..Voice::default()
        };
        let mut stopped = Vec::new();
        free_unheard::<()>(&mut f, |n, s| {
            stopped.push((n, s));
            Ok(())
        })
        .unwrap();
        assert_eq!(stopped, [(3, 8)]);
        assert_eq!(
            (f[3].id, f[3].sound, f[4].sound, f[5].id),
            (FREE, 0xFFFF_FFFE, 9, FREE)
        );
    }

    #[test]
    fn plays() {
        assert_eq!(played(1, || unreachable!()), Played::Start);
        assert_eq!(played(0, || 1), Played::GoOn);
        assert_eq!(played(2, || 0), Played::Silent);
        assert_eq!(going_on(5, 1), 5);
        assert_eq!(going_on(5, 0), 0x8000_0005);
        assert_eq!(end_time(100, 0xFFFF_FFF0, 0x20), 116);
        assert!(time_up(10, 10) && !time_up(9, 10) && time_up(1, 0xFFFF_FFFF));
    }
}
