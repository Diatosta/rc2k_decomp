//! The race's split times (converted 2026-10-21 from `physics::keep_split_time`
//! 0x436B3D and `compare_split_times` 0x436E11): where a split time is kept, and the car's
//! place and nearest gaps among the ten times kept for a split (race mode 3).

use crate::Invalid;

/// The kept split times' room (`dword_601C68`, 0x40 entries).
pub const KEPT_ROOM: u32 = 0x40;

/// `physics::keep_split_time`: the entry for a split time, the checkpoints passed
/// (`g_car` +0x90) less one, held to 0 ..= the split count (`g_splitColourCount`), both
/// by the sign of the difference. Past the room is refused (the original writes on).
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

/// The highest split `compare_split_times` takes (the kept tables' size not worked out).
pub const LAST_COMPARED_SPLIT: u32 = 27;

/// The split's figures (`g_car` +0xA0C .. +0xA1C).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SplitFigures {
    /// +0xA0C: the place, 11 less one for each kept time beaten; -1 when none was.
    pub place: u32,
    /// +0xA10: the gap to the nearest kept time ahead (x10, made positive).
    pub gap_ahead: u32,
    /// +0xA14: the gap to the nearest kept time behind (x10, below 0).
    pub gap_behind: u32,
    /// +0xA18: the name of the one ahead (-1: none).
    pub name_ahead: u32,
    /// +0xA1C: the name of the one behind (-1: none).
    pub name_behind: u32,
}

/// `compare_split_times`' result: the figures, and the last value the loop left in eax.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Compared {
    pub figures: SplitFigures,
    pub last: u32,
}

/// `physics::compare_split_times`: the car's `time` against the ten `kept` for the split
/// (signed differences, x10 wrapping); `name(c)` is asked for a kept time's name only
/// when it is the nearest so far.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots() {
        assert_eq!(kept_split_slot(0, 5), Ok(0));
        assert_eq!(kept_split_slot(3, 5), Ok(2));
        assert_eq!(kept_split_slot(9, 5), Ok(5));
        assert_eq!(kept_split_slot(0x8000_0001, 0x7FFF_FFFF), Ok(0));
        assert!(kept_split_slot(0x50, 0x45).is_err());
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
}
