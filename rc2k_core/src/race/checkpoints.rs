//! The stage's checkpoints: the line's faces chained in order, the checks along them and
//! the map's anchors, the car placed on the line (converted 2026-10-21 from
//! `race::set_line_face_kind` 0x44574A, `sort_checkpoints` 0x4458A7, `count_checks`
//! 0x445AAB, `next_map_anchor` 0x445FEE, `first_check_checkpoint` 0x446280,
//! `init_checkpoints` 0x45A98F, `place_car_on_line` 0x44666F and `nearest_checkpoint_in`
//! 0x4413F8). A face is asked about through closures, in the original's order, so nothing
//! is read that it does not read.

use crate::Invalid;

/// The entries `g_checkpointFaces` and the lists after it have room for.
pub const MOST_CHECKS: u32 = 0x1418;

/// `race::set_line_face_kind`: the kind of the line's faces by the stage's direction
/// (`g_stageDirection`): 0x3E forwards (0), 0x3D backwards (1), else 0.
pub fn line_face_kind(direction: u32) -> u32 {
    match direction {
        0 => 0x3E,
        1 => 0x3D,
        _ => 0,
    }
}

/// Where `chain` stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chain {
    /// The checkpoints chained (0: no start face).
    pub length: u32,
    /// The last face asked about (ebx in the original).
    pub last_face: u32,
    /// The place the last search stopped at (edx).
    pub stopped: u32,
}

/// `race::sort_checkpoints`' chain over the line's faces (`list`, two or more): the
/// first whose byte +0x26 is 1 (`is_start`) put first (none: length 0), then each next
/// the one the last one names (`next_of`, its +0xC; searched over the whole list,
/// swapped into place) until none is found or all are chained.
pub fn chain(
    list: &mut [u32],
    mut is_start: impl FnMut(u32) -> bool,
    mut next_of: impl FnMut(u32) -> u32,
) -> Chain {
    let found = list.len() as u32;
    let mut stopped = 0;
    let mut last_face;
    loop {
        last_face = list[stopped as usize];
        if is_start(last_face) {
            break;
        }
        stopped += 1;
        if stopped == found {
            return Chain {
                length: 0,
                last_face,
                stopped,
            };
        }
    }
    list.swap(stopped as usize, 0);
    let mut length = 1u32;
    while length != found {
        last_face = list[length as usize - 1];
        let next = next_of(last_face);
        match list.iter().position(|&f| f == next) {
            Some(n) => {
                stopped = n as u32;
                list.swap(length as usize, n);
                length += 1;
            }
            None => {
                stopped = found;
                break;
            }
        }
    }
    Chain {
        length,
        last_face,
        stopped,
    }
}

/// `race::count_checks`: each checkpoint's count of checks up to it (itself included):
/// the checkpoints whose face's byte +0x26 (`kind`, asked in order) is neither 0 nor 1.
pub fn check_counts(count: u32, mut kind: impl FnMut(u32) -> u8) -> Vec<u32> {
    let mut checks = 0u32;
    (0..count)
        .map(|n| {
            if !matches!(kind(n), 0 | 1) {
                checks += 1;
            }
            checks
        })
        .collect()
}

/// `race::next_map_anchor`: from checkpoint `from` on, the first below `count` whose
/// face has a place on the map (`placed`, asked in order); None for none.
pub fn next_map_anchor(from: u32, count: u32, mut placed: impl FnMut(u32) -> bool) -> Option<u32> {
    (from..count).find(|&n| placed(n))
}

/// Where `first_check_checkpoint` stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FirstCheck {
    /// The checkpoint found.
    pub found: Option<u32>,
    /// The checkpoint it stopped at (edx).
    pub stopped: u32,
}

/// `race::first_check_checkpoint`: the first checkpoint from 1 whose face's byte +0x26
/// (`kind`, asked in order) is not 0, looked for up to `last` (not included, asked only
/// for a look). None (no look) when `count` - 2 is negative. Refuses a look that would
/// run past the list (the original runs on until it finds one: `last` 0 or 1).
pub fn first_check_checkpoint(
    count: u32,
    last: impl FnOnce() -> u32,
    mut kind: impl FnMut(u32) -> u8,
) -> Result<Option<FirstCheck>, Invalid> {
    if (count.wrapping_sub(2) as i32) < 0 {
        return Ok(None);
    }
    let last = last();
    let mut n = 1u32;
    loop {
        if n >= MOST_CHECKS {
            return Err(Invalid("a look for a check past the checkpoint list"));
        }
        if kind(n) != 0 {
            return Ok(Some(FirstCheck {
                found: Some(n),
                stopped: n,
            }));
        }
        n += 1;
        if n == last {
            return Ok(Some(FirstCheck {
                found: None,
                stopped: n,
            }));
        }
    }
}

/// The checkpoints `init_checkpoints` lists at most (`dword_60102C`).
pub const MOST_LISTED: u32 = 0x201;

/// `race::init_checkpoints`: the checkpoints 1 .. `end` (not included) whose face's
/// byte +0x26 is set (`flagged`, asked in order). Refuses an end below 2 (the original
/// counts round) and more than `MOST_LISTED` found.
pub fn listed_checkpoints(
    end: u32,
    mut flagged: impl FnMut(u32) -> bool,
) -> Result<Vec<u32>, Invalid> {
    if end < 2 {
        return Err(Invalid(
            "a checkpoint end below 2: the original counts round",
        ));
    }
    let listed: Vec<u32> = (1..end).filter(|&n| flagged(n)).collect();
    if listed.len() as u32 > MOST_LISTED {
        return Err(Invalid("more checkpoints found than the list holds"));
    }
    Ok(listed)
}

/// `race::place_car_on_line`: the checkpoint the car is placed after (`dword_6365E4`)
/// and the next, each held below `count`. Refuses no checkpoints (the original reads
/// entry -1).
pub fn line_neighbours(place: u32, count: u32) -> Result<(u32, u32), Invalid> {
    if count == 0 {
        return Err(Invalid("no checkpoints to place the car between"));
    }
    let c = place.min(count - 1);
    Ok((c, c.wrapping_add(1).min(count - 1)))
}

/// `race::place_car_on_line`: a + (b - a) x t / 65536 as `imul` / `shrd 16` / `add` make
/// it, and the product's high half (edx).
pub fn lerp(a: u32, b: u32, t: u32) -> (u32, u32) {
    let product = b.wrapping_sub(a) as i32 as i64 * t as i32 as i64;
    (
        ((product >> 16) as u32).wrapping_add(a),
        (product >> 32) as u32,
    )
}

/// The nearest checkpoint so far (`race::nearest_checkpoint_in`: ebp, edi).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Nearest {
    pub checkpoint: u32,
    pub distance: u32,
}

impl Default for Nearest {
    /// None yet: checkpoint 0 at -1.
    fn default() -> Self {
        Self {
            checkpoint: 0,
            distance: u32::MAX,
        }
    }
}

impl Nearest {
    /// Checkpoint `n` at `distance`: kept when as near or nearer (unsigned: the later of
    /// equals).
    pub fn consider(&mut self, n: u32, distance: u32) {
        if distance <= self.distance {
            *self = Self {
                checkpoint: n,
                distance,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_and_counts() {
        assert_eq!([0, 1, 2].map(line_face_kind), [0x3E, 0x3D, 0]);
        let kinds = [1u8, 0, 5, 1, 2];
        assert_eq!(check_counts(5, |n| kinds[n as usize]), [0, 0, 1, 1, 2]);
        assert_eq!(next_map_anchor(1, 5, |n| n == 3), Some(3));
        assert_eq!(next_map_anchor(4, 5, |n| n == 3), None);
    }

    #[test]
    fn chains() {
        // Faces 10 .. 13: 12 starts, then 12 -> 10 -> 13 -> 11.
        let next = |f: u32| match f {
            12 => 10,
            10 => 13,
            13 => 11,
            _ => 99,
        };
        let mut list = [10, 11, 12, 13];
        let c = chain(&mut list, |f| f == 12, next);
        assert_eq!(list, [12, 10, 13, 11]);
        assert_eq!(
            c,
            Chain {
                length: 4,
                last_face: 13,
                stopped: 3
            }
        );
        // A break in the chain.
        let mut list = [10, 11, 12];
        let c = chain(&mut list, |f| f == 11, |_| 50);
        assert_eq!(
            (list, c.length, c.last_face, c.stopped),
            ([11, 10, 12], 1, 11, 3)
        );
        // No start.
        let mut list = [10, 11];
        let c = chain(&mut list, |_| false, next);
        assert_eq!((c.length, c.last_face, c.stopped), (0, 11, 2));
    }

    #[test]
    fn first_check() {
        assert_eq!(
            first_check_checkpoint(1, || unreachable!(), |_| 1),
            Ok(None)
        );
        let kinds = [9u8, 0, 0, 4, 0];
        assert_eq!(
            first_check_checkpoint(5, || 5, |n| kinds[n as usize]),
            Ok(Some(FirstCheck {
                found: Some(3),
                stopped: 3
            }))
        );
        assert_eq!(
            first_check_checkpoint(5, || 3, |n| kinds[n as usize]),
            Ok(Some(FirstCheck {
                found: None,
                stopped: 3
            }))
        );
        assert!(first_check_checkpoint(5, || 1, |_| 0).is_err());
        assert_eq!(listed_checkpoints(6, |n| n % 2 == 1), Ok(vec![1, 3, 5]));
        assert!(listed_checkpoints(1, |_| true).is_err());
        assert!(listed_checkpoints(0x203, |_| true).is_err());
        assert_eq!(
            listed_checkpoints(0x202, |_| true).map(|l| l.len()),
            Ok(0x201)
        );
    }

    #[test]
    fn placing() {
        assert!(line_neighbours(0, 0).is_err());
        assert_eq!(line_neighbours(3, 10), Ok((3, 4)));
        assert_eq!(line_neighbours(9, 10), Ok((9, 9)));
        assert_eq!(line_neighbours(u32::MAX, 10), Ok((9, 9)));
        assert_eq!(lerp(10, 20, 0x8000), (15, 0));
        assert_eq!(lerp(20, 10, 0x8000), (15, u32::MAX));
        let mut n = Nearest::default();
        n.consider(0, 9);
        n.consider(1, 5);
        n.consider(2, 5);
        n.consider(3, 7);
        assert_eq!(
            n,
            Nearest {
                checkpoint: 2,
                distance: 5
            }
        );
    }
}
