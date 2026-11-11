//! `system::keep_start_time` (0x410F29): keeps the date and time the game was started at.
//!
//! `GetLocalTime` into the game's `SystemTime`, and from it the weekday, then the day, month
//! and year (a word), then the hour, minute, second and hundredths are kept (nine bytes):
//! what `logfile::date_text` and `math::seed_random` read. One
//! caller, `start`, as its first call.

// ---- game data ----

/// The nine bytes kept (the image's three variables at 0x51839F). Written once, by
/// `start`'s first call.
pub const KEPT_LEN: usize = 9;

/// The start's moment as kept (all 0 before `keep_start_time`).
pub fn start_time() -> [u8; KEPT_LEN] {
    let system = crate::system::owner::system();
    system.start.kept.read()
}

/// The nine bytes kept: weekday, day, month, year (two), hour, minute, second,
/// hundredths.
pub fn kept(t: &rc2k_core::system::time::LocalTime) -> [u8; KEPT_LEN] {
    // Converted: `rc2k_core::system::time::start_time`.
    rc2k_core::system::time::start_time(t)
}

pub fn keep_start_time(system: &crate::system::owner::System) {
    // The game's `SystemTime` (0x518988), a local here: nothing else read it.
    let time = crate::os::local_time();
    system.start.kept.write(&kept(&time));
}
