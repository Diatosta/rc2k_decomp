//! The stage's own sounds (from `sound::stage_sounds` 0x40E2A9): the
//! timed sound's wait and the two sounds at a random pitch and place.

/// The timed sound's wait (`StageSounds.timed_arm` the arming flag, `StageSounds.timed_delay` the delay,
/// `StageSounds.timed_due` when due, `StageSounds.timed_waiting` waiting) and its slot's start flag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Wait {
    pub arm: u32,
    pub delay: u32,
    pub due: u32,
    pub waiting: u32,
    pub start: u32,
}

/// The wait's turn at `clock`: armed (cleared), due the clock + the delay and waiting;
/// while waiting and not due (`js`) the slot is skipped (false); once due its slot is
/// told to start and the wait ends.
pub fn wait_turn(w: &mut Wait, clock: u32) -> bool {
    if w.arm == 1 {
        w.arm = 0;
        w.due = clock.wrapping_add(w.delay);
        w.waiting = 1;
    }
    if w.waiting == 1 {
        if (clock.wrapping_sub(w.due) as i32) < 0 {
            return false;
        }
        w.start = 1;
        w.waiting = 0;
    }
    true
}

/// A sound at a random pitch and place, from one random number `n`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Random {
    /// 0x21 + the low five bits.
    pub volume: u32,
    /// The mix rate + (the next nine bits - 0x100) x the mix rate / 4096 (`sar`).
    pub rate: u32,
    /// (nine bits - 0x100) x 0.01, and nine bits x 0.01.
    pub across: f32,
    pub ahead: f32,
}

/// A whole number x 0.01 as the x87 makes it (`fild`, `fmul` of the float 0.01, `fstp`:
/// the product is exact, so one rounding).
pub fn hundredth(v: i32) -> f32 {
    (v as f64 * 0.01f32 as f64) as f32
}

pub fn random(n: u32, mix_rate: u32) -> Random {
    let rate = ((((n >> 5) & 0x1FF).wrapping_sub(0x100) as i32).wrapping_mul(mix_rate as i32) >> 12)
        as u32;
    Random {
        volume: (n & 0x1F) + 0x21,
        rate: rate.wrapping_add(mix_rate),
        across: hundredth(((n >> 14) & 0x1FF) as i32 - 0x100),
        ahead: hundredth(((n >> 23) & 0x1FF) as i32),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waits_and_randoms() {
        let mut w = Wait {
            arm: 1,
            delay: 50,
            ..Wait::default()
        };
        assert!(!wait_turn(&mut w, 100));
        assert_eq!((w.arm, w.due, w.waiting), (0, 150, 1));
        assert!(wait_turn(&mut w, 150));
        assert_eq!((w.start, w.waiting), (1, 0));
        let r = random(0x1F | (0x180 << 5) | (0x100 << 14) | (2 << 23), 22050);
        assert_eq!((r.volume, r.rate), (0x40, 22050 + ((0x80 * 22050) >> 12)));
        assert_eq!((r.across, r.ahead), (0.0, hundredth(2)));
    }
}
