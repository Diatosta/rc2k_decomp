//! `strings::append_time` (0x401A39): appends a time given in hundredths of a second, as
//! "MM:SS.cc" or "HH:MM:SS.cc".
//!
//! Takes hundredths of a second, cursor. Returns new cursor.
//! The hours field is written from one hour upwards, or always when `g_timeShowHours` is 1.

// ---- game data ----

/// 1 = always write the hours (`g_timeShowHours`, 0x4E0148, 0 at the start). The network's lobby sets it for a while.
pub static SHOW_HOURS: crate::session::GameThread<crate::session::PlainDword> =
    crate::session::GameThread::new(crate::session::PlainDword::new(0));

pub fn show_hours() -> bool {
    SHOW_HOURS.get().get() == 1
}

pub fn append_time(hundredths: u32, show_hours: bool) -> Vec<u8> {
    rc2k_core::strings::format::time(hundredths, show_hours)
}
