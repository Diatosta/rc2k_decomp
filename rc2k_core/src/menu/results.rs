//! The players' results (converted 2026-10-21 from `menu::rank_stage_times` 0x48340B,
//! `menu::rank_totals` 0x48356F, `menu::score_four_players` 0x485CBB and
//! `menu::score_two_players` 0x485B0E): the four players ranked by their stage times or
//! totals, and each one's points, time bonus and new total.

/// The four players' (byte offset 0, 4, 8, 12, value) pairs sorted by the original's
/// exchange loop: each pair i < j exchanged when j's value is at most i's (times) or at
/// least i's (totals), signed.
pub fn rank(values: [u32; 4], by_totals: bool) -> [(u32, u32); 4] {
    let mut pairs = [0, 1, 2, 3].map(|k| (4 * k as u32, values[k]));
    for i in 0..3 {
        for j in i + 1..4 {
            let (vi, vj) = (pairs[i].1 as i32, pairs[j].1 as i32);
            if if by_totals { vj >= vi } else { vj <= vi } {
                pairs.swap(i, j);
            }
        }
    }
    pairs
}

/// The time bonus: the stage's time less the player's, 0 when the player's is greater
/// (signed).
pub fn bonus(time: u32, target: u32) -> u32 {
    if time as i32 > target as i32 {
        0
    } else {
        target.wrapping_sub(time)
    }
}

/// Two players' places: 1 and 2, or 2 and 1 when the second's time is below the first's
/// (signed).
pub fn two_places(times: [u32; 2]) -> [u32; 2] {
    if (times[1] as i32) < times[0] as i32 {
        [2, 1]
    } else {
        [1, 2]
    }
}

/// A player's score.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Score {
    pub before: u32,
    pub points: u32,
    pub bonus: u32,
    pub extra: u32,
    pub sum: u32,
    pub total: u32,
}

/// A player's score from the total before, the place's points, the time and the stage's,
/// and the player's +0x94.
pub fn score(before: u32, points: u32, time: u32, target: u32, extra: u32) -> Score {
    let bonus = bonus(time, target);
    let sum = points.wrapping_add(bonus).wrapping_add(extra);
    Score {
        before,
        points,
        bonus,
        extra,
        sum,
        total: before.wrapping_add(sum),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn results() {
        assert_eq!(
            rank([30, 10, 20, 10], false),
            [(12, 10), (4, 10), (8, 20), (0, 30)]
        );
        assert_eq!(rank([1, 5, 3, 5], true), [(12, 5), (4, 5), (8, 3), (0, 1)]);
        assert_eq!((bonus(100, 90), bonus(80, 90)), (0, 10));
        assert_eq!((two_places([5, 4]), two_places([5, 5])), ([2, 1], [1, 2]));
        assert_eq!(score(10, 5, 80, 90, 2).total, 27);
    }
}
