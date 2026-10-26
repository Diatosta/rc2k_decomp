//! `drive::unpack_inputs` (0x409338): a packed input word (`drive::pack_inputs`) taken
//! apart: eax bits 4 .. 12, edx bits 13 .. 21, ecx the top ten (`sar` 22).
//!
//! In:  eax the packed word
//! Out: eax, ecx, edx the words; the rest kept; no flags read
//!
//! Callers: `drive::play_inputs`, `drive::record_inputs`.
//! Self-test: the original and the Rust version on words by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, EDX, Regs};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version: [eax, ecx, edx].
// Converted: `rc2k_core::drive::inputs::unpack_inputs`.
pub use rc2k_core::drive::inputs::unpack_inputs;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let [eax, ecx, edx] = unpack_inputs(r.eax);
    Ok(Left::nothing().with(EAX, eax).with(ECX, ecx).with(EDX, edx))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
