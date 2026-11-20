//! The engine's torque from its speed: the speed's share of its top, a curve of eight steps,
//! the cut at the top, the engine revving free, the drive's two figures, and the revs heard
//! eased toward the engine's speed.
//!
//! Floats: what the x87 keeps on its stack is an `f64`, rounded to `f32` where it is stored.
//! The curve's entry comes from the stored share times the steps (exact in an `f64`): the same
//! entry at any precision.

use super::x87::{below, loaded};

/// The whole number toward 0 (chopped), `i32::MIN` for what does not fit or is not a number.
pub(crate) fn fistp_chop(v: f64) -> i32 {
    let r = v.trunc();
    if r.is_nan() || r < i32::MIN as f64 || r >= 2147483648.0 {
        i32::MIN
    } else {
        r as i32
    }
}

/// The engine's figures the torque is made from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Engine {
    /// The engine's speed and its top.
    pub speed: f32,
    pub top: f32,
    /// 0 and 1: the curve's range.
    pub zero: f32,
    pub one: f32,
    /// 8: the curve's steps.
    pub steps: f32,
    /// 0.25: the share's factor for the torque.
    pub quarter: f32,
    /// The torque's factor.
    pub factor: f32,
}

/// What the engine torque makes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Torque {
    /// The speed's share of its top.
    pub share: f32,
    /// The curve's entry, when the share is in 0 ..= below 1.
    pub entry: Option<u32>,
    /// The share x `quarter`, from the unrounded quotient.
    pub revs: f32,
    /// The curve's line at the share (or its first value), x `factor`.
    pub torque: f32,
}

/// `curve(i)` gives the line's two ends (curve x, curve y), asked only on the curve's path;
/// `first` gives the first y, asked only off it.
pub fn engine_torque(
    e: &Engine,
    curve: impl FnOnce(u32) -> ([f32; 2], [f32; 2]),
    first: impl FnOnce() -> f32,
) -> Torque {
    let quotient = e.speed as f64 / e.top as f64;
    let share = quotient as f32;
    let (entry, value) = if below(share as f64, e.one as f64) && !below(share as f64, e.zero as f64)
    {
        let i = fistp_chop(share as f64 * e.steps as f64) as u32;
        let ([a0, a1], [c0, c1]) = curve(i);
        let s = share as f64;
        let line = (s - a0 as f64) / (a1 as f64 - a0 as f64) * (c1 as f64 - c0 as f64) + c0 as f64;
        (Some(i), line as f32)
    } else {
        (None, loaded(first()))
    };
    Torque {
        share,
        entry,
        revs: (quotient * e.quarter as f64) as f32,
        torque: (value as f64 * e.factor as f64) as f32,
    }
}

/// The cut: at or past the top (unordered counts as below: no cut) the cut is set to 5;
/// below it, while the cut counts down, it goes down by one. Either way the torque is 0 then
/// (Some(the cut)); else None.
pub fn torque_cut(speed: f32, top: f32, cut: u32) -> Option<u32> {
    if !below(speed as f64, top as f64) {
        Some(5)
    } else if cut != 0 {
        Some(cut.wrapping_sub(1))
    } else {
        None
    }
}

/// What the free rev reads besides the torque's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FreeRev {
    /// The drive ratio and the drive torque figure.
    pub drive: f32,
    pub drive_factor: f32,
    /// Gear 0's ratio and 10000.
    pub ratio: f32,
    pub k10000: f32,
    /// The throttle gearing (a whole number) and 256.
    pub gearing: i32,
    pub k256: f32,
    /// The step's time.
    pub step: f32,
    /// The engine's speed.
    pub engine: f32,
    /// The drag's factor and base.
    pub drag_factor: f32,
    pub drag_base: f32,
}

/// Returns (the push, the engine speed, the drag). The engine speed is brought toward 0 by the
/// drag, not past it (at 0 or not a number left alone).
pub fn free_rev(f: &FreeRev) -> (f32, f32, f32) {
    let push = (f.drive as f64 * f.drive_factor as f64 / f.ratio as f64 / f.k10000 as f64
        * f.gearing as f64
        / f.k256 as f64) as f32;
    let engine = (push as f64 * f.step as f64 / f.ratio as f64 + f.engine as f64) as f32;
    let drag = (engine as f64 * f.drag_factor as f64 + f.drag_base as f64) as f32;
    let eased = if engine == 0.0 || engine.is_nan() {
        engine
    } else if engine < 0.0 {
        let y = (engine as f64 + drag as f64) as f32;
        if below(y as f64, 0.0) { y } else { 0.0 }
    } else {
        let y = (engine as f64 - drag as f64) as f32;
        if below(y as f64, 0.0) { 0.0 } else { y }
    };
    (push, eased, drag)
}

/// What the drive's two figures are made from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Drive {
    /// The engine's torque and its base.
    pub torque: f32,
    pub base: f32,
    /// The throttle gearing (a whole number) and 256.
    pub gearing: i32,
    pub k256: f32,
    /// The drive torque figure, the gear's ratio and 10000.
    pub factor: f32,
    pub ratio: f32,
    pub k10000: f32,
    /// The drive cut is set.
    pub cut: bool,
}

/// Returns (the drive ratio, the drive torque): ratio = (torque + base) x gearing / 256 - base;
/// torque = ratio x factor / gear ratio / 10000, or 0 with the cut.
pub fn drive(d: &Drive) -> (f32, f32) {
    let b = d.base as f64;
    let engine = ((d.torque as f64 + b) * d.gearing as f64 / d.k256 as f64 - b) as f32;
    let drive = if d.cut {
        0.0
    } else {
        (engine as f64 * d.factor as f64 / d.ratio as f64 / d.k10000 as f64) as f32
    };
    (engine, drive)
}

/// Whether the work car's figure is above the sound object's (not equal, not below, not
/// unordered): the four figures then go to the object.
pub fn shares_figures(figure: f32, kept: f32) -> bool {
    !below(figure as f64, kept as f64) && figure != kept
}

/// The revs heard are held to this range (by the sign of the difference).
pub(crate) const REVS_LOW: u32 = 2000;
pub(crate) const REVS_HIGH: u32 = 12000;

/// Eases the revs heard toward `target` by 50/256 of the gap, then holds them to
/// `REVS_LOW` ..= `REVS_HIGH`. Returns (the revs, the step).
pub fn ease_revs(revs: u32, target: u32) -> (u32, u32) {
    let step = ((target.wrapping_sub(revs).wrapping_mul(50)) as i32 >> 8) as u32;
    let mut r = revs.wrapping_add(step);
    if (r.wrapping_sub(REVS_LOW) as i32) < 0 {
        r = REVS_LOW;
    }
    if (r.wrapping_sub(REVS_HIGH) as i32) >= 0 {
        r = REVS_HIGH;
    }
    (r, step)
}

#[cfg(test)]
mod tests;
