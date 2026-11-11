//! The gear ratios and the sums between the engine's speed and the driven wheels'
//! (from `physics::make_gear_ratios` 0x4219E6,
//! `physics::revs_from_wheels` 0x43C0C9, `physics::wheels_from_revs` 0x43C124,
//! `physics::revs_from_speed` 0x43C101, `physics::drop_to_neutral` 0x43FD71 and
//! `physics::settle_drive_speeds` 0x43BF6F). The ratios are `g_gearRatios` (0x6140C0, by
//! gear: -1 the reverse at 0x6140BC, 0 neutral, seven forward gears).
//!
//! Floats: what the x87 keeps on its stack is an `f64`, rounded to `f32` where the
//! original stores (`physics::x87`): the same bits at the game's double precision. The
//! three sums are one product or quotient of floats: correctly rounded at any precision.

use super::x87::{below, loaded, negated};
use crate::Invalid;

/// The ratio table's entries: reverse (-1) ..= gear 7.
pub const RATIOS: usize = 9;

/// The gear's place in the ratio table (0 for reverse), or None in neutral; refused
/// outside -1 ..= 7.
pub fn ratio_index(gear: u32) -> Result<Option<usize>, Invalid> {
    if gear == 0 {
        return Ok(None);
    }
    if !(-1..=7).contains(&(gear as i32)) {
        return Err(Invalid("a gear outside g_gearRatios (-1 ..= 7)"));
    }
    Ok(Some((gear as i32 + 1) as usize))
}

/// `physics::revs_from_wheels` in gear: the driven speed (with drive kind 1 the two
/// driven speeds' sum times `half`, 0.5) over the gear's ratio.
pub fn revs_from_wheels(drive: f32, drive_2: f32, both: bool, half: f32, ratio: f32) -> f32 {
    if both {
        ((drive as f64 + drive_2 as f64) * half as f64 / ratio as f64) as f32
    } else {
        (drive as f64 / ratio as f64) as f32
    }
}

/// `physics::wheels_from_revs` in gear: (the driven speed, the second's with drive kind 1:
/// the same, reloaded).
pub fn wheels_from_revs(engine: f32, ratio: f32, both: bool) -> (f32, Option<f32>) {
    let speed = (engine as f64 * ratio as f64) as f32;
    (speed, both.then(|| loaded(speed)))
}

/// `physics::revs_from_speed` in gear: the car's speed over the gear's ratio.
pub fn revs_from_speed(speed: f32, ratio: f32) -> f32 {
    (speed as f64 / ratio as f64) as f32
}

/// `physics::settle_drive_speeds`' step: a driven speed brought toward 0 by `step`, never
/// past it. A speed of 0 (or no number) is left (None); one that reaches or crosses 0
/// becomes +0 (a no number after the step too). One sum of floats: correctly rounded at
/// any precision.
pub fn settle_speed(speed: f32, step: f32) -> Option<f32> {
    if speed == 0.0 || speed.is_nan() {
        return None;
    }
    if speed < 0.0 {
        let y = (speed as f64 + step as f64) as f32;
        Some(if y >= 0.0 { 0.0 } else { y })
    } else {
        let y = (speed as f64 - step as f64) as f32;
        Some(if y < 0.0 || y.is_nan() { 0.0 } else { y })
    }
}

/// `physics::drop_to_neutral`'s test: with the switch (`contact_flag`) set, in gear, the
/// engine speed below `limit` (`stall_engine_speed`; or unordered) and `block` (`no_auto_neutral`)
/// 0: the engine stalls into neutral (the caller sets gear 0 and the change's fields).
pub fn drops_to_neutral(
    switch: u32,
    gear: u32,
    engine: f32,
    limit: f32,
    block: impl FnOnce() -> u32,
) -> bool {
    switch != 0 && gear != 0 && below(engine as f64, limit as f64) && block() == 0
}

/// What `physics::make_gear_ratios` makes the ratios from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GearSetup {
    /// `gear_count`: the forward gears.
    pub count: u32,
    /// The top gear's ends (`flt_5CB978`, `flt_5CB970`) and the first's (`flt_5CB974`,
    /// `figures`).
    pub top: [f32; 2],
    pub first: [f32; 2],
    /// `gear_ratio_setting`: the setup's gearing (a whole number).
    pub gearing: i32,
    /// `quarter`, `two_and_a_quarter`, `engine_speed_limit`: the gearing's factor and two divisors.
    pub factor: f32,
    pub divisor: f32,
    pub engine_speed_limit: f32,
    /// `one` (1.0).
    pub one: f32,
    /// `gear_ratio_step`: neutral's.
    pub neutral: f32,
}

/// `fld a; fsub b; fmul k; fimul i; fadd b; fdiv d; fdiv w; fstp`.
fn setup_ratio(ends: [f32; 2], s: &GearSetup) -> f32 {
    let (a, b) = (ends[0] as f64, ends[1] as f64);
    (((a - b) * s.factor as f64 * s.gearing as f64 + b)
        / s.divisor as f64
        / s.engine_speed_limit as f64) as f32
}

/// `physics::make_gear_ratios`: the ratios (reverse ..= gear 7) and the step between
/// gears (kept in `scratch`). The top gear's and the first's from the setup; the
/// gears between in even steps added on at 80 bits (gears 2 .. 5 always, 6 and 7 one more
/// step each only past five and six gears; with four gears the fourth is a step's, as the
/// original); reverse the first's negative; neutral's copied. A count above 7 refused.
pub fn make_ratios(s: &GearSetup, ratios: &mut [f32; RATIOS]) -> Result<f32, Invalid> {
    if s.count > 7 {
        return Err(Invalid("a gear count above 7 (past g_gearRatios)"));
    }
    let n = s.count as usize;
    ratios[1 + n] = setup_ratio(s.top, s);
    ratios[2] = setup_ratio(s.first, s);
    let step = (ratios[1 + n] as f64 - ratios[2] as f64) / (s.count as i32 as f64 - s.one as f64);
    let mut acc = ratios[2] as f64;
    for gear in 2..=5 {
        acc += step;
        ratios[1 + gear] = acc as f32;
    }
    if n > 5 {
        acc += step;
    }
    ratios[7] = acc as f32;
    if n > 6 {
        acc += step;
    }
    ratios[8] = acc as f32;
    ratios[0] = negated(ratios[2]);
    ratios[1] = loaded(s.neutral);
    Ok(step as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums() {
        assert_eq!(ratio_index(0), Ok(None));
        assert_eq!(ratio_index(u32::MAX), Ok(Some(0)));
        assert_eq!(ratio_index(7), Ok(Some(8)));
        assert!(ratio_index(8).is_err());
        assert_eq!(revs_from_wheels(3.0, 5.0, true, 0.5, 2.0), 2.0);
        assert_eq!(revs_from_wheels(3.0, 5.0, false, 0.5, 2.0), 1.5);
        assert_eq!(wheels_from_revs(3.0, 2.0, true), (6.0, Some(6.0)));
        assert_eq!(wheels_from_revs(3.0, 2.0, false), (6.0, None));
        assert_eq!(revs_from_speed(1.0, 0.0), f32::INFINITY);
        assert_eq!(settle_speed(0.5, 0.01), Some(0.49));
        assert_eq!(settle_speed(-0.005, 0.01), Some(0.0));
        assert_eq!(settle_speed(0.005, 0.01), Some(0.0));
        assert_eq!(settle_speed(-0.0, 0.01), None);
        assert_eq!(settle_speed(1.0, f32::NAN), Some(0.0));
        assert!(drops_to_neutral(1, 2, 100.0, 200.0, || 0));
        assert!(drops_to_neutral(1, 2, f32::NAN, 200.0, || 0));
        assert!(!drops_to_neutral(1, 2, 200.0, 200.0, || 0));
        assert!(!drops_to_neutral(1, 0, 100.0, 200.0, || 0));
        assert!(!drops_to_neutral(1, 2, 100.0, 200.0, || 1));
    }

    #[test]
    fn ratios() {
        let s = GearSetup {
            count: 5,
            top: [1.0, 1.0],
            first: [3.0, 3.0],
            gearing: 0,
            factor: 1.0,
            divisor: 1.0,
            engine_speed_limit: 1.0,
            one: 1.0,
            neutral: 0.0,
        };
        let mut r = [0.0f32; RATIOS];
        assert_eq!(make_ratios(&s, &mut r), Ok(-0.5));
        assert_eq!(r, [-3.0, 0.0, 3.0, 2.5, 2.0, 1.5, 1.0, 1.0, 1.0]);
        let mut r = [0.0f32; RATIOS];
        let seven = GearSetup { count: 7, ..s };
        assert_eq!(make_ratios(&seven, &mut r), Ok(-1.0 / 3.0));
        assert_eq!(r[8], 1.0);
        assert!(make_ratios(&GearSetup { count: 8, ..s }, &mut r).is_err());
    }
}
