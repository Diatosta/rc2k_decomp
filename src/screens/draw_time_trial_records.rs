//! `screens::draw_time_trial_records` (0x48DE45): the time trial's two record rows: a copy
//! of `screens::draw_stage_records` whose top-speed row reads the selected record's +0x2C
//! speed, +0x7C name, +0x9C nationality and +0xA0 car, and draws its car's sprite at the
//! first row's y (0x1FC, sic: `notes/bugs.md`); a dead `mov eax, 44h; add eax, [edi+10h]`
//! left out.
//!
//! In:  edi = the best-time entry
//! Out: carry as the last `draw_text` leaves it; every register preserved
//!
//! Caller: `screens::draw_time_trial_stage_select`. See `screens::draw_stage_records` for
//! the rest: the same Rust version (`draw_records_rows` with `TIME_TRIAL_ROWS`), dispatch
//! and self-test.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::screens::draw_stage_records::{TIME_TRIAL_ROWS, rows_entry};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    rows_entry("screens::draw_time_trial_records", r, &TIME_TRIAL_ROWS)
}
