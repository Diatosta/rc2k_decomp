//! Learning a control on the controller page.
//!
//! A control is learnt for up to 3 s: a wheel's pedals (device kind 3, controls 1 and 2) by the
//! axis that moved, any other control by the controller's reading. Space (0x39) cancels. The
//! result goes into the player's bindings.

/// The time a control is learnt for, in ms.
pub const LEARNING_MS: u32 = 0xBB8;
/// The key that cancels.
pub const CANCEL_KEY: u8 = 0x39;
/// The control set when the time runs out (none).
pub const TIMED_OUT: u32 = u32::MAX;
/// A player's device kind that learns its pedals by axis.
pub const PEDALS_KIND: u32 = 3;

/// Whether the learning's time is up: `since` (the clock less its start) at least 3 s.
pub fn timed_out(since: u32) -> bool {
    since.wrapping_sub(LEARNING_MS) as i32 >= 0
}

/// Whether the control is a pedal learnt by its axis.
pub fn by_axis(kind: u32, control: u32) -> bool {
    kind == PEDALS_KIND && (control == 1 || control == 2)
}

/// A pedal's axis learnt: the player's `axis_codes` byte it goes to (+0x3B, +0x3A) and the
/// found code.
pub fn pedal(control: u32) -> (usize, u32) {
    if control == 1 { (3, 0x200) } else { (2, 0x100) }
}

/// The player's binding of control 1 .. 11 (+0x50 + 4 (control - 1)). None for another.
pub fn binding(control: u32) -> Option<usize> {
    (1..=11).contains(&control).then(|| control as usize - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules() {
        assert!(!timed_out(0xBB7) && timed_out(0xBB8) && !timed_out(0xFFFF_FFFF));
        assert!(by_axis(3, 2) && !by_axis(3, 3) && !by_axis(2, 1));
        assert_eq!(pedal(1), (3, 0x200));
        assert_eq!(pedal(2), (2, 0x100));
        assert_eq!(binding(1), Some(0));
        assert_eq!(binding(11), Some(10));
        assert_eq!((binding(0), binding(12)), (None, None));
    }
}
