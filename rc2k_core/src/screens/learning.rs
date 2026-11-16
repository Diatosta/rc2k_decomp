//! Learning a control on the controller page.
//!
//! A control is learnt for up to 5 s; the first thing that moves from where the controller
//! rested (`rc2k_core::input::profile::detect`) is the answer, and when the time runs out or
//! space (0x39) cancels, the control stays as it was.

/// The time a control is learnt for, in ms.
pub const LEARNING_MS: u32 = 5000;
/// The key that cancels.
pub const CANCEL_KEY: u8 = 0x39;
/// The control set when the time runs out (none).
pub const TIMED_OUT: u32 = u32::MAX;

/// Whether the learning's time is up: `since` (the clock less its start) at least 5 s.
pub fn timed_out(since: u32) -> bool {
    since.wrapping_sub(LEARNING_MS) as i32 >= 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules() {
        assert!(!timed_out(4999) && timed_out(5000) && !timed_out(0xFFFF_FFFF));
    }
}
