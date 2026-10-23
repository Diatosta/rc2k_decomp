//! `marks::map_pixel` (0x4BEFD2): pixel ecx of row `dword_833DB6` placed against the line
//! pair and kept as four bytes at `[ebp + 4 ecx]`.
//!
//! In:  ecx the pixel, ebp the table's row; `dword_833DB6` the row number
//! Out: every register kept
//!
//! See `marks/pixels.rs`. One caller, `marks::map_shape_rows`. Self-test: the original and
//! the Rust version on line pairs, pixels and seeds by lot (both of the shake's branches);
//! put back afterwards.
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
    Ok(left_of(&unsafe { map_pixel(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
