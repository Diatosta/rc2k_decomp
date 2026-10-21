//! The start's count down (converted 2026-10-21 from `drive::countdown_beep` 0x440EA3
//! and `hold_count_tick` 0x460C10): a car's second of the count down and when its beep
//! is cued; a hold counted down after the start.

/// The count down's length in hundredths (held there) and the seconds it shows.
pub const LENGTH: u32 = 1500;
pub const SECONDS: u32 = 6;

/// What `countdown_beep` does for one car.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Beep {
    /// Finished, or not running: nothing.
    Idle,
    /// Before the count down's start: the time since it (below 0; the original's eax).
    Before(u32),
    /// The car's second (-1 from 6 on) and the hundredths past it; `cue` on a whole
    /// second not -1 (the second's sound is cued).
    At {
        second: u32,
        remainder: u32,
        cue: bool,
    },
}

/// `drive::countdown_beep`'s rule: for a car not finished and running, the time since
/// the count down's start (`elapsed`, asked only then) held at 1500, in seconds.
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

/// `drive::hold_count_tick`: the hold (`dword_6235EC`) counted down to 0.
pub fn hold_tick(hold: &mut u32) {
    if *hold != 0 {
        *hold -= 1;
    }
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
    fn hold() {
        let mut h = 2;
        hold_tick(&mut h);
        hold_tick(&mut h);
        hold_tick(&mut h);
        assert_eq!(h, 0);
    }
}
