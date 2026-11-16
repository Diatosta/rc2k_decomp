//! The start's count down: a car's second of the count down and when its beep is cued; the beep
//! played for a listed car; a hold counted down after the start.

/// The count down's length in hundredths (held there) and the seconds it shows.
pub const LENGTH: u32 = 1500;
pub const SECONDS: u32 = 6;

/// What the count down does for one car this tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Beep {
    /// Finished, or not running: nothing.
    Idle,
    /// Before the count down's start: the time since it (below 0).
    Before(u32),
    /// The car's second (-1 from 6 on) and the hundredths past it; `cue` on a whole
    /// second not -1 (the second's sound is cued).
    At {
        second: u32,
        remainder: u32,
        cue: bool,
    },
}

/// For a car not finished and running, the time since the count down's start (`elapsed`, asked
/// only then), held at `LENGTH`, in seconds and hundredths.
pub fn countdown_beep(finished: u32, running: u32, elapsed: impl FnOnce() -> u32) -> Beep {
    if finished != 0 || running == 0 {
        return Beep::Idle;
    }
    let mut t = elapsed();
    if (t as i32) < 0 {
        return Beep::Before(t);
    }
    if (t.wrapping_sub(LENGTH) as i32) >= 0 {
        t = LENGTH;
    }
    let (mut second, remainder) = (t / 100, t % 100);
    if second >= SECONDS {
        second = u32::MAX;
    }
    Beep::At {
        second,
        remainder,
        cue: remainder == 0 && second != u32::MAX,
    }
}

/// How the beep is played (the sound and its volume).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BeepSound {
    /// The car's sound slot plus the beep.
    pub sound: u32,
    /// `BEEP_VOLUME` (-0x6F).
    pub volume: u32,
}

/// The beeps' volume.
pub const BEEP_VOLUME: u32 = 0xFFFF_FF91;

/// The beep for a listed car's sound slot 0 or 1 (unsigned), played on its channel; none for
/// other slots.
pub fn beep_sound(slot: u32, sound: u32) -> Option<BeepSound> {
    (slot <= 1).then(|| BeepSound {
        sound: slot.wrapping_add(sound),
        volume: BEEP_VOLUME,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beeps() {
        assert_eq!(countdown_beep(1, 1, || unreachable!()), Beep::Idle);
        assert_eq!(countdown_beep(0, 0, || unreachable!()), Beep::Idle);
        assert_eq!(countdown_beep(0, 1, || u32::MAX), Beep::Before(u32::MAX));
        assert_eq!(
            countdown_beep(0, 1, || 300),
            Beep::At {
                second: 3,
                remainder: 0,
                cue: true
            }
        );
        assert_eq!(
            countdown_beep(0, 1, || 599),
            Beep::At {
                second: 5,
                remainder: 99,
                cue: false
            }
        );
        assert_eq!(
            countdown_beep(0, 1, || 0x7FFF_FFFF),
            Beep::At {
                second: u32::MAX,
                remainder: 0,
                cue: false
            }
        );
    }

    #[test]
    fn beep_sounds() {
        assert_eq!(beep_sound(2, 5), None);
        assert_eq!(beep_sound(u32::MAX, 5), None);
        assert_eq!(
            beep_sound(1, 5),
            Some(BeepSound {
                sound: 6,
                volume: BEEP_VOLUME
            })
        );
    }
}
