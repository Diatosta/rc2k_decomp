//! The camera's ground checks (converted 2026-10-21 from `physics::camera_low_switch`
//! 0x438BF4, `camera_over_ground` 0x438CEC and `camera_clear_of_ground` 0x438DD2): how
//! high the camera is over the ground found under it, the switch begun when it is low,
//! and the points across its heading that must have ground. What the camera variables
//! (`unk_71A65C` ..) are is a reading: the mode, the place, the turns. And the camera
//! kept out of the ground (converted 2026-10-21 from `physics::camera_avoid_ground`
//! 0x43907F): turned to face from a checkpoint behind, then back toward its heading in
//! halving-by-eight steps, then pitched up or down until clear.

/// Below this over the ground (signed) the camera is low.
pub const LOW: u32 = 0x800;
/// At this over the ground (signed) or more it is clear.
pub const CLEAR: u32 = 0x10000;
/// The two points across the heading (x, a half unit either way) that must have ground,
/// in the original's order.
pub const SIDES: [u32; 2] = [0xFFFF_8000, 0x8000];

/// The camera's height over the ground (`unk_71A680` - `dword_639828`, wrapping).
pub fn over(camera_y: u32, ground: u32) -> u32 {
    camera_y.wrapping_sub(ground)
}

/// Low: below `LOW` by the sign of the `cmp`.
pub fn is_low(gap: u32) -> bool {
    (gap.wrapping_sub(LOW) as i32) < 0
}

/// Clear: `CLEAR` or more by the sign of the `cmp`.
pub fn is_clear(gap: u32) -> bool {
    (gap.wrapping_sub(CLEAR) as i32) >= 0
}

/// `camera_low_switch` looks: `unk_71A0A0` below 0 (or no number), no switch under way
/// (`unk_71A66C` 0) and the mode (`unk_71A65C`) 0 or 1.
pub fn low_switch_looks(below_zero: bool, switching: u32, mode: u32) -> bool {
    below_zero && switching == 0 && matches!(mode, 0 | 1)
}

/// The switch begun: the mode before (`unk_71A664`, the mode `unk_71A660` + 1), the mode
/// 4, under way (`unk_71A66C`) 1, `unk_71A674` 0, `unk_71A670` 10, and the clock
/// (`unk_71A668`), in the original's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LowSwitch {
    pub before: u32,
    pub mode: u32,
    pub switching: u32,
    pub w674: u32,
    pub w670: u32,
    pub clock: u32,
}

/// The switch from the mode `mode` at `clock`.
pub fn low_switch(mode: u32, clock: u32) -> LowSwitch {
    LowSwitch {
        before: mode.wrapping_add(1),
        mode: 4,
        switching: 1,
        w674: 0,
        w670: 10,
        clock,
    }
}

/// `physics::camera_avoid_ground`: the checkpoint `back` before the car's, when there is
/// one (not below 0, by the sign).
pub fn back_checkpoint(checkpoint: u32, back: u32) -> Option<u32> {
    let n = checkpoint.wrapping_sub(back);
    ((n as i32) >= 0).then_some(n)
}

/// The turn back's first step (`dword_605264`), and the step at or below which it ends.
pub const FIRST_TURN_STEP: u32 = 0x400_0000;
pub const LAST_TURN_STEP: u32 = 0x1000;

/// The gap from the heading to the kept one, made positive (signed).
pub fn turn_gap(kept: u32, heading: u32) -> u32 {
    let d = kept.wrapping_sub(heading);
    if (d as i32) < 0 { d.wrapping_neg() } else { d }
}

/// A turn step toward the kept heading: the step, negated when the kept one is below
/// (signed); None when the gap is below the step (unsigned).
pub fn turn_step(kept: u32, heading: u32, step: u32) -> Option<u32> {
    if turn_gap(kept, heading) < step {
        return None;
    }
    Some(if (kept.wrapping_sub(heading) as i32) < 0 {
        step.wrapping_neg()
    } else {
        step
    })
}

/// The next step: an eighth; None when it is then `LAST_TURN_STEP` or less (the end).
pub fn next_turn_step(step: u32) -> (u32, bool) {
    let s = step >> 3;
    (s, s <= LAST_TURN_STEP)
}

/// The pitch's first limit going down (`dword_608C20`).
pub const PITCH_START: u32 = 0x4000_0000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avoiding() {
        assert_eq!(back_checkpoint(1, 2), None);
        assert_eq!(back_checkpoint(5, 2), Some(3));
        assert_eq!(turn_gap(10, 30), 20);
        assert_eq!(turn_step(10, 30, 20), Some((-20i32) as u32));
        assert_eq!(turn_step(10, 30, 21), None);
        assert_eq!(turn_step(30, 10, 4), Some(4));
        assert_eq!(next_turn_step(FIRST_TURN_STEP), (0x80_0000, false));
        assert_eq!(next_turn_step(0x8000), (0x1000, true));
    }

    #[test]
    fn heights() {
        assert!(is_low(over(0x17FF, 0x1000)));
        assert!(!is_low(over(0x1800, 0x1000)));
        assert!(is_low(over(0x1000, 0x8000_0000)), "by the sign");
        assert!(is_clear(over(0x11000, 0x1000)));
        assert!(!is_clear(over(0x10FFF, 0x1000)));
        assert!(low_switch_looks(true, 0, 1) && !low_switch_looks(true, 0, 2));
        assert!(!low_switch_looks(false, 0, 0) && !low_switch_looks(true, 1, 0));
        assert_eq!(low_switch(3, 9).before, 4);
    }
}
