//! `race::reset_face_lights` (0x45ED70): the lights of the faces marked 0xFF cleared, then
//! `race::clear_lit_faces`.
//!
//! In:  nothing
//! Out: ebx the face count, esi the last face; the rest preserved
//!
//! See `race/course_data.rs`. Callers `sub_4455FC` and `sub_4462B9` (race init). Refuses no
//! faces or 0x100000 or more. Self-test: faces of the test's in the game's pointer and
//! count, put back afterwards.
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
        reset_face_lights(r)
    }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
