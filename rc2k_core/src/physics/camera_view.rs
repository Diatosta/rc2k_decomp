//! The camera's view steps (converted 2026-10-21 from `physics::take_first_view` 0x437F53,
//! `physics::take_second_view` 0x438024, `physics::ease_view_angles` 0x43846D,
//! `physics::ease_view_blend` 0x4385EB, `physics::camera_height_bias` 0x43852D and
//! `physics::ease_view_turn` 0x438B4F): a view's place, the share of a change done, the
//! eased angles, turns, blend and height. The figures are the car block's camera fields
//! (`Car::camera_view` the mode, `camera_position`, `camera_heading`, `camera_pitch`,
//! `camera_offset_turns`, the shares) and the camera mode records' (`off_606320`) offsets and turns. The camera's step chooses (converted
//! 2026-10-21 from `physics::update_camera` 0x437A53 and `physics::camera_frame`
//! 0x44FF1B; their x87 sums stay the hooks'): the fixed cameras, the end of a switch,
//! the modes the views' table holds, and when the camera shakes.
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

/// `take_first_view` / `take_second_view`: the view's place (its record's +4 .. +0xC) plus the
/// offset (`dword_637638` ..).
pub fn view_place(record: [f32; 3], offset: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|k| (record[k] as f64 + offset[k] as f64) as f32)
}

/// `take_first_view` / `take_second_view`: the two turns (`unk_71A99C`, `unk_71A9A0`), halved
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

/// A fixed camera mode's tables: the second ones (`g_nearestCamera2`,
/// `g_replayCameras2`: modes 0x2A ..= 0x2D, 9, 10) or the first two (0x2E ..= 0x31, 7, 8).
pub fn fixed_camera(mode: u32) -> Option<bool> {
    match mode {
        0x2A..=0x2D | 9 | 0xA => Some(true),
        0x2E..=0x31 | 7 | 8 => Some(false),
        _ => None,
    }
}

/// The views' table (`off_606320`) holds modes 0 ..= 0x33.
pub const LAST_MODE: u32 = 0x33;

/// The mode and the one before checked against the views' table.
pub fn check_modes(mode: u32, before: u32) -> Result<(), crate::Invalid> {
    if mode > LAST_MODE || before > LAST_MODE {
        return Err(crate::Invalid(
            "a camera mode past 0x33 (off_606320's room)",
        ));
    }
    Ok(())
}

/// Whether a switch is over: the clock (`unk_719FE0`) less its start (`unk_71A668`) not
/// below its length (`unk_71A670`), by the sign.
pub fn switch_over(clock: u32, began: u32, length: u32) -> bool {
    (clock.wrapping_sub(began).wrapping_sub(length) as i32) >= 0
}

/// The mode after a switch: the one before, but 7 or more (unsigned) back to 0.
pub fn mode_after_switch(before: u32) -> u32 {
    if before >= 7 { 0 } else { before }
}

/// A switch's end: the views' hold (`unk_71AF88`).
pub const SWITCH_HOLD: u32 = 8;

/// Whether the car's light record keeps its +0x20 figure for `flt_606574`: a car below
/// 0x22 marked in `dword_5C5AC0` (`marked`, asked only then); else +0x28.
pub fn keeps_light_figure(car: u32, marked: impl FnOnce() -> u32) -> bool {
    car < 0x22 && marked() != 0
}

/// `physics::camera_frame`: whether the camera shakes: modes 1 ..= 3 with a wheel on the
/// ground (`unk_71A22C` 0).
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
