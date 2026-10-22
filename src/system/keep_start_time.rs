//! `system::keep_start_time` (0x410F29): keeps the date and time the game was started at.
//!
//! In:  nothing
//! Out: every register preserved
//! `GetLocalTime` into the game's `SystemTime` (0x518988), and from it the weekday to
//! `byte_51839F`, day, month and year (a word) to `dword_5183A0`, hour, minute, second
//! and hundredths to `dword_5183A4`: what `logfile::date_text` and `math::seed_random`
//! read. One caller, `start`, as its first call: the hook DLL is not loaded then, so in the
//! game this never gets a call; the standalone program needs it.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{Regs, game_ptr, game_write, rva};
use windows_sys::Win32::Foundation::SYSTEMTIME;
use windows_sys::Win32::System::SystemInformation::GetLocalTime;

// ---- game data ----

const SYSTEM_TIME_RVA: usize = rva(0x518988); // SystemTime
const KEPT_RVA: usize = rva(0x51839F); // byte_51839F, dword_5183A0, dword_5183A4
const KEPT_LEN: usize = 9;

// ---- Rust version ----

/// The nine bytes kept: weekday, day, month, year (two), hour, minute, second,
/// hundredths.
pub fn kept(t: &SYSTEMTIME) -> [u8; KEPT_LEN] {
    // Converted: `rc2k_core::system::time::start_time`.
    rc2k_core::system::time::start_time(&rc2k_core::system::time::LocalTime {
        year: t.wYear,
        month: t.wMonth,
        weekday: t.wDayOfWeek,
        day: t.wDay,
        hour: t.wHour,
        minute: t.wMinute,
        second: t.wSecond,
        milliseconds: t.wMilliseconds,
    })
}

pub unsafe fn keep_start_time() {
    unsafe {
        let time = game_ptr::<SYSTEMTIME>(SYSTEM_TIME_RVA);
        GetLocalTime(time);
        game_write(KEPT_RVA, kept(&time.read_unaligned()));
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { keep_start_time() };
    Ok(())
}
