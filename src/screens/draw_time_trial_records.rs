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

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::screens::draw_stage_records::{TIME_TRIAL_ROWS, dispatch_rows, rows_entry, test_rows};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_time_trial_records",
    rva(0x48DE45),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_rows(&HOOK, r, call_original, &TIME_TRIAL_ROWS);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    rows_entry("screens::draw_time_trial_records", r, &TIME_TRIAL_ROWS)
}

fn self_test() -> (usize, usize) {
    test_rows(&HOOK, call_original, &TIME_TRIAL_ROWS)
}
