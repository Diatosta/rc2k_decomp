//! `marks::draw_mask` (0x4BE6C2): mask esi made 0xFF and each of ebp's shapes drawn into it
//! with its number.
//!
//! In:  esi the mask (0x8000 bytes), ebp the shapes
//! Out: eax the count, ecx 0, esi and edi as the last shape leaves them
//!
//! See `marks/masks.rs`. One caller, `sub_4BDE9A` (the two masks). Refuses a count of 0 or
//! less (the original's loop would run round) or a span outside the mask, before writing.
//! Self-test: the original and the Rust version on shapes and a mask of the test's (1, 2,
//! 5 shapes), the refusals; put back afterwards.
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
    Ok(left_of(&unsafe { draw_mask(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
