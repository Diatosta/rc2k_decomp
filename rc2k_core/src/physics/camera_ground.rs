//! The camera's ground checks: its height over the ground, the low-camera switch, the points
//! across its heading that must have ground, and the turn / pitch steps that keep it out of
//! the ground.

/// Below this height (signed) the camera is low.
pub const LOW: u32 = 0x800;
/// At this height (signed) or more the camera is clear.
pub const CLEAR: u32 = 0x10000;
/// The x offsets across the heading (half a unit either way) that must have ground.
pub const SIDES: [u32; 2] = [0xFFFF_8000, 0x8000];

/// The camera's height over the ground (wrapping).
pub fn over(camera_y: u32, ground: u32) -> u32 {
    camera_y.wrapping_sub(ground)
}

pub fn is_low(gap: u32) -> bool {
    (gap.wrapping_sub(LOW) as i32) < 0
}

pub fn is_clear(gap: u32) -> bool {
    (gap.wrapping_sub(CLEAR) as i32) >= 0
}

/// Whether the low-camera switch looks: below 0, no switch under way, mode 0 or 1.
pub fn low_switch_looks(below_zero: bool, switching: u32, mode: u32) -> bool {
    below_zero && switching == 0 && matches!(mode, 0 | 1)
}

/// The state a low-camera switch sets: `before` the mode to return to, mode 4, under way,
/// `w674` the share 0, `w670` the length 10, and the clock at its start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LowSwitch {
    pub before: u32,
    pub mode: u32,
    pub switching: u32,
    pub w674: u32,
    pub w670: u32,
    pub clock: u32,
}

/// The switch from the mode `mode` at `clock`; `before` is the mode + 1.
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

/// The checkpoint `back` before the car's, when it is not below 0 (by sign).
pub fn back_checkpoint(checkpoint: u32, back: u32) -> Option<u32> {
    let n = checkpoint.wrapping_sub(back);
    ((n as i32) >= 0).then_some(n)
}

/// The turn back's first step, and the step at or below which it ends.
pub const FIRST_TURN_STEP: u32 = 0x400_0000;
pub const LAST_TURN_STEP: u32 = 0x1000;

/// The gap from the heading to the kept one, made positive (signed).
pub fn turn_gap(kept: u32, heading: u32) -> u32 {
    let d = kept.wrapping_sub(heading);
    if (d as i32) < 0 { d.wrapping_neg() } else { d }
}

/// A turn step toward the kept heading, negated when the kept one is below (signed); None
/// when the gap is below the step (unsigned).
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

/// The next step (an eighth), and whether it is the last (`LAST_TURN_STEP` or less).
pub fn next_turn_step(step: u32) -> (u32, bool) {
    let s = step >> 3;
    (s, s <= LAST_TURN_STEP)
}

/// The pitch's first limit going down.
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
