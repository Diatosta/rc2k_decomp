//! `scene::find_object_kind` (0x41F4BD): an object kind's place in the known kinds
//! (`g_knownObjectKinds`, 0x22 dwords): the first whose value, or value + 1, is eax;
//! 0x22 for none.
//!
//! In:  eax the kind
//! Out: eax the place; everything else preserved
//! Callers: 0x425E3A (`scene::load_object`, whose Rust version has its own) and
//! 0x4522A7. Shadow mode: both, compared.
//! Self-test: the original against the Rust version for each known kind, each + 1, and
//! some unknown.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs, game_read, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// g_knownObjectKinds.
const KINDS_RVA: usize = rva(0x5C5790);

// ---- Rust version ----

pub fn find_object_kind(kind: u32) -> u32 {
    // Converted: `rc2k_core::scene::kinds::find_known`.
    rc2k_core::scene::kinds::find_known(|n| unsafe { game_read(KINDS_RVA + 4 * n) }, kind)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, find_object_kind(r.eax)))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
