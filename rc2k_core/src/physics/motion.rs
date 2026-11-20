//! The work car's step of motion in floats: the forces summed, the car moved, the wheels spun,
//! air drag, the spin held on the ground, the wheel load blended, the lowest body point, the
//! timers counted, the motion's length stored and the car's angles made. Floats: the x87's
//! stack values are `f64`, rounded to `f32` where stored.

use super::x87::{abs, below, equal, fistp64, loaded, negated};
use crate::Invalid;

/// The terms' sum over the divisors.
pub fn sum_over(terms: &[f32], divisors: &[f32]) -> f32 {
    let mut acc = terms[0] as f64;
    for &t in &terms[1..] {
        acc += t as f64;
    }
    for &d in divisors {
        acc /= d as f64;
    }
    acc as f32
}

/// The motion's length, rounded to a float.
pub fn motion_length(motion: [f32; 3]) -> f32 {
    crate::math::vector::length(motion) as f32
}

/// The step's five sums of the thirteen forces over gravity, the first three also over the
/// step.
pub fn sum_forces(f: &[f32; 13], gravity: f32, step: f32) -> [f32; 5] {
    [
        sum_over(&f[0..1], &[gravity, step]),
        sum_over(&f[1..2], &[gravity, step]),
        sum_over(&f[2..4], &[gravity, step]),
        sum_over(&f[4..8], &[gravity]),
        sum_over(&f[8..13], &[gravity]),
    ]
}

/// The place (whole numbers) moved by the motion times `scale` and the step, x, z, then y;
/// each move is rounded; returns the last (y's).
pub fn move_place(motion: [f32; 3], place: &mut [u32; 3], scale: f32, step: f32) -> i32 {
    let mut last = 0;
    for k in [0, 2, 1] {
        last = fistp64(motion[k] as f64 * scale as f64 * step as f64);
        place[k] = place[k].wrapping_add(last as u32);
    }
    last
}

/// The two driven wheels' turns on by their drive speeds over the wheel's size, times the
/// step, 65536 and `scale`: returns the second's step.
pub fn spin(
    speeds: [f32; 2],
    turns: &mut [u32; 2],
    wheel: f32,
    step: f32,
    k: f32,
    scale: f32,
) -> i32 {
    let mut last = 0;
    for (speed, turn) in speeds.iter().zip(turns.iter_mut()) {
        last = fistp64(*speed as f64 / wheel as f64 * step as f64 * k as f64 * scale as f64);
        *turn = turn.wrapping_add(last as u32);
    }
    last
}

/// Air drag: each of the motion's three parts gets its square times `k` over `d`, made
/// against the motion.
pub fn air_drag(motion: [f32; 3], k: f32, d: f32, zero: f32) -> [f32; 3] {
    motion.map(|m| {
        let size = abs((m as f64 * m as f64 * k as f64 / d as f64) as f32);
        if below(m as f64, zero as f64) {
            size
        } else {
            negated(size)
        }
    })
}

/// With all four wheels down: the spin's x and z held to ±`limit`; the flag says either now
/// has the limit's size (or no number).
pub fn hold_spin(limit: &mut f32, x: &mut f32, z: &mut f32) -> u32 {
    for s in [&mut *x, &mut *z] {
        if !below(*s as f64, *limit as f64) {
            *s = loaded(*limit);
        }
        *limit = negated(*limit);
        if below(*s as f64, *limit as f64) {
            *s = loaded(*limit);
        }
        *limit = negated(*limit);
    }
    let size = |v: f32| v.abs() as f64;
    u32::from(equal(size(*x), *limit as f64) || equal(size(*z), *limit as f64))
}

#[cfg(test)]
/// The figures of the wheel-load blend.
pub(crate) struct LoadBlend {
    /// The contact depth.
    pub figure: f32,
    /// Three contact figures: their signs pick the slot.
    pub signs: [f32; 3],
    /// Threshold the figure must not be below.
    pub threshold: f32,
    /// What the figure is divided by.
    pub divisor: f32,
    /// The constant 0.
    pub zero: f32,
    /// The constant 1.
    pub one: f32,
}

#[cfg(test)]
/// Unless the figure is below the threshold, the slot (4 + 2 + 1 for each sign below zero)
/// moves towards 1 by the figure over the divisor held to 0 .. 1: (the slot, the share).
pub(crate) fn blend_load(b: &LoadBlend, slots: &mut [f32; 8]) -> Option<(u32, f32)> {
    if below(b.figure as f64, b.threshold as f64) {
        return None;
    }
    let mut slot = 0;
    for (sign, add) in b.signs.iter().zip([4, 2, 1]) {
        if below(*sign as f64, b.zero as f64) {
            slot += add;
        }
    }
    let mut share = (b.figure as f64 / b.divisor as f64) as f32;
    if below(share as f64, b.zero as f64) {
        share = loaded(b.zero);
    }
    if !below(share as f64, b.one as f64) {
        share = loaded(b.one);
    }
    let a = slots[slot as usize] as f64;
    slots[slot as usize] = ((1.0 - a) * share as f64 + a) as f32;
    Some((slot, share))
}

/// With more than four probe points, the deepest of the body's (points 4 ..) into `lowest`
/// and its number into `number` (-1 for none): (the last depth looked at, the count). A count
/// above 0x40 is refused.
pub fn lowest_point(
    count: u32,
    depth: impl Fn(u32) -> u32,
    lowest: &mut u32,
    number: &mut u32,
) -> Result<Option<(u32, u32)>, Invalid> {
    if count > 0x40 {
        return Err(Invalid("more than 0x40 probe points (the depths' room)"));
    }
    if count <= 4 {
        return Ok(None);
    }
    *lowest = 0x64_0000;
    *number = u32::MAX;
    let mut last = 0;
    for n in 4..count {
        last = depth(n);
        if (last.wrapping_sub(*lowest) as i32) < 0 {
            *lowest = last;
            *number = n;
        }
    }
    Ok(Some((last, count)))
}

#[cfg(test)]
/// The lowest body point (a whole number) over `scale`.
pub(crate) fn lowest_share(lowest: u32, scale: f32) -> f32 {
    ((lowest as i32) as f64 / scale as f64) as f32
}

/// The timers counted down each step.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StepCounters {
    /// Down by one when not 0.
    pub first: [u32; 2],
    /// The steps standing in neutral in mode 1.
    pub standing: u32,
    /// On by one unless 0, -1 or 1.
    pub running: u32,
    /// Down by one when not 0.
    pub later: [u32; 3],
    /// The highest engine speed so far.
    pub highest: f32,
}

/// What the countdown reads besides its counters.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StepFacts {
    /// The steps run so far.
    pub steps: u32,
    /// Three switches the countdown reads.
    pub switches: [u32; 3],
    /// The run mode.
    pub mode: u32,
    /// The bits of the parts present.
    pub bits: u32,
    /// The gear.
    pub gear: i32,
    /// The scaled engine speed.
    pub figure: f32,
}

/// The leftover value (the part bits & 0x1E when that is looked at).
pub fn count_down(c: &mut StepCounters, f: &StepFacts) -> Option<u32> {
    let down = |v: &mut u32| {
        if *v != 0 {
            *v -= 1;
        }
    };
    c.first.iter_mut().for_each(down);
    let mut last = None;
    let reset = 'standing: {
        if f.steps < 1000 || f.switches.iter().any(|&s| s != 0) {
            break 'standing Some(true);
        }
        if f.mode != 1 {
            break 'standing None;
        }
        let bits = f.bits & 0x1E;
        last = Some(bits);
        Some(bits == 0x1E || f.gear != 0)
    };
    match reset {
        Some(true) => c.standing = 0,
        Some(false) => c.standing = c.standing.wrapping_add(1),
        None => {}
    }
    if !matches!(c.running, 0 | 1 | u32::MAX) {
        c.running = c.running.wrapping_add(1);
    }
    c.later[..2].iter_mut().for_each(down);
    if !below(f.figure as f64, c.highest as f64) {
        c.highest = loaded(f.figure);
    }
    down(&mut c.later[2]);
    last
}

/// The three stop counters in turn; one that is not 0: at 300 the result (-1 for the first
/// two, 2 for the third) and done; else while `speed` is below `one` (or unordered) it goes up,
/// and on reaching 300 (unsigned) the same.
pub fn stop_timers(counters: &mut [u32; 3], speed: f32, one: f32) -> Option<u32> {
    let slow = below(speed as f64, one as f64);
    for (c, result) in counters.iter_mut().zip([u32::MAX, u32::MAX, 2]) {
        if *c == 0 {
            continue;
        }
        if *c == 300 {
            return Some(result);
        }
        if !slow {
            continue;
        }
        *c = c.wrapping_add(1);
        if *c >= 300 {
            return Some(result);
        }
    }
    None
}

/// Whether the car's angles are made: forced, or the wheels are not all off.
pub fn angles_wanted(force: u32, hold: u32) -> bool {
    force != 0 || hold == 0
}

#[cfg(test)]
mod tests;
