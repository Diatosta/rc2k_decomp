//! The inspection after a stage: which inspected parts of a player's car are at or past
//! their limits, and the verdict (converted 2026-10-21 from `menu::inspection_faults`
//! 0x4A1733 and `menu::inspection_verdict` 0x4A1952). A part's index is its place in the
//! player's +0xD4 .. +0x108 and in the limits and penalties, fourteen each.

use crate::menu::player::entry_kind;

/// The parts inspected, by index; index 4 is not looked at.
pub const INSPECTED: [usize; 13] = [0, 1, 2, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13];

/// `menu::inspection_faults`: the thirteen fault flags, 1 for each inspected part (in
/// order) past its limit (`past` by index).
pub fn faults(past: impl Fn(usize) -> bool) -> [u32; 13] {
    INSPECTED.map(|k| past(k) as u32)
}

/// The verdict: the penalties summed, the score and the player's entry kind (+0xD0).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Verdict {
    pub penalties: u32,
    pub score: u32,
    pub kind: u32,
}

/// `menu::inspection_verdict`: the penalties (`penalty` by index) of the inspected parts
/// past their limits (`past`) that are not waived (`waived`, asked only for those) added;
/// the score 100 less that; the kind by it (`player::entry_kind`).
pub fn verdict(
    past: impl Fn(usize) -> bool,
    waived: impl Fn(usize) -> bool,
    penalty: impl Fn(usize) -> u32,
) -> Verdict {
    let penalties = INSPECTED
        .iter()
        .filter(|&&k| past(k) && !waived(k))
        .fold(0u32, |sum, &k| sum.wrapping_add(penalty(k)));
    let score = 100u32.wrapping_sub(penalties);
    Verdict {
        penalties,
        score,
        kind: entry_kind(score as i32),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspection() {
        let f = faults(|k| k == 4 || k == 13);
        assert_eq!(f[12], 1);
        assert_eq!(f.iter().sum::<u32>(), 1);
        let v = verdict(|k| k < 3, |k| k == 1, |k| 10 * (k as u32 + 1));
        assert_eq!(
            v,
            Verdict {
                penalties: 40,
                score: 60,
                kind: 2
            }
        );
        assert_eq!(verdict(|_| false, |_| false, |_| 0).kind, 1);
    }
}
