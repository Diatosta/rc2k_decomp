//! `marks::fill_mask_spans` (0x4BE712): one entry's rows of spans filled into the mask with
//! the shape's number.
//!
//! In:  esi the entry, edi the entries
//! Out: every register kept
//!
//! See `marks/masks.rs`. One caller, `marks::draw_mask_shape` (twice). Refuses a span
//! outside the mask, before writing. Self-test: the original and the Rust version on
//! shapes and a mask of the test's, the refusal; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::masks::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { fill_mask_spans(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
