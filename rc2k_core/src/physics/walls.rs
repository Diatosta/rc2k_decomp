//! The car's probe points against the course's walls and edges, and its wheels on the
//! ground (converted from `physics::wall_contacts` 0x45D7B6,
//! `wall_edge_contact` 0x45DA14, `crossed_edge_contact` 0x45DC3E, `crossed_edge`
//! 0x45D737, `feel_ground` 0x43F1A8 and `count_ground_probe` 0x438E22; their x87 steps,
//! the edge's heading and the contact's force, stay the hooks'). A face's word +2 has
//! the wall bits 0x400 and 0x800: a probe point whose face now and before have both
//! between them has met a wall.

use crate::Invalid;
use crate::physics::faces::has_edge;

/// The probe tables' room (0x40 points: `probe_point_x` .., `probe_x` ..).
pub const PROBE_ROOM: u32 = 0x40;

/// The wall bits of a face's word +2.
pub const WALL_BITS: u32 = 0xC00;

/// No probe point crossed an edge (`crossed_probe`).
pub const NONE_CROSSED: u32 = u32::MAX;

/// A probe point the tables hold.
pub fn check_probe(n: u32) -> Result<(), Invalid> {
    if n >= PROBE_ROOM {
        return Err(Invalid("a probe point past 0x3F (the tables' room)"));
    }
    Ok(())
}

/// `physics::wall_contacts`' probe count (`probe_count`): 1 ..= 0x40 (the original's loop
/// runs on past 0 and reads past the tables beyond 0x40).
pub fn check_probe_count(n: u32) -> Result<(), Invalid> {
    if n == 0 || n > PROBE_ROOM {
        return Err(Invalid("a probe count of 0 or past 0x40"));
    }
    Ok(())
}

/// A probe point's faces as `wall_contacts` reads them: the word of its face now, and
/// (when its face before differs) that one's word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProbeFaces {
    pub now_word: u32,
    pub before_word: Option<u32>,
}

/// How the walk of the probe points ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Walk {
    /// Probe `probe`'s two faces meet as a wall; `words` the faces' words now or-ed so far.
    Met { probe: u32, words: u32 },
    /// None met: the words now or-ed, `& WALL_BITS` (the car pushed back at `WALL_BITS`).
    Clear { bits: u32 },
}

/// `physics::wall_contacts`' walk: the probe points 0 .. `count` in order (`probe(k)`
/// reads point k's faces), until one's faces have both wall bits between them.
pub fn walk<E>(count: u32, mut probe: impl FnMut(u32) -> Result<ProbeFaces, E>) -> Result<Walk, E> {
    let mut words = 0;
    for k in 0..count {
        let faces = probe(k)?;
        words |= faces.now_word;
        if let Some(before) = faces.before_word
            && (faces.now_word | before) & WALL_BITS == WALL_BITS
        {
            return Ok(Walk::Met { probe: k, words });
        }
    }
    Ok(Walk::Clear {
        bits: words & WALL_BITS,
    })
}

/// `physics::wall_edge_contact`: the wall's edges (its corners +4 .. +0x10, `wall(0 ..
/// 4)`, read as the original reads them) (p1, p2), (p3, p1), (p4, p3), (p2, p4) looked
/// for in the other face's corners (`face_has_edge`): (whether one is shared, the last
/// edge tried (its ends)).
pub fn shared_edge(wall: impl Fn(usize) -> u32, other: [u32; 4]) -> (bool, u32, u32) {
    let mut last = (0, 0);
    for (a, b) in [(0, 1), (2, 0), (3, 2), (1, 3)] {
        last = (wall(a), wall(b));
        if has_edge(last.0, last.1, other) {
            return (true, last.0, last.1);
        }
    }
    (false, last.0, last.1)
}

/// `physics::crossed_edge_contact`: which side of a probe's path (from `before` to `now`,
/// whole-number x, z) a corner lies: (the first product (now z - before z)(x - before x),
/// which the original keeps in `product`; the second (now x - before x)(z - before
/// z) less it). Each difference a 32-bit `sub` taken as signed, each product 64 bits.
pub fn path_side(before: (u32, u32), now: (u32, u32), corner: (u32, u32)) -> (i64, i64) {
    let d = |a: u32, b: u32| i64::from(a.wrapping_sub(b) as i32);
    let first = d(now.1, before.1) * d(corner.0, before.0);
    let second = d(now.0, before.0) * d(corner.1, before.1);
    (first, second.wrapping_sub(first))
}

/// `physics::crossed_edge_contact`: the edge the probe crossed, as the face's corner
/// offsets (from, to), by the sides of its corners p1, p2, p3, p4 (`ahead(j)`: corner j
/// not left of the path, asked in the original's order and only where it asks): p1 ahead
/// and p2 not: p1 -> p2; p2 ahead and p4 not: p2 -> p4; p4 ahead and p3 not: p4 -> p3;
/// else p3 -> p1 (p3 asked once more, and p1 after it when p3 is ahead).
pub fn crossed_edge(mut ahead: impl FnMut(usize) -> bool) -> (u32, u32) {
    if ahead(0) && !ahead(1) {
        (4, 8)
    } else if ahead(1) && !ahead(3) {
        (8, 0x10)
    } else if ahead(3) && !ahead(2) {
        (0x10, 0xC)
    } else {
        if ahead(2) {
            ahead(0);
        }
        (0xC, 4)
    }
}

/// The hardest hit of the step (`contact_depth`) raised to a contact's size (`contact_speed_length`):
/// |hit| when it is not below the kept one (`fabs`, `fcomp`; unordered: kept).
pub fn raised_hit(hit: f32, kept: f32) -> Option<f32> {
    (hit.abs() >= kept).then_some(hit.abs())
}

/// `physics::feel_ground`: which wheels are off the ground, from the four wheels'
/// ground dwords (0 off).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WheelsOff {
    /// `front_wheels_off`: the first two off.
    pub front: bool,
    /// `rear_wheels_off`: the last two off.
    pub back: bool,
    /// `all_wheels_off`: all four off.
    pub all: bool,
}

impl WheelsOff {
    pub fn of(wheels: [u32; 4]) -> Self {
        let front = wheels[0] | wheels[1];
        let back = wheels[2] | wheels[3];
        Self {
            front: front == 0,
            back: back == 0,
            all: front | back == 0,
        }
    }

    /// `driven_wheels_off`: the driven wheels off (the first two, or all four with the drive
    /// kind not 0).
    pub fn driven(self, drive_kind: u32) -> bool {
        if drive_kind != 0 {
            self.all
        } else {
            self.front
        }
    }
}

/// The camera's ground statistics (`physics::count_ground_probe`): probes
/// (`ground_probes`), found (`ground_found`), found on the last face again (`ground_same`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProbeCounts {
    pub tries: u32,
    pub found: u32,
    pub same: u32,
}

impl ProbeCounts {
    /// One probe counted, before the search.
    pub fn tried(&mut self) {
        self.tries = self.tries.wrapping_add(1);
    }

    /// The search's end: found, and then whether the last face (`ground_face_last`) is the
    /// one found (`ground_slot`).
    pub fn searched(&mut self, found: bool, last: u32, now: u32) {
        if found {
            self.found = self.found.wrapping_add(1);
            if last == now {
                self.same = self.same.wrapping_add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walks() {
        let faces = [(0x400, None), (0x400, Some(0x800)), (0, None)];
        let w = walk::<()>(3, |k| {
            let (now_word, before_word) = faces[k as usize];
            Ok(ProbeFaces {
                now_word,
                before_word,
            })
        });
        assert_eq!(
            w,
            Ok(Walk::Met {
                probe: 1,
                words: 0x400
            })
        );
        let w = walk::<()>(2, |_| {
            Ok(ProbeFaces {
                now_word: 0xC01,
                before_word: Some(0),
            })
        });
        assert_eq!(
            w,
            Ok(Walk::Met {
                probe: 0,
                words: 0xC01
            })
        );
        let w = walk::<()>(2, |k| {
            Ok(ProbeFaces {
                now_word: [0x401, 0x802][k as usize],
                before_word: None,
            })
        });
        assert_eq!(w, Ok(Walk::Clear { bits: 0xC00 }));
        assert!(check_probe_count(0).is_err() && check_probe_count(0x41).is_err());
        assert!(check_probe(0x3F).is_ok() && check_probe(0x40).is_err());
        assert_eq!(raised_hit(-3.0, 2.0), Some(3.0));
        assert_eq!(raised_hit(2.0, 2.0), Some(2.0));
        assert_eq!(raised_hit(1.0, 2.0), None);
        assert_eq!(raised_hit(f32::NAN, 2.0), None);
        assert_eq!(raised_hit(1.0, f32::NAN), None);
    }

    #[test]
    fn edges() {
        let wall = |j: usize| [1, 2, 3, 4][j];
        assert_eq!(shared_edge(wall, [9, 9, 9, 9]), (false, 2, 4));
        // (p3, p1) = (3, 1) is the other face's (p2, p1).
        assert_eq!(shared_edge(wall, [1, 3, 8, 9]), (true, 3, 1));
        assert_eq!(path_side((0, 0), (10, 0), (3, 2)), (0, 20));
        let mut asked = Vec::new();
        let e = crossed_edge(|j| {
            asked.push(j);
            j == 2
        });
        assert_eq!(e, (0xC, 4));
        assert_eq!(asked, [0, 1, 3, 2, 0]);
        assert_eq!(crossed_edge(|j| j == 0), (4, 8));
        assert_eq!(crossed_edge(|j| j == 1), (8, 0x10));
    }

    #[test]
    fn wheels() {
        let w = WheelsOff::of([0, 0, 1, 0]);
        assert!(w.front && !w.back && !w.all);
        assert!(w.driven(0) && !w.driven(1));
        let mut c = ProbeCounts::default();
        c.tried();
        c.searched(true, 5, 5);
        c.searched(false, 5, 5);
        assert_eq!(
            c,
            ProbeCounts {
                tries: 1,
                found: 1,
                same: 1
            }
        );
    }
}
