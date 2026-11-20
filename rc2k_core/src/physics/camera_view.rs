//! The camera's view steps: a view's place, the share of a change done, the eased angles,
//! turns, blend and height, the fixed-camera choice and the shake test.
//!
//! Floats: what the x87 keeps on its stack is an `f64`, rounded to `f32` where it is stored.

use super::x87::{fistp64, hold_high, hold_low};

pub const MODES: u32 = 0x34;

#[cfg(test)]
/// The mode held to 0 ..= 0x33.
pub(crate) fn mode_index(mode: u32) -> u32 {
    (mode as i32).clamp(0, MODES as i32 - 1) as u32
}

/// The share of a change done: the time since `start` held to 0 ..= `time` (by sign, then
/// unsigned), and its ratio to `time`.
pub fn share_done(now: u32, start: u32, time: u32) -> (u32, f32) {
    let mut held = now.wrapping_sub(start);
    if (held as i32) < 0 {
        held = 0;
    }
    if held >= time {
        held = time;
    }
    (held, (held as i32 as f64 / time as i32 as f64) as f32)
}

/// A view's place: the record's plus the offset.
pub fn view_place(record: [f32; 3], offset: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|k| (record[k] as f64 + offset[k] as f64) as f32)
}

#[cfg(test)]
/// A view's two turns, halved when `halve`.
pub(crate) fn view_turns(turns: [u32; 2], halve: bool) -> [u32; 2] {
    if halve {
        turns.map(|t| ((t as i32) >> 1) as u32)
    } else {
        turns
    }
}

#[cfg(test)]
/// Each angle change times the share and `scale`, rounded, << 16, added to its angle;
/// returns the last whole number and the last added.
pub(crate) fn ease_angles(
    changes: [f32; 3],
    share: f32,
    scale: f32,
    angles: &mut [u32; 3],
) -> (i32, u32) {
    let (mut n, mut added) = (0, 0);
    for (change, angle) in changes.iter().zip(angles.iter_mut()) {
        n = fistp64(*change as f64 * share as f64 * scale as f64);
        added = (n as u32) << 16;
        *angle = angle.wrapping_add(added);
    }
    (n, added)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blend {
    /// The share done, 1 less it for a blend below 0.
    pub share: f32,
    pub between: f32,
    /// Between the last two ends, times `scale`, negated.
    pub tilt: i32,
}

#[cfg(test)]
/// `ends` are the four shift figures.
pub(crate) fn ease_blend(share: f32, below_zero: bool, ends: [f32; 4], scale: f32) -> Blend {
    let share = if below_zero {
        (1.0 - share as f64) as f32
    } else {
        share
    };
    let s = share as f64;
    let lerp = |a: f32, b: f32| (b as f64 - a as f64) * s + a as f64;
    Blend {
        share,
        between: lerp(ends[0], ends[1]) as f32,
        tilt: fistp64(-(lerp(ends[2], ends[3]) * scale as f64)),
    }
}

/// The height bias: `a` over `b` held to `low` ..= `high`, 1 less it when `below_zero`, times
/// `scale` and `c`, rounded.
pub fn height_bias(
    a: f32,
    b: f32,
    low: f32,
    high: f32,
    below_zero: bool,
    scale: f32,
    c: f32,
) -> i32 {
    let mut x = hold_high(hold_low((a as f64 / b as f64) as f32, low), high);
    if below_zero {
        x = (1.0 - x as f64) as f32;
    }
    fistp64(x as f64 * scale as f64 * c as f64)
}

#[cfg(test)]
/// Moves each turn toward its target by its share; returns the last step.
pub(crate) fn ease_turns(targets: [u32; 3], turns: &mut [u32; 3], shares: [f32; 3]) -> i32 {
    let mut step = 0;
    for k in 0..3 {
        let difference = targets[k].wrapping_sub(turns[k]) as i32;
        step = fistp64(difference as f64 * shares[k] as f64);
        turns[k] = turns[k].wrapping_add(step as u32);
    }
    step
}

/// Which table a fixed camera mode uses: the second (0x2A ..= 0x2D, 9, 10) or the first
/// (0x2E ..= 0x31, 7, 8); None for other modes.
pub fn fixed_camera(mode: u32) -> Option<bool> {
    match mode {
        0x2A..=0x2D | 9 | 0xA => Some(true),
        0x2E..=0x31 | 7 | 8 => Some(false),
        _ => None,
    }
}

pub(crate) const LAST_MODE: u32 = 0x33;

/// Checks the mode and the one before against the table.
pub fn check_modes(mode: u32, before: u32) -> Result<(), crate::Invalid> {
    if mode > LAST_MODE || before > LAST_MODE {
        return Err(crate::Invalid("a camera mode past 0x33 (the table's room)"));
    }
    Ok(())
}

/// Whether a switch is over: the time since its start is not below its length (by sign).
pub fn switch_over(clock: u32, began: u32, length: u32) -> bool {
    (clock.wrapping_sub(began).wrapping_sub(length) as i32) >= 0
}

/// The mode after a switch: the one before, but 7 or more (unsigned) back to 0.
pub fn mode_after_switch(before: u32) -> u32 {
    if before >= 7 { 0 } else { before }
}

pub const SWITCH_HOLD: u32 = 8;

/// Whether the light record keeps its first figure rather than the second: a car below 0x22
/// that `marked` approves (asked only then).
pub fn keeps_light_figure(car: u32, marked: impl FnOnce() -> u32) -> bool {
    car < 0x22 && marked() != 0
}

/// Whether the camera shakes: modes 1 ..= 3 with a wheel on the ground.
pub fn shakes(mode: u32, all_off: u32) -> bool {
    matches!(mode, 1..=3) && all_off == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_choices() {
        assert_eq!(fixed_camera(0x2B), Some(true));
        assert_eq!(fixed_camera(8), Some(false));
        assert_eq!(fixed_camera(3), None);
        assert!(check_modes(0x33, 0).is_ok() && check_modes(0, 0x34).is_err());
        assert!(switch_over(10, 2, 8) && !switch_over(10, 2, 9));
        assert_eq!(mode_after_switch(7), 0);
        assert_eq!(mode_after_switch(6), 6);
        assert!(!keeps_light_figure(0x22, || panic!()));
        assert!(keeps_light_figure(3, || 1));
        assert!(shakes(2, 0) && !shakes(2, 1) && !shakes(4, 0));
    }

    #[test]
    fn shares_and_places() {
        assert_eq!(mode_index(u32::MAX), 0);
        assert_eq!(mode_index(0x40), 0x33);
        assert_eq!(share_done(1100, 1000, 400), (100, 0.25));
        assert_eq!(share_done(900, 1000, 400), (0, 0.0));
        assert_eq!(share_done(2000, 1000, 400), (400, 1.0));
        assert_eq!(
            view_place([1.0, 2.0, 3.0], [0.5, 0.5, -1.0]),
            [1.5, 2.5, 2.0]
        );
        assert_eq!(view_turns([u32::MAX - 2, 6], true), [u32::MAX - 1, 3]);
        assert_eq!(view_turns([5, 6], false), [5, 6]);
    }

    #[test]
    fn eases() {
        let mut angles = [0u32, 1, 2];
        assert_eq!(
            ease_angles([1.0, -0.5, 0.25], 0.5, 4.0, &mut angles),
            (0, 0)
        );
        assert_eq!(angles, [2 << 16, 1u32.wrapping_sub(1 << 16), 2]);
        let b = ease_blend(0.25, true, [0.0, 4.0, 1.0, 3.0], 2.0);
        assert_eq!((b.share, b.between, b.tilt), (0.75, 3.0, -5));
        assert_eq!(height_bias(3.0, 2.0, 0.0, 1.0, false, 4.0, 2.0), 8);
        assert_eq!(height_bias(1.0, 4.0, 0.0, 1.0, true, 4.0, 2.0), 6);
        let mut turns = [0u32, 10, 0];
        assert_eq!(
            ease_turns([100, 0, u32::MAX - 9], &mut turns, [0.5, 0.25, 1.0]),
            -10
        );
        assert_eq!(turns, [50, 8, u32::MAX - 9]);
    }
}
