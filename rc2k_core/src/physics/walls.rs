//! The car's probe points against the course's walls and edges, and its wheels on the ground
//! (their x87 steps stay with the callers). A face's flags have the wall bits 0x400 and 0x800: a
//! probe point whose face now and before have both between them has met a wall.

use crate::Invalid;
use crate::physics::faces::has_edge;

/// The probe tables' room (0x40 points).
pub const PROBE_ROOM: u32 = 0x40;

/// The wall bits of a face's flags.
pub const WALL_BITS: u32 = 0xC00;

/// No probe point crossed an edge.
pub const NONE_CROSSED: u32 = u32::MAX;

/// A probe point the tables hold.
pub fn check_probe(n: u32) -> Result<(), Invalid> {
    if n >= PROBE_ROOM {
        return Err(Invalid("a probe point past 0x3F (the tables' room)"));
    }
    Ok(())
}

/// The probe count: 1 ..= 0x40.
pub fn check_probe_count(n: u32) -> Result<(), Invalid> {
    if n == 0 || n > PROBE_ROOM {
        return Err(Invalid("a probe count of 0 or past 0x40"));
    }
    Ok(())
}

/// A probe point's faces: the flags word of its face now, and (when its face before differs)
/// that one's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProbeFaces {
    pub now_word: u32,
    pub before_word: Option<u32>,
}

/// How the walk of the probe points ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Walk {
    /// Probe `probe`'s two faces meet as a wall; `words` the words now or-ed so far.
    Met { probe: u32, words: u32 },
    /// None met: the words now or-ed, `& WALL_BITS`.
    Clear { bits: u32 },
}

/// The probe points 0 .. `count` in order (`probe(k)` reads point k's faces), until one's
/// faces have both wall bits between them.
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

/// The wall's edges (p1, p2), (p3, p1), (p4, p3), (p2, p4) looked for in the other face's
/// corners: (whether one is shared, the last edge tried).
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

/// Which side of a probe's path (from `before` to `now`, whole-number x, z) a corner lies:
/// (the first product (now z - before z)(x - before x); the second (now x - before x)
/// (z - before z) less it). Differences are signed 32-bit, products 64-bit.
pub fn path_side(before: (u32, u32), now: (u32, u32), corner: (u32, u32)) -> (i64, i64) {
    let d = |a: u32, b: u32| i64::from(a.wrapping_sub(b) as i32);
    let first = d(now.1, before.1) * d(corner.0, before.0);
    let second = d(now.0, before.0) * d(corner.1, before.1);
    (first, second.wrapping_sub(first))
}

/// The edge the probe crossed, as the face's corner offsets (from, to), by the sides of its
/// corners (`ahead(j)`: corner j not left of the path, asked in a fixed order and only where
/// needed): p1 ahead and p2 not: p1 -> p2; p2 ahead and p4 not: p2 -> p4; p4 ahead and p3 not:
/// p4 -> p3; else p3 -> p1.
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

/// The hardest hit of the step raised to a contact's size: |hit| when not below the kept one.
pub fn raised_hit(hit: f32, kept: f32) -> Option<f32> {
    (hit.abs() >= kept).then_some(hit.abs())
}

/// Which wheels are off the ground (a ground dword of 0 is off).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WheelsOff {
    /// The first two off.
    pub front: bool,
    /// The last two off.
    pub back: bool,
    /// All four off.
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

    /// The driven wheels off (the first two, or all four with the drive kind not 0).
    pub fn driven(self, drive_kind: u32) -> bool {
        if drive_kind != 0 {
            self.all
        } else {
            self.front
        }
    }
}

/// The camera's ground statistics: probes, found, found on the last face again.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProbeCounts {
    /// Probes made.
    pub tries: u32,
    /// Probes that found a face.
    pub found: u32,
    /// Probes that found the last face again.
    pub same: u32,
}

impl ProbeCounts {
    /// One probe counted, before the search.
    pub fn tried(&mut self) {
        self.tries = self.tries.wrapping_add(1);
    }

    /// The search's end: found, and then whether the last face is the one found.
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
mod tests;
