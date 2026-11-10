//! `math::seed_random` (0x401864): how the random numbers start.
//!
//! One step of the generator on the start's time (`dword_5183A4`: hour, minute, second,
//! hundredths) with its date mixed in (`dword_5183A0`: day, month, year); unless that is
//! 0, it is turned left by the day of the week (`byte_51839F`) plus 8 and becomes the
//! state. One caller, `start`, right after the time is taken.

// ---- game data ----

pub fn seed_random() {
    crate::math::next_random::wait_for_random();
    // `byte_51839F`, `dword_5183A0`, `dword_5183A4` (`system::keep_start_time`).
    let kept = crate::system::keep_start_time::start_time();
    let clock = rc2k_core::math::random::StartClock {
        time: u32::from_le_bytes([kept[5], kept[6], kept[7], kept[8]]),
        date: u32::from_le_bytes([kept[1], kept[2], kept[3], kept[4]]),
        weekday: kept[0],
    };
    crate::math::next_random::with_random(|r| r.seed(clock));
    crate::math::next_random::seed_streams();
}
