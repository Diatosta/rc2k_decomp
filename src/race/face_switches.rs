//! `race::face_switches` (0x45322C): `dword_6240E0` / `dword_6240E4` 1 when the face's
//! first / second texture is animated (`dword_63DA2C` / `dword_63DA38`), else 0 (see
//! `race/course_faces.rs`).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! Caller: `sub_452C6F`. Shadow mode: by state. Self-test: the four combinations.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::course_faces::face_switches;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { face_switches() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
