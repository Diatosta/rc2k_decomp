//! `race::group_records_by_kind` (0x421E42): the course's records put in order of kind,
//! each kind's first and last noted.
//!
//! In:  nothing
//! Out: ecx 0x100, eax, ebx, edx, esi, edi, ebp as `group_records_by_kind` says
//!
//! See `race/course_data.rs`. One caller, `sub_4394CB` (locacurrcourse). Refuses more than
//! 0x78000 records. Self-test: records of the test's in the game's pointer and count, the
//! two tables put back afterwards.
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
        group_records_by_kind(r)
    }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
