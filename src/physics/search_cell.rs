//! `physics::search_cell` (0x441FBA): the grid cell of (eax, ecx) (each >> 21; key =
//! (z cell << 11) + x cell), its course record by `physics::find_course_record`, then its
//! faces by `physics::search_record_faces`. CF set when a face has the point; clear when
//! not, or when the cell has no record.
//!
//! In:  eax x, ecx z (whole numbers, the point's plus or less a cell)
//! Out: CF; ecx the z cell << 11; the rest as `find_course_record` and (when a record is
//!      found) `search_record_faces` leave them
//!
//! See `physics/ground.rs`. Caller: `sub_441E2F` (nine cells around the point; `jb`).
//! Refuses what `find_course_record` and `search_record_faces` refuse (both before
//! anything).
//! Self-test: a copy of the original (its calls to copies of the two) on the test's
//! ground with records for six cells of the grid, the test's points and their
//! neighbours.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::ground::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version: `r` as the original leaves it, and the carry.
pub unsafe fn search_cell(r: &mut Regs) -> Result<bool, Refused> {
    unsafe { search_cell_at(&Ground::game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    let carry = unsafe { search_cell(&mut g) }?;
    Ok(left_of(&g, carry))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
