//! The gear ratios (-1 the reverse, 0 neutral, seven forward gears) and the sums between the
//! engine's speed and the driven wheels'.
//!
//! Floats: what the x87 keeps on its stack is an `f64`, rounded to `f32` where it is stored.
//! The three sums are one product or quotient of floats: correctly rounded at any precision.

#[cfg(test)]
use super::x87::negated;
use super::x87::{below, loaded};
#[cfg(test)]
use crate::Invalid;

#[cfg(test)]
/// The ratio table's entries: reverse (-1) ..= gear 7.
pub(crate) const RATIOS: usize = 9;

#[cfg(test)]
/// The gear's place in the ratio table (gear + 1), or None in neutral; refused outside -1 ..= 7.
pub(crate) fn ratio_index(gear: i32) -> Result<Option<usize>, Invalid> {
    if gear == 0 {
        return Ok(None);
    }
    if !(-1..=7).contains(&gear) {
        return Err(Invalid("a gear outside g_gearRatios (-1 ..= 7)"));
    }
    Ok(Some((gear + 1) as usize))
}

/// The revs from the wheels in gear: the driven speed (with drive kind 1 the two speeds' sum
/// times `half`) over the gear's ratio.
pub fn revs_from_wheels(drive: f32, drive_2: f32, both: bool, half: f32, ratio: f32) -> f32 {
    if both {
        ((drive as f64 + drive_2 as f64) * half as f64 / ratio as f64) as f32
    } else {
        (drive as f64 / ratio as f64) as f32
    }
}

/// The wheels from the revs in gear: the driven speed, and with drive kind 1 the same again.
pub fn wheels_from_revs(engine: f32, ratio: f32, both: bool) -> (f32, Option<f32>) {
    let speed = (engine as f64 * ratio as f64) as f32;
    (speed, both.then(|| loaded(speed)))
}

/// The revs from the speed in gear: the car's speed over the gear's ratio.
pub fn revs_from_speed(speed: f32, ratio: f32) -> f32 {
    (speed as f64 / ratio as f64) as f32
}

#[cfg(test)]
/// A driven speed brought toward 0 by `step`, never past it. A speed of 0 or not a number is
/// left (None); one that reaches or crosses 0 becomes +0.
pub(crate) fn settle_speed(speed: f32, step: f32) -> Option<f32> {
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

/// Whether the engine stalls into neutral: the switch set, in gear, the engine speed below
/// `limit` (or unordered) and `block` 0.
pub fn drops_to_neutral(
    switch: u32,
    gear: i32,
    engine: f32,
    limit: f32,
    block: impl FnOnce() -> u32,
) -> bool {
    switch != 0 && gear != 0 && below(engine as f64, limit as f64) && block() == 0
}

#[cfg(test)]
/// What the ratios are made from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GearSetup {
    /// The forward gears.
    pub count: u32,
    /// The top gear's ends and the first's.
    pub top: [f32; 2],
    pub first: [f32; 2],
    /// The setup's gearing (a whole number).
    pub gearing: i32,
    /// The gearing's factor and two divisors.
    pub factor: f32,
    pub divisor: f32,
    pub engine_speed_limit: f32,
    /// 1.0.
    pub one: f32,
    /// Neutral's ratio.
    pub neutral: f32,
}

#[cfg(test)]
fn setup_ratio(ends: [f32; 2], s: &GearSetup) -> f32 {
    let (a, b) = (ends[0] as f64, ends[1] as f64);
    (((a - b) * s.factor as f64 * s.gearing as f64 + b)
        / s.divisor as f64
        / s.engine_speed_limit as f64) as f32
}

#[cfg(test)]
/// The ratios (reverse ..= gear 7) and the step between gears. The top gear's and the first's
/// come from the setup; gears 2 ..= 5 follow in even steps, the sixth and seventh one more
/// step each only past five and six gears (with four gears the fourth is a step's); reverse is
/// the first's negative and neutral's is copied. A count above 7 is refused.
pub(crate) fn make_ratios(s: &GearSetup, ratios: &mut [f32; RATIOS]) -> Result<f32, Invalid> {
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
        assert_eq!(ratio_index(-1), Ok(Some(0)));
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
