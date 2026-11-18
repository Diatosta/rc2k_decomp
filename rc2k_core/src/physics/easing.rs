//! Whole-number values eased towards a target each physics step.

/// Adds half the way (a signed halving) from `value` up to `target`; returns the step. A value
/// already past the target eases back towards it.
pub fn ease_up(value: &mut u32, target: u32) -> u32 {
    let step = (target.wrapping_sub(*value) as i32 >> 1) as u32;
    *value = value.wrapping_add(step);
    step
}

/// Takes half the way (a signed halving) from `value` down to `target`; returns the step. A
/// value already past the target eases back towards it.
pub fn ease_down(value: &mut u32, target: u32) -> u32 {
    let step = (value.wrapping_sub(target) as i32 >> 1) as u32;
    *value = value.wrapping_sub(step);
    step
}

/// Unless `blocked`, `value` follows `target` << 8: set to it when `jump` is set, else stepped
/// by 4 towards it. Returns the goal, None when blocked.
pub fn ease_value(blocked: u32, jump: u32, target: u32, value: &mut u32) -> Option<u32> {
    if blocked != 0 {
        return None;
    }
    let goal = target << 8;
    if jump != 0 {
        *value = goal;
    } else if goal != *value {
        let step = if (goal.wrapping_sub(*value) as i32) >= 0 {
            4u32
        } else {
            4u32.wrapping_neg()
        };
        *value = value.wrapping_add(step);
    }
    Some(goal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halves() {
        let mut v = 10;
        assert_eq!(ease_up(&mut v, 20), 5);
        assert_eq!(v, 15);
        assert_eq!(ease_down(&mut v, 4), 5);
        assert_eq!(v, 10);
        // Past the target the step turns back (an unsigned halving jumped by about 2^31).
        let mut w = 20;
        assert_eq!(ease_up(&mut w, 10), (-5i32) as u32);
        assert_eq!(w, 15);
        let mut x = 4;
        assert_eq!(ease_down(&mut x, 10), (-3i32) as u32);
        assert_eq!(x, 7);
        // Edge: a difference of 0x80000000 and 0x7FFFFFFF.
        let mut y = 0;
        assert_eq!(ease_up(&mut y, 0x8000_0000), 0xC000_0000);
        let mut z = 0;
        assert_eq!(ease_up(&mut z, 0x7FFF_FFFF), 0x3FFF_FFFF);
    }

    #[test]
    fn follows() {
        let mut v = 0;
        assert_eq!(ease_value(1, 0, 1, &mut v), None);
        assert_eq!(ease_value(0, 0, 1, &mut v), Some(0x100));
        assert_eq!(v, 4);
        assert_eq!(ease_value(0, 1, 1, &mut v), Some(0x100));
        assert_eq!(v, 0x100);
        assert_eq!(ease_value(0, 0, 0, &mut v), Some(0));
        assert_eq!(v, 0xFC);
    }
}
