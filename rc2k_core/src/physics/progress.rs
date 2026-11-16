//! The work car's progress along the stage: the start's "go", the splits taken and the finish,
//! the warnings for a car off the line or turned away from it, whether the car's checkpoint has
//! one after it, and the checkpoint looked for near the last one and the next split's point
//! passed (their x87 distance and angle stay with the callers).

use crate::Invalid;

/// The start's "go" notice.
pub const GO_NOTICE: u32 = 0xC7;
/// The notice of a split taken in race mode 2.
pub const SPLIT_NOTICE: u32 = 0xC9;
/// The finish notice.
pub const FINISH_NOTICE: u32 = 0xC8;

/// The cheat bit that keeps the "go" back.
pub const NO_GO_CHEAT: u32 = 16;

/// What the "go" notice tests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GoFacts {
    /// The grades' mode.
    pub mode: u32,
    /// The start's count down.
    pub countdown: u32,
    /// The "go" shown.
    pub gone: u32,
    /// Finished.
    pub finished: u32,
    /// The cheat bits.
    pub cheats: u32,
}

impl GoFacts {
    /// Whether the "go" is given now: mode 1, the count down, `gone` and `finished` 0 and the
    /// cheat bit clear.
    pub fn go_due(&self) -> bool {
        self.mode == 1
            && self.countdown == 0
            && self.gone == 0
            && self.finished == 0
            && (self.cheats >> NO_GO_CHEAT) & 1 == 0
    }
}

/// The highest split taken (the splits' tables hold 0x40).
pub const LAST_SPLIT: u32 = 0x3F;

/// Whether a split is taken: the splits taken below the checkpoints passed (unsigned); a split
/// past `LAST_SPLIT` then is refused.
pub fn split_due(split: u32, checks: u32) -> Result<bool, Invalid> {
    if split >= checks {
        return Ok(false);
    }
    if split > LAST_SPLIT {
        return Err(Invalid("a split past 0x3F"));
    }
    Ok(true)
}

/// Whether the car is finished once the splits taken are `split` of `count`.
pub fn finished_at(split: u32, count: u32) -> bool {
    split == count
}

/// The split's seconds in hundredths (wrapping).
pub fn split_bonus(seconds: u32) -> u32 {
    seconds.wrapping_mul(100)
}

/// A difference made positive.
pub fn spread(a: u32, b: u32) -> u32 {
    let d = a.wrapping_sub(b);
    if (d as i32) < 0 { d.wrapping_neg() } else { d }
}

/// The highest checkpoint the warnings take (the angles table holds 0x1418 entries; index
/// 0x1418 itself passes the check, one past the table: kept).
pub const LAST_CHECKPOINT: u32 = 0x1418;

/// The next checkpoint, when the car's checkpoint has one below the count; one past
/// `LAST_CHECKPOINT` is refused then.
pub fn checkpoint_ahead(checkpoint: u32, count: u32) -> Result<Option<u32>, Invalid> {
    let next = checkpoint.wrapping_add(1);
    if (next.wrapping_sub(count) as i32) >= 0 {
        return Ok(None);
    }
    if checkpoint > LAST_CHECKPOINT {
        return Err(Invalid("a checkpoint below 0 or past 0x1418"));
    }
    Ok(Some(next))
}

/// A counter's cue time when put back, and the step it moves on after a cue.
pub const CUE_FIRST: u32 = 300;
pub const CUE_STEP: u32 = 3000;

/// The turned-away cue, checked first.
pub const TURNED_CUE: u32 = 0x50;
/// The off-the-line cue, checked second.
pub const OFF_LINE_CUE: u32 = 0x52;

/// The warnings' counters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Warnings {
    /// Steps off the line.
    pub off: u32,
    /// Steps turned away.
    pub turned: u32,
    /// The next off-line cue.
    pub off_time: u32,
    /// The next turned cue.
    pub turned_time: u32,
}

/// Whether the car is off the line: not near a checkpoint, or its checkpoint four or more
/// from the last (`gap` asked only when near).
pub fn off_line(near: bool, gap: impl FnOnce() -> u32) -> bool {
    !near || (gap().wrapping_sub(4) as i32) >= 0
}

impl Warnings {
    /// The off-line counter one up, or put back to 0 with its cue at `CUE_FIRST`.
    pub fn count_off(&mut self, off: bool) {
        if off {
            self.off = self.off.wrapping_add(1);
        } else {
            self.off = 0;
            self.off_time = CUE_FIRST;
        }
    }

    /// The turned counter: one up when the heading `turn` is at or past `limit` (unsigned),
    /// else put back to 0 with its cue at `CUE_FIRST`.
    pub fn count_turned(&mut self, turn: u32, limit: u32) {
        if turn < limit {
            self.turned = 0;
            self.turned_time = CUE_FIRST;
        } else {
            self.turned = self.turned.wrapping_add(1);
        }
    }
}

/// A counter's cue: due when the count is at or past its time; returns the time moved on.
pub fn cue_due(count: u32, time: u32) -> Option<u32> {
    ((count.wrapping_sub(time) as i32) >= 0).then(|| time.wrapping_add(CUE_STEP))
}

/// The checkpoints looked at first, about the last one: (it - 1, it + 1), wrapping.
pub fn near_window(last: u32) -> (u32, u32) {
    let from = last.wrapping_sub(1);
    (from, from.wrapping_add(2))
}

/// The distance (whole number) past which every checkpoint is looked at.
pub const NEAR_DISTANCE: u32 = 0xA0000;

/// Whether the nearest of the window is too far (unsigned): then all are looked at.
pub fn search_all(distance: u32) -> bool {
    distance > NEAR_DISTANCE
}

/// The fatal error for a split's face past the face count.
pub const SEGMENT_FATAL: u32 = 0x8060_1843;

/// The split and its checkpoint the tables hold: the split is checked first, the checkpoint
/// asked for after.
pub fn check_split_point(split: u32, checkpoint: impl FnOnce() -> u32) -> Result<u32, Invalid> {
    if split > LAST_SPLIT + 1 {
        return Err(Invalid("a split past 0x40"));
    }
    let cp = checkpoint();
    if cp > LAST_CHECKPOINT {
        return Err(Invalid("a checkpoint past 0x1418"));
    }
    Ok(cp)
}

/// Whether the split's face is a course face (else a fatal error).
pub fn face_known(face: u32, face_count: u32) -> bool {
    face < face_count
}

/// Whether the car has passed the split's point: its angle from the point not below the
/// checkpoint's heading less a quarter turn (signed). Returns (passed, that heading).
pub fn passed_point(angle: u32, heading: u32) -> (bool, u32) {
    let quarter_back = heading.wrapping_sub(0x4000_0000);
    ((angle.wrapping_sub(quarter_back) as i32) >= 0, quarter_back)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_points() {
        assert_eq!(near_window(0), (u32::MAX, 1));
        assert!(search_all(0xA0001) && !search_all(0xA0000));
        assert_eq!(check_split_point(0x40, || 5), Ok(5));
        assert!(check_split_point(0x41, || panic!()).is_err());
        assert!(check_split_point(0, || 0x1419).is_err());
        assert!(face_known(3, 4) && !face_known(4, 4));
        assert_eq!(passed_point(0x4000_0000, 0x8000_0000), (true, 0x4000_0000));
        assert!(!passed_point(0x3FFF_0000, 0x8000_0000).0);
    }

    #[test]
    fn go() {
        let mut f = GoFacts {
            mode: 1,
            ..Default::default()
        };
        assert!(f.go_due());
        f.cheats = 1 << 16;
        assert!(!f.go_due());
        f.cheats = 1 << 15;
        f.countdown = 1;
        assert!(!f.go_due());
    }

    #[test]
    fn splits() {
        assert_eq!(split_due(2, 3), Ok(true));
        assert_eq!(split_due(3, 3), Ok(false));
        assert!(split_due(0x40, 0x41).is_err());
        assert!(finished_at(5, 5));
        assert_eq!(split_bonus(30), 3000);
    }

    #[test]
    fn ahead() {
        assert_eq!(checkpoint_ahead(3, 5), Ok(Some(4)));
        assert_eq!(checkpoint_ahead(4, 5), Ok(None));
        assert_eq!(
            checkpoint_ahead(u32::MAX, 5),
            Err(Invalid("a checkpoint below 0 or past 0x1418"))
        );
        assert_eq!(
            checkpoint_ahead(0x1419, 0x2000),
            Err(Invalid("a checkpoint below 0 or past 0x1418"))
        );
    }

    #[test]
    fn warnings() {
        assert_eq!(spread(3, 7), 4);
        assert_eq!(spread(7, 3), 4);
        assert!(off_line(false, || unreachable!()));
        assert!(!off_line(true, || 3));
        assert!(off_line(true, || 4));
        let mut w = Warnings {
            off: 299,
            turned: 5,
            off_time: 300,
            turned_time: 300,
        };
        w.count_off(true);
        w.count_turned(10, 20);
        assert_eq!((w.off, w.turned, w.turned_time), (300, 0, 300));
        assert_eq!(cue_due(w.turned, w.turned_time), None);
        assert_eq!(cue_due(w.off, w.off_time), Some(3300));
        assert_eq!(cue_due(0x7FFF_FFFF, 0x8000_0000), None);
    }
}
