//! `logfile::date_text` (0x401D74): writes the date and time the game started at.
//!
//! Takes cursor. Returns the cursor after the text. "Monday 12th October 2026 14:49:56.46": the
//! day's name, the day with its ending, the month's name, the year, the time with hundredths. From
//! what `system::keep_start_time` took from `GetLocalTime` at the game's start: `byte_51839F` the
//! day of the week, `dword_5183A0` the day, the month and (a word) the year, `dword_5183A4` hour,
//! minute, second and hundredths. After the text a carriage return, a line feed and two NULs are
//! written, which the cursor is not moved past. One caller, the log's start.

use crate::errors::BadInput;
use rc2k_core::files::log::Started;

// ---- Rust version ----

/// The text, without what is written after it (converted: `rc2k_core::files::log::date`, on the
/// start's moment as the game keeps it). Refuses a weekday, day or month past the tables.
pub fn date_text() -> Result<Vec<u8>, BadInput> {
    // byte_51839F the weekday, then day, month, year (word), hour, minute, second,
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
    ] = crate::system::keep_start_time::start_time();
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

/// For Rust callers: the text at the cursor, and after it CR, LF and two NULs. Returns the
/// text with the CR and LF (what a read up to the first NUL finds there).
pub fn write_date_text(at: &dyn crate::pointers::Store) -> Result<Vec<u8>, BadInput> {
    let text = date_text()?;
    at.store(0, &text)?;
    crate::pointers::store_value(at, text.len(), 0x0A0Du32)?;
    let mut line = text;
    line.extend_from_slice(b"\r\n");
    Ok(line)
}
