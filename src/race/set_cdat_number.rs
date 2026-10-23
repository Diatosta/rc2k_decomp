//! `race::set_cdat_number` (0x4456DD): al's two digits into the course data's two paths.
//!
//! In:  al = the number
//! Out: eax the tens' digit, edx the units', ebx 10; the rest preserved
//!
//! See `race/course_data.rs`. Callers `sub_4455FC` (with 0) and `race::find_present_courses`. Self-test:
//! the original and the Rust version on the game's paths, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::course_data::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        set_cdat_number(r)
    }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
