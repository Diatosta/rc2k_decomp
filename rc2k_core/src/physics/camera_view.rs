//! The camera's view steps (converted 2026-10-21 from `physics::take_view_a` 0x437F53,
//! `physics::take_view_b` 0x438024, `physics::ease_view_angles` 0x43846D,
//! `physics::ease_view_blend` 0x4385EB, `physics::camera_height_bias` 0x43852D and
//! `physics::ease_view_turn` 0x438B4F): a view's place, the share of a change done, the
//! eased angles, turns, blend and height. What the figures are (the camera's mode,
//! place, turns, a view's figures) is a reading.
//!
//! Floats: what the x87 keeps on its stack is an `f64`, rounded to `f32` where the
//! original stores (`physics::x87`): the same bits at the game's double precision.

use super::x87::{fistp64, hold_high, hold_low};

/// The camera modes' records (`off_606320`): 0x34.
pub const MODES: u32 = 0x34;

/// The mode (`unk_71A65C`) held to 0 ..= 0x33.
pub fn mode_index(mode: u32) -> u32 {
    (mode as i32).clamp(0, MODES as i32 - 1) as u32
}

/// The share of a change done (0x438498 .. 0x4384BD): the time since its start
/// (`unk_719FE0` - `unk_71A668`) held to 0 ..= the record's change time (+0x3C, by the
/// sign, then unsigned): (the time held, the original's eax and kept in `dword_62400C`;
/// over the change time, `flt_605260`).
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

/// `take_view_a` / `take_view_b`: the view's place (its record's +4 .. +0xC) plus the
/// offset (`dword_637638` ..).
pub fn view_place(record: [f32; 3], offset: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|k| (record[k] as f64 + offset[k] as f64) as f32)
}

/// `take_view_a` / `take_view_b`: the two turns (`unk_71A99C`, `unk_71A9A0`), halved
/// (`sar`) when `halve`.
pub fn view_turns(turns: [u32; 2], halve: bool) -> [u32; 2] {
    if halve {
        turns.map(|t| ((t as i32) >> 1) as u32)
    } else {
        turns
    }
}

/// `ease_view_angles`: each of the record's three angle changes (+0x40 ..) times the
/// share done and `scale` (`flt_5C5D98`, 65536) `fistp`ed (through `dword_62400C`),
/// << 16, added to its angle (`unk_71A694` ..): (the last whole number, the last added).
pub fn ease_angles(changes: [f32; 3], share: f32, scale: f32, angles: &mut [u32; 3]) -> (i32, u32) {
    let (mut n, mut added) = (0, 0);
    for (change, angle) in changes.iter().zip(angles.iter_mut()) {
        n = fistp64(*change as f64 * share as f64 * scale as f64);
        added = (n as u32) << 16;
        *angle = angle.wrapping_add(added);
    }
    (n, added)
}

/// What `ease_view_blend` makes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blend {
    /// The share done, 1 less it for a blend below 0 (stored back into `flt_605260`).
    pub share: f32,
    /// `unk_71A6A0`: between `flt_608B20` and `flt_608B24`.
    pub between: f32,
    /// `dword_608B38`: between `flt_608B28` and `flt_608B2C`, x `scale`, negated.
    pub tilt: i32,
}

/// `ease_view_blend`: `ends` the four (`flt_608B20` .. `flt_608B2C`).
pub fn ease_blend(share: f32, below_zero: bool, ends: [f32; 4], scale: f32) -> Blend {
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

/// `camera_height_bias`: `a` (`flt_60631C`) over `b` (`flt_606290`) held to `low` ..=
/// `high` (`flt_5C5D90`, `flt_5C5D38`), 1 less it for a bias below 0, x `scale`
/// (`flt_5C5D98`) x `c` (`flt_60628C`), `fistp`ed (through `dword_62400C`).
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

/// `ease_view_turn`: each turn moved toward its target by its share: the difference
/// (whole numbers) times the share `fistp`ed (through `dword_62400C`), added: the last
/// step.
pub fn ease_turns(targets: [u32; 3], turns: &mut [u32; 3], shares: [f32; 3]) -> i32 {
    let mut step = 0;
    for k in 0..3 {
        let difference = targets[k].wrapping_sub(turns[k]) as i32;
        step = fistp64(difference as f64 * shares[k] as f64);
        turns[k] = turns[k].wrapping_add(step as u32);
    }
    step
}

#[cfg(test)]
mod tests {
    use super::*;

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
