//! `race::read_stage_car_numbers` (0x437971): the stage's line of the cars' data text
//! ('l' or 'L' and the stage's two digits) and its three-digit numbers, four apart, into
//! `g_stageCarNumbers` (0x601430) and their sum into `g_stageCarTotal` (0x601C4C).
//!
//! In:  nothing
//! Out: ecx = the place reached, ebp = the count, eax = the last number parsed (0 for a
//!      failed one; the entry's when no line was found); the rest preserved
//!
//! See `race/race_cars.rs`. One caller, `sub_43603C` (modes 0 and 2). Refuses more than
//! 0x100 numbers (worked out first). Shadow mode: the Rust version, the variables put
//! back, the original's, compared. Self-test: texts of the test's in the game's buffer
//! (stage found and not, lower case, a stage of 0 against a line without digits, numbers
//! to the text's end), all put back; the refusal on the Rust version alone.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::race_cars::*;
use crate::race::start_place::left_changed;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { read_stage_car_numbers(r) }?;
    Ok(left_changed(r, &g))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
