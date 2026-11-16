//! Appends a time given in hundredths of a second, as "MM:SS.cc" or "HH:MM:SS.cc". The hours field
//! is written from one hour upwards, or always when the show-hours setting is 1.

/// Whether the hours are always written (the setting is 1). The network's lobby sets it for a
/// while.
pub fn show_hours(strings: &crate::strings::owner::Strings) -> bool {
    strings.text.show_hours.get() == 1
}

pub fn append_time(hundredths: u32, show_hours: bool) -> Vec<u8> {
    rc2k_core::strings::format::time(hundredths, show_hours)
}
