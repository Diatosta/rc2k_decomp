//! `marks::map_shape_rows` (0x4BE855): entry esi of the rows at edi mapped pixel by pixel into the table.
//!
//! In:  esi the entry, edi the entries
//! Out: every register kept
//!
//! See `marks/pixels.rs`. Callers: `marks::map_shape` (twice). Refuses rows or spans outside the 0x100 x 0x80
//! table, before writing. Self-test: the original and the Rust version on shapes, line
//! pairs and seeds by lot with a table of the test's, the refusal; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::pixels::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { map_shape_rows(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
