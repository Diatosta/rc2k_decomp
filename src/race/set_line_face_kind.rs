//! `race::set_line_face_kind` (0x44574A): the kind of the faces the best line runs
//! through, by the stage's direction.
//!
//! In:  nothing
//! Out: ebx = the kind; the rest preserved
//!
//! See `race/best_line.rs`. One caller, `sub_4458A7`. Self-test: the original and the Rust
//! version on the game's variables, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::best_line::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        set_line_face_kind(r)
    }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
