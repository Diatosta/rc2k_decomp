//! Writes the date and time the game started at, as "Monday 12th October 2026 14:49:56.46" (the
//! day's name, the day with its ending, the month's name, the year, the time with hundredths).
//! After the text a carriage return, a line feed and two NULs are written, which the cursor is
//! not moved past.

use crate::errors::BadInput;
use rc2k_core::files::log::Started;

/// The text, without what is written after it (converted: `rc2k_core::files::log::date`, on the
/// start's moment as the game keeps it). Refuses a weekday, day or month past the tables.
pub fn date_text(system: &crate::system::owner::System) -> Result<Vec<u8>, BadInput> {
    // The weekday, then day, month, year (word), hour, minute, second,
    // hundredths (`system::keep_start_time`).
    let [
        weekday,
        day,
        month,
        year_low,
        year_high,
        hour,
        minute,
        second,
        hundredths,
    ] = crate::system::keep_start_time::start_time(system);
    let started = Started {
        weekday,
        day,
        month,
        year: u16::from_le_bytes([year_low, year_high]),
        hour,
        minute,
        second,
        hundredths,
    };
    rc2k_core::files::log::date(&started).ok_or(BadInput(
        "a start date whose weekday, day or month is past the name tables",
    ))
}

/// The text at the cursor, and after it CR, LF and two NULs. Returns the
/// text with the CR and LF (what a read up to the first NUL finds there).
pub fn write_date_text(
    system: &crate::system::owner::System,
    at: &dyn crate::cells::Store,
) -> Result<Vec<u8>, BadInput> {
    let text = date_text(system)?;
    at.store(0, &text)?;
    crate::cells::store_value(at, text.len(), 0x0A0Du32)?;
    let mut line = text;
    line.extend_from_slice(b"\r\n");
    Ok(line)
}
