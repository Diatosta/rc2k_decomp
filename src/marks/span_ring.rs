//! `marks::span_ring` (0x4BE469): a shape's ring of quads spanned into the mask's runs, its
//! angles kept at esi.
//!
//! In:  ebp the shape (+4 the quads), esi the angles' table
//! Out: eax the spans so far, esi 8 a quad on
//!
//! See `marks/rings.rs`; `sub_4BE592` (`span_ring_quad`) is ported inside it (not hooked:
//! it reads the quad's number from its caller's stack), and the quad it works out is thrown
//! away for the corner table's (section 16). Callers: `sub_4BDE9A` (`marks::make_masks`,
//! the two shapes). Refuses a count of 0 or less or past 0x80, and runs or spans past their
//! room, before writing. Self-test: the original and the Rust version on shapes and corner
//! tables by lot in a work block of the test's, the refusals; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::rings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { span_ring(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
