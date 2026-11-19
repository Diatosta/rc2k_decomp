//! `math::seed_random`: how the random numbers start.
//!
//! One step of the generator on the start's time (hour, minute, second, hundredths) with
//! its date (day, month, year) mixed in; unless that is 0, it is turned left by the day of
//! the week plus 8 and becomes the state. Run right after the start time is taken.

// ---- game data ----

/// Seeds the generator and its streams from the nine bytes of the start time (weekday, day,
/// month, year (two), hour, minute, second, hundredths).
pub fn seed_random(random: &crate::math::next_random::Randoms, kept: [u8; 9]) {
    let clock = rc2k_core::math::random::StartClock {
        time: u32::from_le_bytes([kept[5], kept[6], kept[7], kept[8]]),
        date: u32::from_le_bytes([kept[1], kept[2], kept[3], kept[4]]),
        weekday: kept[0],
    };
    random.with_random(|r| r.seed(clock));
    random.seed_streams();
}
