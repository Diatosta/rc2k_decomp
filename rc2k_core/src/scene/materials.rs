//! What an object's drawing takes from its kind's record (converted 2026-10-21 from
//! `scene::set_object_material` 0x45170E and its 0x45079A): the kind's way (+0x20 of its
//! record), and for a kind that moves between two shapes (way 4) the bytes between them
//! and the fraction of the way it has moved, from a clock of four phases.

/// The way of a kind with two shapes.
pub const MOVING: u32 = 4;

/// The phases' lengths (`dword_62DB30` ..): four bases, four random parts, and a 0.
pub const LENGTHS: [u32; 9] = [200, 400, 200, 400, 400, 400, 400, 400, 0];

/// The most time a call adds.
pub const MOST_STEP: u32 = 100;

/// The bytes between an object's two shapes: the next object's first point less this
/// one's, times 12.
pub fn move_by(first_points: [u32; 2]) -> u32 {
    first_points[1]
        .wrapping_sub(first_points[0])
        .wrapping_mul(12)
}

/// A moving object's state (+0x30 .. +0x3C of its record): the clock when last here, the
/// time into the phase, the phase's length, the phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShapeClock {
    pub last: u32,
    pub time: u32,
    pub length: u32,
    pub phase: u32,
}

impl ShapeClock {
    /// 0x45079A at `clock`: the time since last here (at most 100) added; while the time
    /// is not under the length it loses the length, the phase goes on by one of four and
    /// the new length is the phase's base plus `random()`'s top half times its random
    /// part. Returns the fraction: 0 in phase 0, time over length in 1, 1 in 2, 1 less that
    /// in 3 (the x87's `fild; fild; fdivp`, stored as a float).
    pub fn step(&mut self, clock: u32, mut random: impl FnMut() -> u32) -> f32 {
        self.time = self
            .time
            .wrapping_add(clock.wrapping_sub(self.last).min(MOST_STEP));
        while self.time >= self.length {
            self.time -= self.length;
            self.phase = (self.phase + 1) & 3;
            let part = ((random() as u64 * LENGTHS[4 + self.phase as usize] as u64) >> 32) as u32;
            self.length = part.wrapping_add(LENGTHS[self.phase as usize]);
        }
        self.last = clock;
        let ratio = || self.time as i32 as f64 / self.length as i32 as f64;
        match self.phase {
            1 => ratio() as f32,
            2 => 1.0,
            3 => (1.0 - ratio()) as f32,
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_clock() {
        assert_eq!(move_by([0x100, 0x103]), 36);
        let mut c = ShapeClock {
            last: 1000,
            time: 250,
            length: 300,
            phase: 1,
        };
        let f = c.step(1030, || unreachable!());
        assert_eq!((c.time, c.last, f), (280, 1030, (280.0f64 / 300.0) as f32));
        // past the length: phase 2, a length of 200 plus half of 400
        let f = c.step(1200, || 0x8000_0000);
        assert_eq!((c.time, c.length, c.phase, f), (80, 400, 2, 1.0));
        let mut c = ShapeClock {
            last: 1000,
            time: 100,
            length: 300,
            phase: 3,
        };
        let f = c.step(999, || 0);
        // a step back counts as at most 100: 200 of 300 into phase 3
        assert_eq!(
            (c.time, c.phase, f),
            (200, 3, (1.0 - 200.0f64 / 300.0) as f32)
        );
        let mut c = ShapeClock {
            last: 0,
            time: 5000,
            length: 7,
            phase: 1,
        };
        // with no random parts the four phases take 1200 a round
        assert_eq!(c.step(0, || 0), 1.0);
        assert_eq!((c.time, c.length, c.phase), (193, 200, 2));
    }
}
