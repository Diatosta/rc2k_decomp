//! Keeps the date and time the game was started at.
//!
//! `GetLocalTime` into the game's `SystemTime`, and from it the weekday, then the day, month and
//! year (a word), then the hour, minute, second and hundredths are kept (nine bytes): what
//! `logfile::date_text` and `math::seed_random` read.

/// The nine bytes kept. Written once, by `start`'s first call.
pub(super) const KEPT_LEN: usize = 9;

/// The start's moment as kept (all 0 before `keep_start_time`).
pub fn start_time(system: &crate::system::owner::System) -> [u8; KEPT_LEN] {
    system.start().kept.read()
}

/// The nine bytes kept: weekday, day, month, year (two), hour, minute, second,
/// hundredths.
pub fn kept(t: &rc2k_core::system::time::LocalTime) -> [u8; KEPT_LEN] {
    rc2k_core::system::time::start_time(t)
}

pub fn keep_start_time(system: &crate::system::owner::System) {
    // The game's `SystemTime`, a local here: nothing else read it.
    let time = crate::platform::os::local_time();
    system.start().kept.write(&kept(&time));
}
