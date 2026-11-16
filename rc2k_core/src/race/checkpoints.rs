//! The stage's checkpoints: the line's faces chained in order, the checks along them, the map's
//! anchors, the car placed on the line and the nearest checkpoint. Faces are asked about through
//! closures in a fixed order, so nothing is read that is not needed.

use crate::Invalid;

/// The entries the checkpoint-face table and the lists after it have room for.
pub const MOST_CHECKS: u32 = 0x1418;

/// The kind of the line's faces by direction: 0x3E forwards (0), 0x3D backwards (1), else 0.
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
    /// The last face asked about.
    pub last_face: u32,
    /// The place the last search stopped at.
    pub stopped: u32,
}

/// The chain over the line's faces (`list`, two or more): the first face with kind byte 1
/// (`is_start`) goes first (none: length 0), then each next the one the last names (`next_of`),
/// until none is found or all are chained.
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

/// Each checkpoint's count of checks up to it (itself included): faces whose kind is not 0 or 1.
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

/// From checkpoint `from` on, the first below `count` whose face has a map place; None if none.
pub fn next_map_anchor(from: u32, count: u32, mut placed: impl FnMut(u32) -> bool) -> Option<u32> {
    (from..count).find(|&n| placed(n))
}

/// Where `first_check_checkpoint` stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FirstCheck {
    /// The checkpoint found.
    pub found: Option<u32>,
    /// The checkpoint it stopped at.
    pub stopped: u32,
}

/// The first checkpoint from 1 whose face's kind is not 0, looked for up to `last` (excluded).
/// None when `count` - 2 is negative. Refuses a look past the list (`last` 0 or 1).
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

/// The checkpoints `listed_checkpoints` lists at most.
pub const MOST_LISTED: u32 = 0x201;

/// The checkpoints 1 .. `end` whose face is flagged. Refuses an end below 2 and more than
/// `MOST_LISTED` found.
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

/// The checkpoint the car is placed after and the next, each held below `count`. Refuses a
/// line without checkpoints.
pub fn line_neighbours(place: u32, count: u32) -> Result<(u32, u32), Invalid> {
    if count == 0 {
        return Err(Invalid("no checkpoints to place the car between"));
    }
    let c = place.min(count - 1);
    Ok((c, c.wrapping_add(1).min(count - 1)))
}

/// a + (b - a) x t / 65536, with the product's high half.
pub fn lerp(a: u32, b: u32, t: u32) -> (u32, u32) {
    let product = b.wrapping_sub(a) as i32 as i64 * t as i32 as i64;
    (
        ((product >> 16) as u32).wrapping_add(a),
        (product >> 32) as u32,
    )
}

/// The nearest checkpoint so far.
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
    /// Checkpoint `n` at `distance`: kept when as near or nearer (unsigned; the later of equals).
    pub fn consider(&mut self, n: u32, distance: u32) {
        if distance <= self.distance {
            *self = Self {
                checkpoint: n,
                distance,
            };
        }
    }
}

/// Whether `checkpoint` is one of the line's (0 ..= `count` - 1), by signed tests.
pub fn on_line(checkpoint: u32, count: u32) -> bool {
    (checkpoint as i32) >= 0 && (checkpoint.wrapping_sub(count) as i32) < 0
}

/// The nearest's distance over the scale: one rounding to the float (the scale is 65536).
pub fn distance_units(distance: u32, scale: f32) -> f32 {
    (distance as i32 as f64 / scale as f64) as f32
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
        assert!(on_line(0, 5) && on_line(4, 5) && !on_line(5, 5) && !on_line(u32::MAX, 5));
        // By sign: a checkpoint less a count past 0x80000000 is not below 0.
        assert!(!on_line(3, 0x8000_0005) && !on_line(0x8000_0000, 1));
        assert_eq!(distance_units(0x18000, 65536.0), 1.5);
        assert_eq!(distance_units(u32::MAX, 65536.0), -1.0 / 65536.0);
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
