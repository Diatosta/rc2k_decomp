//! `race::copy_course_header` (0x445736): the course memory's first 0x100 bytes to
//! `g_courseHeader`.
//!
//! In:  nothing
//! Out: esi and edi past the bytes, ecx 0; the rest preserved
//!
//! See `race/course_data.rs`. Callers `sub_4455FC` and `sub_459B6A`'s function. Self-test: a
//! course memory of the test's, the header put back afterwards.
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
        copy_course_header(r)
    }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
