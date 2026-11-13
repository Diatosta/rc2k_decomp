//! The race's split times: where a split time is kept, the car's place and nearest gaps among
//! the ten times kept for a split (mode 3) or the 90 rivals' times scaled to the split
//! (mode 1), the checkpoints followed through the splits and the race timers' steps by mode.

use crate::Invalid;

/// The kept split times' room.
pub const KEPT_ROOM: u32 = 0x40;

/// The entry for a split time: the checkpoints passed less one, held to 0 ..= the split count
/// (signed). Past the room is refused.
pub fn kept_split_slot(checks: u32, count: u32) -> Result<u32, Invalid> {
    let mut k = checks.wrapping_sub(1);
    if (k as i32) < 0 {
        k = 0;
    }
    if (k.wrapping_sub(count) as i32) >= 0 {
        k = count;
    }
    if k >= KEPT_ROOM {
        return Err(Invalid("a split index past 0x40"));
    }
    Ok(k)
}

/// The highest split compared (each kept table is 28 dwords).
pub const LAST_COMPARED_SPLIT: u32 = 27;

/// The split's figures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SplitFigures {
    /// The place: 11 less one for each kept time beaten; -1 when none was.
    pub place: u32,
    /// The gap to the nearest kept time ahead (x10, made positive).
    pub gap_ahead: u32,
    /// The gap to the nearest kept time behind (x10, below 0).
    pub gap_behind: u32,
    /// The name of the one ahead (-1: none).
    pub name_ahead: u32,
    /// The name of the one behind (-1: none).
    pub name_behind: u32,
}

/// The figures, and the last value the loop left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Compared {
    pub figures: SplitFigures,
    pub last: u32,
}

/// The car's `time` against the ten `kept` for the split (signed differences, x10 wrapping);
/// `name(c)` is asked for a kept time's name only when it is the nearest so far.
pub fn compare_split(time: u32, kept: &[u32; 10], name: impl Fn(usize) -> u32) -> Compared {
    let mut f = SplitFigures {
        place: 11,
        gap_ahead: 999_999,
        gap_behind: (-999_999i32) as u32,
        name_ahead: u32::MAX,
        name_behind: u32::MAX,
    };
    let mut last = time;
    for (c, &k) in kept.iter().enumerate() {
        let beaten = (time.wrapping_sub(k) as i32) < 0;
        last = time.wrapping_sub(k).wrapping_mul(10);
        if beaten {
            f.place = f.place.wrapping_sub(1);
            if (last.wrapping_sub(f.gap_behind) as i32) >= 0 {
                f.gap_behind = last;
                last = name(c);
                f.name_behind = last;
            }
        } else if (last.wrapping_sub(f.gap_ahead) as i32) < 0 {
            f.gap_ahead = last;
            last = name(c);
            f.name_ahead = last;
        }
    }
    if f.place >= 11 {
        f.place = u32::MAX;
    }
    f.gap_ahead = f.gap_ahead.wrapping_neg();
    Compared { figures: f, last }
}

/// The rivals of race mode 1.
pub const RIVALS: usize = 90;

/// A rival's default figures: (time 12000 + 1000 c, car c % 23, name's string id 0x5A5 + c).
pub fn default_rival(c: u32) -> (u32, u32, u32) {
    (c.wrapping_mul(1000).wrapping_add(12000), c % 23, c + 0x5A5)
}

/// A 64-bit value divided by `d`: (quotient, remainder); refused for a zero divisor or a
/// quotient past 32 bits.
pub fn div(value: u64, d: u32) -> Result<(u32, u32), Invalid> {
    if d == 0 || value / d as u64 > u32::MAX as u64 {
        return Err(Invalid("a division the original's div would fault on"));
    }
    Ok(((value / d as u64) as u32, (value % d as u64) as u32))
}

/// The scale of a split: its base x 1000 / the divisor, as `div` leaves it.
pub fn split_scale(base: u32, divisor: u32) -> Result<(u32, u32), Invalid> {
    div(base as u64 * 1000, divisor)
}

/// A rival's time scaled to the split: time x scale / 1000, as `div` leaves it.
pub fn scaled_time(time: u32, scale: u32) -> Result<(u32, u32), Invalid> {
    div(time as u64 * scale as u64, 1000)
}

/// The car's place among the scaled times: one past the first not below its time (signed),
/// `RIVALS` at most; and the last time read.
pub fn rank(time: u32, scaled: impl Fn(usize) -> u32) -> (u32, u32) {
    let mut c = 0;
    let mut last;
    loop {
        last = scaled(c);
        if (last.wrapping_sub(time) as i32) >= 0 {
            break;
        }
        c += 1;
        if c == RIVALS - 1 {
            break;
        }
    }
    (c as u32 + 1, last)
}

/// The gap (x10, wrapping) from an earlier time to a later one.
pub fn gap(earlier: u32, later: u32) -> u32 {
    later.wrapping_sub(earlier).wrapping_mul(10)
}

/// What the split's figure is made from, by the race mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitFigure {
    /// Mode 2: the car's place.
    Place,
    /// Mode 1: ranked among the rivals.
    Rank,
    /// Mode 3: compared against the ten kept.
    Compare,
    /// Any other mode: nothing.
    Nothing,
}

pub fn split_figure(mode: u32) -> SplitFigure {
    match mode {
        2 => SplitFigure::Place,
        1 => SplitFigure::Rank,
        3 => SplitFigure::Compare,
        _ => SplitFigure::Nothing,
    }
}

/// The highest checkpoint.
pub const LAST_CHECKPOINT: u32 = 0x1418;
/// The highest split.
pub const LAST_SPLIT: u32 = 0x40;

/// The checkpoint found and the car's split, checked before anything is written.
pub fn check_tracked(checkpoint: u32, split: u32) -> Result<(), Invalid> {
    if checkpoint > LAST_CHECKPOINT {
        return Err(Invalid("a checkpoint past 0x1418"));
    }
    if split > LAST_SPLIT {
        return Err(Invalid("a split past 0x40"));
    }
    Ok(())
}

/// At the split's last checkpoint: the checkpoint's count less 1 is the split.
pub fn at_split_end(count: u32, split: u32) -> bool {
    count.wrapping_sub(1) == split
}

/// A split point still ahead: the split below the splits' count (unsigned).
pub fn split_ahead(split: u32, splits: u32) -> bool {
    split < splits
}

/// The race timers' steps by the race mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerRun {
    /// Past mode 3: nothing.
    None,
    /// Mode 2: the start's count down, the counted split and the go notice.
    Start,
    /// Modes 0, 1, 3: the split.
    Race,
}

/// The steps for race mode `mode`.
pub fn timer_run(mode: u32) -> TimerRun {
    match mode {
        2 => TimerRun::Start,
        0 | 1 | 3 => TimerRun::Race,
        _ => TimerRun::None,
    }
}

/// The timers' rate: 1, or 1 times `factor` once the car has finished.
pub fn timer_rate(finished: bool, factor: f32) -> f32 {
    if finished {
        (1.0 * factor as f64) as f32
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn figures() {
        assert_eq!(split_figure(2), SplitFigure::Place);
        assert_eq!(split_figure(1), SplitFigure::Rank);
        assert_eq!(split_figure(3), SplitFigure::Compare);
        assert_eq!(split_figure(0), SplitFigure::Nothing);
    }

    #[test]
    fn slots() {
        assert_eq!(kept_split_slot(0, 5), Ok(0));
        assert_eq!(kept_split_slot(3, 5), Ok(2));
        assert_eq!(kept_split_slot(9, 5), Ok(5));
        assert_eq!(kept_split_slot(0x8000_0001, 0x7FFF_FFFF), Ok(0));
        assert!(kept_split_slot(0x50, 0x45).is_err());
    }

    #[test]
    fn ranked() {
        assert_eq!(default_rival(3), (15000, 3, 0x5A8));
        assert_eq!(split_scale(5, 2), Ok((2500, 0)));
        assert!(split_scale(5, 0).is_err());
        assert!(scaled_time(u32::MAX, u32::MAX).is_err());
        assert_eq!(scaled_time(12345, 500), Ok((6172, 500)));
        let t = |c: usize| 100 * c as u32;
        assert_eq!(rank(250, t), (4, 300));
        assert_eq!(rank(0, t), (1, 0));
        assert_eq!(rank(100_000, t), (90, 8800));
        assert_eq!(gap(300, 250), (-500i32) as u32);
    }

    #[test]
    fn compared() {
        let kept = [100, 200, 300, 400, 500, 600, 700, 800, 900, 1000];
        let c = compare_split(450, &kept, |c| 0x100 + c as u32);
        assert_eq!(
            c.figures,
            SplitFigures {
                place: 5,
                gap_ahead: (-500i32) as u32,
                gap_behind: (-500i32) as u32,
                name_ahead: 0x103,
                name_behind: 0x104,
            }
        );
        assert_eq!(c.last, (-5500i32) as u32);
        let slow = compare_split(5000, &kept, |_| 7);
        assert_eq!(
            (slow.figures.place, slow.figures.name_behind),
            (u32::MAX, u32::MAX)
        );
        let fast = compare_split(0, &kept, |_| 7);
        assert_eq!((fast.figures.place, fast.figures.name_ahead), (1, u32::MAX));
        assert_eq!(fast.figures.gap_ahead, (-999_999i32) as u32);
    }

    #[test]
    fn tracked() {
        assert!(check_tracked(0x1418, 0x40).is_ok());
        assert!(check_tracked(0x1419, 0).is_err());
        assert!(check_tracked(0, 0x41).is_err());
        assert!(at_split_end(3, 2) && !at_split_end(0, 0));
        assert!(split_ahead(1, 2) && !split_ahead(2, 2));
    }

    #[test]
    fn timers() {
        assert_eq!(timer_run(2), TimerRun::Start);
        assert_eq!(timer_run(3), TimerRun::Race);
        assert_eq!(timer_run(4), TimerRun::None);
        assert_eq!(timer_rate(false, 0.5), 1.0);
        assert_eq!(timer_rate(true, 0.5), 0.5);
    }
}
