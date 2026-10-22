//! `logfile::date_text` (0x401D74): writes the date and time the game started at.
//!
//! In:  ebx = cursor
//! Out: ebx = the cursor after the text; everything else preserved
//! "Monday 12th October 2026 14:49:56.46": the day's name, the day with its ending, the
//! month's name, the year, the time with hundredths. From what `sub_410F29` took from
//! `GetLocalTime` at the game's start: `byte_51839F` the day of the week, `dword_5183A0`
//! the day, the month and (a word) the year, `dword_5183A4` hour, minute, second and
//! hundredths. After the text a carriage return, a line feed and two NULs are written,
//! which the cursor is not moved past. One caller, the log's start.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::framework::{Regs, game_read, rva};
use rc2k_core::files::log::Started;

// ---- game data ----

const WEEKDAY_RVA: usize = rva(0x51839F); // byte_51839F, then day, month, year (word),
const DAY_RVA: usize = rva(0x5183A0); // hour, minute, second, hundredths
const MONTH_RVA: usize = rva(0x5183A1);
const YEAR_RVA: usize = rva(0x5183A2);
const TIME_RVA: usize = rva(0x5183A4);
// ---- Rust version ----

/// The text, without what is written after it (converted: `rc2k_core::files::log::date`, on the
/// start's moment as the game keeps it). Refuses a weekday, day or month past the tables.
pub unsafe fn date_text() -> Result<Vec<u8>, BadInput> {
    let started = unsafe {
        let [hour, minute, second, hundredths]: [u8; 4] = game_read(TIME_RVA);
        Started {
            weekday: game_read(WEEKDAY_RVA),
            day: game_read(DAY_RVA),
            month: game_read(MONTH_RVA),
            year: game_read(YEAR_RVA),
            hour,
            minute,
            second,
            hundredths,
        }
    };
    rc2k_core::files::log::date(&started).ok_or(BadInput(
        "a start date whose weekday, day or month is past the name tables",
    ))
}

/// For Rust callers: the text at the cursor, and after it CR, LF and two NULs. Returns the
/// cursor after the text.
pub unsafe fn write_date_text(cursor: *mut u8) -> Result<*mut u8, BadInput> {
    unsafe {
        let text = date_text()?;
        std::ptr::copy(text.as_ptr(), cursor, text.len());
        let end = cursor.add(text.len());
        (end as *mut u32).write_unaligned(0x0A0D);
        Ok(end)
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { date_text() }?;
    r.ebx = unsafe { write_date_text(r.ebx as *mut u8) }.map_or(r.ebx, |end| end as u32);
    Ok(())
}
