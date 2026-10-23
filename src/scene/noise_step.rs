//! `scene::noise_step` (0x440F4E): one step of the scene's noise (`scene/next_noise.rs`).
//!
//! In:  eax the state
//! Out: eax the next; everything else preserved (edx pushed and popped)
//! Callers: 0x440F2E and `scene::next_noise` (0x440F3E), whose Rust versions use the
//! step themselves. Shadow mode: both, compared.
//! Self-test: the original against the Rust version on a set of states.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, crate::scene::next_noise::noise_step(r.eax)))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
