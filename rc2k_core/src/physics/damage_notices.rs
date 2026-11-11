//! The work car's damage warnings (from `physics::damage_notices`
//! 0x4B0241): each damage gauge (a float of the work copy, `brake_heat` .. `tyre_wear`)
//! moves a level (`damage_levels` .. `unk_71A3C8`) by whole-number thresholds, in the
//! original's order of 82 steps (read from the disassembly): a rising step, the gauge not
//! below its threshold, sets the level and queues a notice; a falling one (the hysteresis
//! of some gauges), below it, sets the level back.
//!
//! Floats: each threshold is a whole number made a float (`fild` / `fstp`, exact) and
//! compared with the gauge (`fcomp`, unordered counting as below): exact at any
//! precision.

use super::x87::below;

/// The levels (dwords from `damage_levels`) and the gauges (dwords from `brake_heat`).
pub const LEVELS: usize = 35;
pub const GAUGES: usize = 47;

/// One step: a level, the level it applies at, the threshold, the gauge, the new level;
/// a rising one also the notice and its rank.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Up {
        level: usize,
        from: u32,
        limit: i32,
        gauge: usize,
        to: u32,
        notice: u32,
        rank: u32,
    },
    Down {
        level: usize,
        from: u32,
        limit: i32,
        gauge: usize,
        to: u32,
    },
}
use Step::{Down, Up};

/// The original's steps, in order.
#[rustfmt::skip]
pub const STEPS: [Step; 82] = [
    Up { level: 22, from: 0, limit: 1, gauge: 2, to: 1, notice: 0xCF, rank: 1 },
    Down { level: 22, from: 1, limit: 1, gauge: 2, to: 0 },
    Up { level: 22, from: 1, limit: 50, gauge: 2, to: 2, notice: 0xCF, rank: 2 },
    Down { level: 22, from: 2, limit: 50, gauge: 2, to: 1 },
    Up { level: 22, from: 2, limit: 75, gauge: 2, to: 3, notice: 0xD0, rank: 3 },
    Down { level: 22, from: 3, limit: 75, gauge: 2, to: 2 },
    Up { level: 23, from: 0, limit: 75, gauge: 0, to: 1, notice: 0xD1, rank: 1 },
    Down { level: 23, from: 1, limit: 75, gauge: 0, to: 0 },
    Up { level: 24, from: 0, limit: 75, gauge: 1, to: 1, notice: 0xD1, rank: 1 },
    Down { level: 24, from: 1, limit: 75, gauge: 1, to: 0 },
    Up { level: 0, from: 0, limit: 50, gauge: 3, to: 1, notice: 0xD2, rank: 1 },
    Up { level: 0, from: 1, limit: 75, gauge: 3, to: 2, notice: 0xD2, rank: 2 },
    Up { level: 0, from: 2, limit: 90, gauge: 3, to: 3, notice: 0xD3, rank: 3 },
    Up { level: 0, from: 3, limit: 100, gauge: 3, to: 4, notice: 0xD4, rank: 4 },
    Up { level: 1, from: 0, limit: 50, gauge: 4, to: 1, notice: 0xD5, rank: 1 },
    Up { level: 1, from: 1, limit: 75, gauge: 4, to: 2, notice: 0xD5, rank: 2 },
    Up { level: 1, from: 2, limit: 90, gauge: 4, to: 3, notice: 0xD6, rank: 3 },
    Up { level: 1, from: 3, limit: 100, gauge: 4, to: 4, notice: 0xD7, rank: 4 },
    Up { level: 2, from: 0, limit: 50, gauge: 5, to: 1, notice: 0xD8, rank: 1 },
    Up { level: 2, from: 1, limit: 75, gauge: 5, to: 2, notice: 0xD8, rank: 2 },
    Up { level: 2, from: 2, limit: 90, gauge: 5, to: 3, notice: 0xD9, rank: 3 },
    Up { level: 2, from: 3, limit: 100, gauge: 5, to: 4, notice: 0xDA, rank: 4 },
    Up { level: 3, from: 0, limit: 50, gauge: 6, to: 1, notice: 0xDB, rank: 1 },
    Up { level: 3, from: 1, limit: 75, gauge: 6, to: 2, notice: 0xDB, rank: 2 },
    Up { level: 3, from: 2, limit: 90, gauge: 6, to: 3, notice: 0xDC, rank: 3 },
    Up { level: 3, from: 3, limit: 100, gauge: 6, to: 4, notice: 0xDD, rank: 4 },
    Up { level: 4, from: 0, limit: 50, gauge: 7, to: 1, notice: 0xDE, rank: 1 },
    Up { level: 4, from: 1, limit: 75, gauge: 7, to: 2, notice: 0xDE, rank: 2 },
    Up { level: 4, from: 2, limit: 90, gauge: 7, to: 3, notice: 0xDF, rank: 3 },
    Up { level: 4, from: 3, limit: 100, gauge: 7, to: 4, notice: 0xE0, rank: 4 },
    Up { level: 5, from: 0, limit: 50, gauge: 8, to: 1, notice: 0xE1, rank: 1 },
    Up { level: 5, from: 1, limit: 75, gauge: 8, to: 2, notice: 0xE1, rank: 2 },
    Up { level: 5, from: 2, limit: 90, gauge: 8, to: 3, notice: 0xE2, rank: 3 },
    Up { level: 5, from: 3, limit: 100, gauge: 8, to: 4, notice: 0xE3, rank: 4 },
    Up { level: 6, from: 0, limit: 50, gauge: 18, to: 1, notice: 0xE4, rank: 1 },
    Up { level: 6, from: 1, limit: 75, gauge: 18, to: 2, notice: 0xE4, rank: 2 },
    Up { level: 6, from: 2, limit: 90, gauge: 18, to: 3, notice: 0xE5, rank: 3 },
    Up { level: 6, from: 3, limit: 100, gauge: 18, to: 4, notice: 0xE6, rank: 4 },
    Up { level: 7, from: 0, limit: 50, gauge: 24, to: 1, notice: 0xE7, rank: 1 },
    Up { level: 7, from: 1, limit: 75, gauge: 24, to: 2, notice: 0xE7, rank: 2 },
    Up { level: 7, from: 2, limit: 90, gauge: 24, to: 3, notice: 0xE8, rank: 3 },
    Up { level: 7, from: 3, limit: 100, gauge: 24, to: 4, notice: 0xE9, rank: 4 },
    Up { level: 8, from: 0, limit: 50, gauge: 25, to: 1, notice: 0xEA, rank: 1 },
    Up { level: 8, from: 1, limit: 75, gauge: 25, to: 2, notice: 0xEA, rank: 2 },
    Up { level: 8, from: 2, limit: 90, gauge: 25, to: 3, notice: 0xEB, rank: 3 },
    Up { level: 8, from: 3, limit: 100, gauge: 25, to: 4, notice: 0xEC, rank: 4 },
    Up { level: 9, from: 0, limit: 50, gauge: 33, to: 1, notice: 0xED, rank: 1 },
    Up { level: 9, from: 1, limit: 75, gauge: 33, to: 2, notice: 0xED, rank: 2 },
    Up { level: 9, from: 2, limit: 90, gauge: 33, to: 3, notice: 0xEE, rank: 3 },
    Up { level: 9, from: 3, limit: 100, gauge: 33, to: 4, notice: 0xEF, rank: 4 },
    Up { level: 10, from: 0, limit: 50, gauge: 41, to: 1, notice: 0xF0, rank: 1 },
    Up { level: 10, from: 1, limit: 75, gauge: 41, to: 2, notice: 0xF0, rank: 2 },
    Up { level: 10, from: 2, limit: 90, gauge: 41, to: 3, notice: 0xF1, rank: 3 },
    Up { level: 10, from: 3, limit: 100, gauge: 41, to: 4, notice: 0xF2, rank: 4 },
    Up { level: 13, from: 0, limit: 50, gauge: 38, to: 1, notice: 0xF3, rank: 1 },
    Up { level: 13, from: 1, limit: 75, gauge: 38, to: 2, notice: 0xF3, rank: 2 },
    Up { level: 13, from: 2, limit: 90, gauge: 38, to: 3, notice: 0xF4, rank: 3 },
    Up { level: 13, from: 3, limit: 100, gauge: 38, to: 4, notice: 0xF5, rank: 4 },
    Up { level: 11, from: 0, limit: 50, gauge: 32, to: 1, notice: 0xF6, rank: 1 },
    Up { level: 11, from: 1, limit: 75, gauge: 32, to: 2, notice: 0xF6, rank: 2 },
    Up { level: 11, from: 2, limit: 90, gauge: 32, to: 3, notice: 0xF7, rank: 3 },
    Up { level: 25, from: 0, limit: 100, gauge: 26, to: 1, notice: 0xF8, rank: 1 },
    Up { level: 26, from: 0, limit: 100, gauge: 27, to: 1, notice: 0xF9, rank: 1 },
    Up { level: 27, from: 0, limit: 100, gauge: 28, to: 1, notice: 0xFA, rank: 1 },
    Up { level: 28, from: 0, limit: 100, gauge: 29, to: 1, notice: 0xFB, rank: 1 },
    Up { level: 29, from: 0, limit: 100, gauge: 30, to: 1, notice: 0xFC, rank: 1 },
    Up { level: 30, from: 0, limit: 100, gauge: 31, to: 1, notice: 0xFD, rank: 1 },
    Up { level: 31, from: 0, limit: 101, gauge: 19, to: 1, notice: 0xFE, rank: 1 },
    Up { level: 32, from: 0, limit: 101, gauge: 20, to: 1, notice: 0xFF, rank: 1 },
    Up { level: 33, from: 0, limit: 101, gauge: 21, to: 1, notice: 0x100, rank: 1 },
    Up { level: 34, from: 0, limit: 101, gauge: 22, to: 1, notice: 0x101, rank: 1 },
    Up { level: 12, from: 0, limit: 50, gauge: 46, to: 1, notice: 0x102, rank: 1 },
    Up { level: 12, from: 1, limit: 75, gauge: 46, to: 2, notice: 0x102, rank: 2 },
    Up { level: 12, from: 2, limit: 90, gauge: 46, to: 3, notice: 0x103, rank: 3 },
    Up { level: 14, from: 0, limit: 100, gauge: 9, to: 1, notice: 0x104, rank: 1 },
    Up { level: 15, from: 0, limit: 100, gauge: 11, to: 1, notice: 0x105, rank: 1 },
    Up { level: 16, from: 0, limit: 100, gauge: 12, to: 1, notice: 0x106, rank: 1 },
    Up { level: 17, from: 0, limit: 100, gauge: 13, to: 1, notice: 0x107, rank: 1 },
    Up { level: 18, from: 0, limit: 100, gauge: 14, to: 1, notice: 0x108, rank: 1 },
    Up { level: 19, from: 0, limit: 100, gauge: 15, to: 1, notice: 0x109, rank: 1 },
    Up { level: 20, from: 0, limit: 100, gauge: 16, to: 1, notice: 0x10A, rank: 1 },
    Up { level: 21, from: 0, limit: 100, gauge: 17, to: 1, notice: 0x10B, rank: 1 },
];

/// A notice queued: its id, the gauge value and its rank.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Queued {
    pub notice: u32,
    pub gauge: usize,
    pub rank: u32,
}

/// The steps on the levels: `gauge(k)` read where a step applies, `limit(t)` told of each
/// threshold stored, `queue` each notice (its refusal stops the steps).
pub fn damage_notices<E>(
    levels: &mut [u32; LEVELS],
    gauge: impl Fn(usize) -> f32,
    mut limit: impl FnMut(i32),
    mut queue: impl FnMut(Queued) -> Result<(), E>,
) -> Result<(), E> {
    for step in STEPS {
        let (level, from, threshold, k) = match step {
            Up {
                level,
                from,
                limit,
                gauge,
                ..
            }
            | Down {
                level,
                from,
                limit,
                gauge,
                ..
            } => (level, from, limit, gauge),
        };
        if levels[level] != from {
            continue;
        }
        limit(threshold);
        let is_below = below(gauge(k) as f64, threshold as f32 as f64);
        match step {
            Up {
                to, notice, rank, ..
            } if !is_below => {
                levels[level] = to;
                queue(Queued {
                    notice,
                    gauge: k,
                    rank,
                })?;
            }
            Down { to, .. } if is_below => levels[level] = to,
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notices() {
        let mut levels = [0u32; LEVELS];
        let mut queued = vec![];
        // Gauge 2 (the first steps' `engine_heat`) at 60: two rising steps.
        let gauge = |k: usize| if k == 2 { 60.0 } else { 0.0 };
        damage_notices(
            &mut levels,
            gauge,
            |_| {},
            |q| -> Result<(), ()> {
                queued.push(q.notice);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(levels[0x16], 2);
        assert_eq!(queued, [0xCF, 0xCF]);
        // Back down below 50: one falling step (to 1; the step from 1 came before it).
        let gauge = |k: usize| if k == 2 { 10.0 } else { 0.0 };
        damage_notices(&mut levels, gauge, |_| {}, |_| -> Result<(), ()> { Ok(()) }).unwrap();
        assert_eq!(levels[0x16], 1);
        // A refusal stops the steps.
        let mut levels = [0u32; LEVELS];
        assert!(damage_notices(&mut levels, |_| f32::INFINITY, |_| {}, |_| Err(())).is_err());
    }
}
