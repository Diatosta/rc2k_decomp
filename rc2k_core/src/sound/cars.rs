//! The cars' race sounds that are whole-number logic (converted 2026-10-21): the spoken
//! cue queue (`sound::drop_cue` 0x40EE32, `sound::play_cue_queue` 0x40ED4A), the timed
//! slots (`sound::play_timed_slot` 0x40E59E), the one-off hit and cue sounds
//! (`sound::car_hit_sound` 0x40E182, `sound::car_cue_sound` 0x40E1E5) and the voices'
//! marks (`sound::mark_car_voices` 0x40D75A).

use crate::Invalid;

/// A rate or volume that means the sample's own (-0x6F).
pub const OWN: u32 = 0xFFFF_FF91;
/// No one-off sound waiting (-2).
pub const NONE: u32 = 0xFFFF_FFFE;
/// The most cues a queue holds.
pub const MOST_CUES: u32 = 16;
/// The most sounds a cue holds.
pub const MOST_SOUNDS: u32 = 6;

/// A spoken cue: 0x20 bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cue {
    pub first: u32,
    /// How many of `sounds` it has.
    pub count: u32,
    pub sounds: [u32; 6],
}

/// A car's queue of spoken cues (the sound object's +0x298, 0x210 bytes;
/// `physics::queue_sound_cue` fills it).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CueQueue {
    pub first: u32,
    /// The place in the first cue (-1: none begun).
    pub place: u32,
    /// When the next sound is due.
    pub due: u32,
    pub count: u32,
    pub cues: [Cue; 16],
}

impl CueQueue {
    /// From the game's 0x84 dwords.
    pub fn from_words(w: &[u32; 0x84]) -> Self {
        let mut q = CueQueue {
            first: w[0],
            place: w[1],
            due: w[2],
            count: w[3],
            ..CueQueue::default()
        };
        for (k, cue) in q.cues.iter_mut().enumerate() {
            let c = &w[4 + 8 * k..12 + 8 * k];
            *cue = Cue {
                first: c[0],
                count: c[1],
                sounds: c[2..8].try_into().unwrap(),
            };
        }
        q
    }

    /// As the game's 0x84 dwords.
    pub fn to_words(&self) -> [u32; 0x84] {
        let mut w = [0u32; 0x84];
        w[..4].copy_from_slice(&[self.first, self.place, self.due, self.count]);
        for (k, cue) in self.cues.iter().enumerate() {
            w[4 + 8 * k] = cue.first;
            w[5 + 8 * k] = cue.count;
            w[6 + 8 * k..12 + 8 * k].copy_from_slice(&cue.sounds);
        }
        w
    }
}

/// What a drop did, for the registers it leaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dropped {
    /// No cue at the place: the count as it is.
    None { count: u32 },
    /// The last cue dropped: the count left.
    Last { count: u32 },
    /// The cues after it moved up.
    Moved,
}

/// The cue at `place` taken out, the ones after it moved up. Refused: more than 16.
pub fn drop_cue(q: &mut CueQueue, place: u32) -> Result<Dropped, Invalid> {
    if q.count > MOST_CUES {
        return Err(Invalid("a cue queue of more than 16"));
    }
    if place >= q.count {
        return Ok(Dropped::None { count: q.count });
    }
    q.count -= 1;
    if place >= q.count {
        return Ok(Dropped::Last { count: q.count });
    }
    let (at, end) = (place as usize, q.count as usize);
    q.cues.copy_within(at + 1..end + 1, at);
    Ok(Dropped::Moved)
}

/// What a turn of the cue queue did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CueTurn {
    /// The object not in use.
    NotInUse,
    /// No cue queued.
    Empty,
    /// The next sound not due yet.
    NotDue,
    /// The cues ran out: the place looked at last.
    RanOut { place: u32 },
    /// This sound to play (the caller sends it at the sample's own volume).
    Play { sound: u32 },
}

/// A turn of a car's cue queue at `clock`: once the next sound is due (or none begun) the
/// place moves on; a cue played out is dropped and the next begun, and the place's sound
/// is due again after its length (`duration`, at the sample's own rate) and 25 ms (sounds
/// 0x13F and 0x140 0x1000000 later: never). `drop(q, place)` takes the first cue out
/// (`drop_cue` at 0; the place it was at given). Refused (before anything): more than 16
/// cues or a cue of more than six sounds; what `duration` and `drop` refuse.
pub fn cue_turn<E: From<Invalid>>(
    q: &mut CueQueue,
    in_use: bool,
    clock: u32,
    duration: impl FnOnce(u32) -> Result<u32, E>,
    mut drop: impl FnMut(&mut CueQueue, u32) -> Result<(), E>,
) -> Result<CueTurn, E> {
    if !in_use {
        return Ok(CueTurn::NotInUse);
    }
    if q.count == 0 {
        return Ok(CueTurn::Empty);
    }
    if q.count > MOST_CUES {
        return Err(Invalid("a cue queue of more than 16").into());
    }
    if q.cues[..q.count as usize]
        .iter()
        .any(|c| c.count > MOST_SOUNDS)
    {
        return Err(Invalid("a cue of more than six sounds").into());
    }
    if q.place != u32::MAX && (clock.wrapping_sub(q.due) as i32) < 0 {
        return Ok(CueTurn::NotDue);
    }
    q.place = q.place.wrapping_add(1);
    while q.place >= q.cues[0].count {
        let place = q.place;
        drop(q, place)?;
        q.place = 0;
        if q.count == 0 {
            q.place = u32::MAX;
            return Ok(CueTurn::RanOut { place });
        }
    }
    let sound = q.cues[0].sounds[q.place as usize];
    q.due = duration(sound)?.wrapping_add(0x19).wrapping_add(clock);
    if sound == 0x13F || sound == 0x140 {
        q.due = q.due.wrapping_add(0x100_0000);
    }
    Ok(CueTurn::Play { sound })
}

/// A timed slot of the race's sounds: six dwords.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimedSlot {
    pub sound: u32,
    /// 1: start it.
    pub start: u32,
    pub playing: u32,
    /// When it ends.
    pub ends: u32,
    pub volume: u32,
    /// `OWN`: the sample's own.
    pub rate: u32,
}

/// The sound that starts even while playing.
pub const ALWAYS_STARTS: u32 = 0xE9;

/// What a slot's turn did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotTurn {
    Nothing,
    /// Looked at the clock (its end may have come: not playing then).
    Checked,
    /// Started: the volume and rate to send it with, and its end.
    Start {
        volume: u32,
        rate: u32,
        ends: u32,
    },
}

/// A slot's turn at `clock`: asked to start, it starts unless playing (sound 0xE9 even
/// then): the volume and rate (`settings`), its end the clock + its length (`duration`),
/// playing. Else a playing one stops once the clock is at or past its end. What the two
/// refuse, before anything is written.
pub fn slot_turn<E>(
    s: &mut TimedSlot,
    clock: u32,
    settings: impl FnOnce(u32, u32, u32) -> Result<(u32, u32), E>,
    duration: impl FnOnce(u32, u32) -> Result<u32, E>,
) -> Result<SlotTurn, E> {
    if s.start == 1 {
        if s.sound == ALWAYS_STARTS || s.playing != 1 {
            let (volume, rate) = settings(s.sound, s.volume, s.rate)?;
            let ends = duration(s.sound, rate)?.wrapping_add(clock);
            s.start = 0;
            s.ends = ends;
            s.playing = 1;
            return Ok(SlotTurn::Start { volume, rate, ends });
        }
        s.start = 0;
    } else if s.playing != 1 {
        return Ok(SlotTurn::Nothing);
    }
    if (clock.wrapping_sub(s.ends) as i32) >= 0 {
        s.playing = 0;
    }
    Ok(SlotTurn::Checked)
}

/// What a one-off sound's turn did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OneOff {
    /// None waiting.
    Nothing,
    /// Taken, but too quiet: nothing sent.
    Quiet { volume: u32, rate: u32 },
    /// Taken, and stopped (a volume of 0, the cue sound).
    Stop { volume: u32, rate: u32 },
    /// Taken, to be played at its place.
    Play { volume: u32, rate: u32 },
}

/// A car's one-off sound (`waiting`, -2 none) taken, with its volume and the sample's own
/// rate (`settings`): under `least` (4 for a hit, 1 for a cue) it is not played, and with
/// `stop_quiet` (the cue) stopped instead. `record` refuses a sound past the samples,
/// before anything.
pub fn one_off<E>(
    waiting: &mut u32,
    volume: u32,
    least: u32,
    stop_quiet: bool,
    record: impl FnOnce(u32) -> Result<(), E>,
    settings: impl FnOnce(u32, u32, u32) -> Result<(u32, u32), E>,
) -> Result<(u32, OneOff), E> {
    let sound = *waiting;
    if sound == NONE {
        return Ok((sound, OneOff::Nothing));
    }
    record(sound)?;
    *waiting = NONE;
    let (volume, rate) = settings(sound, volume, OWN)?;
    Ok((
        sound,
        if volume >= least {
            OneOff::Play { volume, rate }
        } else if stop_quiet {
            OneOff::Stop { volume, rate }
        } else {
            OneOff::Quiet { volume, rate }
        },
    ))
}

/// A whole number as the float `fild` / `fstp` makes it.
pub fn from_int(v: i32) -> u32 {
    (v as f32).to_bits()
}

/// A car's voice marks by its car's +0x25C (`kind`) and its place `n`: (whether it
/// sounds, and its voice when set: 0, or !n for a computer's car).
pub fn voice_mark(n: u32, kind: u32) -> (u32, Option<u32>) {
    match kind {
        0 => (0, None),
        1 => (1, Some(0)),
        _ => (1, Some(!n)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cue(sounds: &[u32]) -> Cue {
        let mut c = Cue {
            count: sounds.len() as u32,
            ..Cue::default()
        };
        c.sounds[..sounds.len()].copy_from_slice(sounds);
        c
    }

    #[test]
    fn cues() {
        let mut q = CueQueue {
            place: u32::MAX,
            count: 2,
            ..CueQueue::default()
        };
        q.cues[0] = cue(&[10, 11]);
        q.cues[1] = cue(&[0x13F]);
        assert_eq!(CueQueue::from_words(&q.to_words()), q);
        let d = |_| Ok::<u32, Invalid>(100);
        let x = |q: &mut CueQueue, _| drop_cue(q, 0).map(|_| ());
        assert_eq!(
            cue_turn(&mut q, true, 1000, d, x),
            Ok(CueTurn::Play { sound: 10 })
        );
        assert_eq!((q.place, q.due), (0, 1125));
        assert_eq!(cue_turn(&mut q, true, 1124, d, x), Ok(CueTurn::NotDue));
        assert_eq!(
            cue_turn(&mut q, true, 1125, d, x),
            Ok(CueTurn::Play { sound: 11 })
        );
        assert_eq!(
            cue_turn(&mut q, true, 1300, d, x),
            Ok(CueTurn::Play { sound: 0x13F })
        );
        assert_eq!((q.count, q.due), (1, 1425 + 0x100_0000));
        q.due = 0;
        assert_eq!(
            cue_turn(&mut q, true, 2000, d, x),
            Ok(CueTurn::RanOut { place: 1 })
        );
        assert_eq!(q.place, u32::MAX);
        assert_eq!(cue_turn(&mut q, false, 0, d, x), Ok(CueTurn::NotInUse));
        let mut q = CueQueue {
            count: 3,
            ..CueQueue::default()
        };
        q.cues[1].first = 5;
        q.cues[2].first = 6;
        assert_eq!(drop_cue(&mut q, 0), Ok(Dropped::Moved));
        assert_eq!((q.count, q.cues[0].first, q.cues[1].first), (2, 5, 6));
        assert_eq!(drop_cue(&mut q, 4), Ok(Dropped::None { count: 2 }));
    }

    #[test]
    fn slots_and_one_offs() {
        let set = |_, v, r| Ok::<_, ()>((v, r));
        let mut s = TimedSlot {
            sound: 3,
            start: 1,
            volume: 9,
            rate: 7,
            ..TimedSlot::default()
        };
        assert_eq!(
            slot_turn(&mut s, 100, set, |_, _| Ok(50)),
            Ok(SlotTurn::Start {
                volume: 9,
                rate: 7,
                ends: 150
            })
        );
        assert_eq!(
            slot_turn(&mut s, 149, set, |_, _| Ok(50)),
            Ok(SlotTurn::Checked)
        );
        assert_eq!(s.playing, 1);
        assert_eq!(
            slot_turn(&mut s, 150, set, |_, _| Ok(50)),
            Ok(SlotTurn::Checked)
        );
        assert_eq!(s.playing, 0);
        assert_eq!(
            slot_turn(&mut s, 150, set, |_, _| Ok(50)),
            Ok(SlotTurn::Nothing)
        );
        let mut w = 5;
        assert_eq!(
            one_off(
                &mut w,
                3,
                4,
                false,
                |_| Ok::<_, ()>(()),
                |_, v, _| Ok((v, 1))
            ),
            Ok((5, OneOff::Quiet { volume: 3, rate: 1 }))
        );
        assert_eq!(w, NONE);
        let mut w = 5;
        assert_eq!(
            one_off(
                &mut w,
                0,
                1,
                true,
                |_| Ok::<_, ()>(()),
                |_, v, _| Ok((v, 1))
            ),
            Ok((5, OneOff::Stop { volume: 0, rate: 1 }))
        );
        assert_eq!(voice_mark(2, 3), (1, Some(!2)));
        assert_eq!(from_int(-3), (-3.0f32).to_bits());
    }
}
