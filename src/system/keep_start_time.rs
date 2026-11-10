//! `system::keep_start_time` (0x410F29): keeps the date and time the game was started at.
//!
//! `GetLocalTime` into the game's `SystemTime` (0x518988), and from it the weekday to
//! `byte_51839F`, day, month and year (a word) to `dword_5183A0`, hour, minute, second and
//! hundredths to `dword_5183A4`: what `logfile::date_text` and `math::seed_random` read. One
//! caller, `start`, as its first call.

// ---- game data ----

/// The nine bytes kept: out of the image since 2026-10-24 (they were `byte_51839F`,
/// `dword_5183A0`, `dword_5183A4`). Written once, by `start`'s first call.
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
