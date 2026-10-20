//! The service area's readouts (converted 2026-10-20): `screens::draw_repair_times`
//! 0x4A70D7 (sixteen repair times as clocks) and `screens::draw_service_time` 0x4A5DE7 (the
//! time available and left, or the overrun and its penalty).

use crate::screens::screen::{Drawn, Screen};
use crate::strings::format::clock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    /// `stru_51E3E4`: the repair times.
    Times,
    /// `stru_51E584`, `stru_51E5A4`: the time available, the time left.
    Available,
    Remaining,
    /// `stru_51E524`, `stru_51E544`: the overrun, the penalty.
    Overrun,
    Penalty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// `dword_75ACC0`: the penalty.
    Penalty,
    /// `seconds`, `dword_7559EC`: work variables.
    A,
    B,
}

/// The repair times' columns and rows: left column first in each row.
const COLUMNS: [i32; 2] = [0x6A, 0x1B9];
const ROWS: [i32; 8] = [0x6E, 0x8B, 0xA8, 0xC5, 0xE2, 0xFF, 0x11C, 0x1C4];

/// The sixteen times (`dword_75AB60`); what the last text left, and its x.
pub fn repair_times<S: Screen<Font = Font>>(
    s: &mut S,
    times: &[u32; 16],
) -> Result<(Drawn, i32), S::Error> {
    let mut last = (Drawn::default(), 0);
    for (i, &t) in times.iter().enumerate() {
        let (x, y) = (COLUMNS[i % 2], ROWS[i / 2]);
        last = (s.line(Font::Times, &clock(t), x, y)?, x);
    }
    Ok(last)
}

/// The free time the service area gives (`service::state::FREE_SERVICE_TIME`).
pub const FREE_SERVICE_TIME: i32 = 900;
const LEFT_X: i32 = 0x42;
const RIGHT_X: i32 = 0x2DE;
const Y: i32 = 0x2C;

/// "<label>:  <value as a clock>".
fn readout<S: Screen<Font = Font>>(
    s: &mut S,
    (id, font): (u32, Font),
    value: i32,
    x: i32,
) -> Result<bool, S::Error> {
    let mut line = s.string_bytes(id);
    line.extend_from_slice(b":  ");
    line.extend(clock(value as u32));
    Ok(s.line(font, &line, x, Y)?.too_wide)
}

/// The two readouts from the time budget and the repairs' time; the carry of the last
/// text. An overrun of 0 after its 60 is added (only by a wrap) is the original's fatal
/// error 0x80701010 (its division's guard).
pub fn service_time<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    budget: i32,
    repairs: i32,
) -> Result<bool, S::Error> {
    let available = budget.wrapping_sub(FREE_SERVICE_TIME);
    s.keep(Work::Penalty, 0);
    s.keep(Work::A, available as u32);
    if budget.wrapping_sub(repairs) >= FREE_SERVICE_TIME {
        readout(s, (0x48D, Font::Available), available, LEFT_X)?;
        let remaining = available.wrapping_sub(repairs);
        s.keep(Work::B, remaining as u32);
        return readout(s, (0x48E, Font::Remaining), remaining, RIGHT_X);
    }
    let mut overrun = FREE_SERVICE_TIME.wrapping_sub(budget.wrapping_sub(repairs));
    s.keep(Work::B, overrun as u32);
    readout(s, (0x490, Font::Overrun), overrun, LEFT_X)?;
    if overrun != 0 {
        overrun = overrun.wrapping_add(60);
        s.keep(Work::B, overrun as u32);
    }
    if overrun == 0 {
        return Err(s.fatal(0x8070_1010, ""));
    }
    let penalty = (overrun as u32 / 60).wrapping_mul(10);
    s.keep(Work::Penalty, penalty);
    readout(s, (0x491, Font::Penalty), penalty as i32, RIGHT_X)
}
