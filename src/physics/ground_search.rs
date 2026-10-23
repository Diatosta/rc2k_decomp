//! `physics::ground_search` (0x441E2F): the course face under the point (`dword_639824`,
//! `dword_63982C`): `physics::try_last_face`, else `dword_5CCC74` = -1 and
//! `physics::search_cell` on the point's cell and its eight neighbours (`ground::CELLS`,
//! in the original's order) until one has it. `g_groundFound` (0x639830) 1 when found,
//! else 0. All inside `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; CF the found (no caller reads it: they read
//!      `g_groundFound`, `dword_5CCC74`, `dword_639828`)
//!
//! See `physics/ground.rs`. Caller: `physics::find_ground` (0x441E29), its only one.
//! Refuses (before anything) what `find_course_record` or `search_record_faces` would
//! refuse on any of the nine cells (also on cells past the one found).
//! Self-test: a copy of the original (its calls to copies) on the test's ground, records
//! for six cells, the test's points with last faces -1 and two real ones; found counted.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::ground::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version: the carry (the registers are kept).
pub unsafe fn ground_search() -> Result<bool, Refused> {
    let mut r = Regs::default();
    unsafe { ground_search_at(&Ground::game(), &mut r) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { ground_search() }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
