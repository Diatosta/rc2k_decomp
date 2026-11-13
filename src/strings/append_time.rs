//! Appends a time given in hundredths of a second, as "MM:SS.cc" or "HH:MM:SS.cc". The hours field
//! is written from one hour upwards, or always when the show-hours setting is 1.

/// 1 = always write the hours (0 at the start). The network's lobby sets it for a while.
pub static SHOW_HOURS: crate::session::GameThread<crate::session::PlainDword> =
    crate::session::GameThread::new(crate::session::PlainDword::new(0));

pub fn show_hours() -> bool {
    SHOW_HOURS.get().get() == 1
}

pub fn append_time(hundredths: u32, show_hours: bool) -> Vec<u8> {
    rc2k_core::strings::format::time(hundredths, show_hours)
}
