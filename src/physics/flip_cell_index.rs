//! `physics::flip_cell_index` (0x43DB13): a 2 x 2 cell (ebx, edx, each 0 or 1) turned by
//! one of eight ways (eax & 7: 1 flips ebx, 2 flips edx, 3 both; 4 .. 7 swap the two
//! first, then 4 flips edx, 5 both, 6 none, 7 ebx), then edx = 2 edx + ebx.
//!
//! In:  eax the way, ebx and edx the cell
//! Out: eax & 7, ebx and edx as worked out; the rest kept; no flags read (callers store
//!      edx)
//!
//! Caller: `sub_43D59A` (twice).
//! Self-test: a copy of the original against the core, every way with cells 0 .. 3 and
//! odd values.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, EDX, Regs};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version: (eax, ebx, edx) out.
// Converted: `rc2k_core::physics::ground::flip_cell_index`.
pub use rc2k_core::physics::ground::flip_cell_index;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (a, b, d) = flip_cell_index(r.eax, r.ebx, r.edx);
    Ok(Left::nothing().with(EAX, a).with(EBX, b).with(EDX, d))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
