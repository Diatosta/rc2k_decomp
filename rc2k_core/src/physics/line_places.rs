//! Where a car may be put on the line (Converted from
//! `physics::reset_point_blocked` 0x43F0DF, `physics::reset_car` 0x43EFEF (its search),
//! `drive::checkpoint_taken` 0x4375A5 and `drive::car_near_checkpoint` 0x4375F2): a
//! checkpoint too near the stage's ends or another car is no place for it. Every
//! comparison takes the sign of the difference (`js` / `jns`), as the originals.

/// How near the stage's start and end (in checkpoints) no car is put.
pub const END_MARGIN: u32 = 8;

/// A block's +0x25C when the block is a computer car's (`grade_mode` 2) for `reset_point_blocked`.
pub const IN_STAGE: u32 = 2;

/// Whether a checkpoint is less than `END_MARGIN` from the start (by the sign).
pub fn near_start(checkpoint: u32) -> bool {
    (checkpoint.wrapping_sub(END_MARGIN) as i32) < 0
}

/// The last checkpoint a car may be put at, plus one (`g_checkpointCount` - 8).
pub fn end_of(count: u32) -> u32 {
    count.wrapping_sub(END_MARGIN)
}

/// Whether a checkpoint is at or past `end` (by the sign).
pub fn near_end(checkpoint: u32, end: u32) -> bool {
    (checkpoint.wrapping_sub(end) as i32) >= 0
}

/// `reset_point_blocked`'s answer for one block, with the values the original leaves in
/// its registers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blocked {
    /// The block's car is not in the stage: free (the candidate kept).
    Away,
    /// Near the start: blocked (the candidate left).
    Start,
    /// Near the end: blocked (the candidate left, `end` the other).
    End { end: u32 },
    /// Else: blocked when the gap to the block's checkpoint (+0x94) is in -8 ..= 7 (the gap
    /// left, `end` the other).
    Gap { end: u32, gap: u32, blocked: bool },
}

impl Blocked {
    /// The carry the original returns.
    pub fn carry(&self) -> bool {
        match *self {
            Blocked::Away => false,
            Blocked::Start | Blocked::End { .. } => true,
            Blocked::Gap { blocked, .. } => blocked,
        }
    }
}

/// `physics::reset_point_blocked`: the candidate checked against one block (its +0x25C
/// `state`); `count` and the block's checkpoint asked only where the original reads them.
pub fn reset_point_blocked(
    state: u32,
    candidate: u32,
    count: impl FnOnce() -> u32,
    checkpoint: impl FnOnce() -> u32,
) -> Blocked {
    if state != IN_STAGE {
        return Blocked::Away;
    }
    if near_start(candidate) {
        return Blocked::Start;
    }
    let end = end_of(count());
    if near_end(candidate, end) {
        return Blocked::End { end };
    }
    let gap = candidate.wrapping_sub(checkpoint());
    let blocked =
        (gap.wrapping_sub(END_MARGIN) as i32) < 0 && (gap.wrapping_add(END_MARGIN) as i32) >= 0;
    Blocked::Gap { end, gap, blocked }
}

/// `physics::reset_car`'s tries: the offsets from the car's checkpoint in the order
/// tried, d then -d for d = 0 .. `RESET_TRIES` (0 twice).
pub const RESET_TRIES: u32 = 0x40;

pub fn reset_offsets() -> impl Iterator<Item = u32> {
    (0..RESET_TRIES).flat_map(|d| [d, d.wrapping_neg()])
}

/// `drive::car_near_checkpoint`: for a car in the stage (`in_stage`, its +0x25C not 0),
/// (the distance from its checkpoint +0x98 to `checkpoint`, made positive; whether that is
/// below 4 by the sign); None for one not in it (its checkpoint not asked).
pub fn car_near(
    in_stage: bool,
    car_checkpoint: impl FnOnce() -> u32,
    checkpoint: u32,
) -> Option<(u32, bool)> {
    if !in_stage {
        return None;
    }
    let mut d = car_checkpoint().wrapping_sub(checkpoint);
    if (d as i32) < 0 {
        d = d.wrapping_neg();
    }
    Some((d, (d.wrapping_sub(4) as i32) < 0))
}

/// `drive::checkpoint_taken`: near the start or the end (`count` - 8), or a car near
/// (`any_car_near` asked only past those).
pub fn checkpoint_taken(checkpoint: u32, count: u32, any_car_near: impl FnOnce() -> bool) -> bool {
    near_start(checkpoint) || near_end(checkpoint, end_of(count)) || any_car_near()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked() {
        assert_eq!(
            reset_point_blocked(1, 0, || unreachable!(), || unreachable!()),
            Blocked::Away
        );
        assert_eq!(
            reset_point_blocked(2, 7, || unreachable!(), || unreachable!()),
            Blocked::Start
        );
        assert_eq!(
            reset_point_blocked(2, 92, || 100, || unreachable!()),
            Blocked::End { end: 92 }
        );
        let b = reset_point_blocked(2, 50, || 100, || 57);
        assert_eq!(
            b,
            Blocked::Gap {
                end: 92,
                gap: (-7i32) as u32,
                blocked: true
            }
        );
        assert!(reset_point_blocked(2, 50, || 100, || 58).carry());
        assert!(!reset_point_blocked(2, 50, || 100, || 59).carry());
        assert!(!reset_point_blocked(2, 50, || 100, || 42).carry());
        assert!(reset_point_blocked(2, 50, || 100, || 43).carry());
    }

    #[test]
    fn offsets() {
        let o: Vec<u32> = reset_offsets().take(5).collect();
        assert_eq!(o, [0, 0, 1, u32::MAX, 2]);
        assert_eq!(reset_offsets().count(), 0x80);
    }

    #[test]
    fn near() {
        assert_eq!(car_near(false, || unreachable!(), 3), None);
        assert_eq!(car_near(true, || 10, 13), Some((3, true)));
        assert_eq!(car_near(true, || 17, 13), Some((4, false)));
        assert!(checkpoint_taken(3, 100, || unreachable!()));
        assert!(checkpoint_taken(95, 100, || unreachable!()));
        assert!(!checkpoint_taken(50, 100, || false));
        assert!(checkpoint_taken(50, 100, || true));
    }
}
