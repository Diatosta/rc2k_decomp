//! `scene::approximate_distance` (0x44EB78): how long a line is, near enough, without a
//! square root.
//!
//! In:  eax, ebx, ecx = the three differences
//! Out: eax = the length; ebx and ecx = the two smaller parts' shares of it;
//!      everything else preserved
//!
//! Each difference over 256 (signed) and without its sign; the largest times 0xF2, the
//! middle one times 0x60 and the smallest times 0x4B, added.
//! Callers: `scene::add_point_lights` and 0x44F8C7.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, Regs};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- Rust version ----

/// Returns the length and what is left in ebx and ecx. Converted:
/// `rc2k_core::scene::distance::approximate`.
pub fn approximate_distance(differences: [u32; 3]) -> [u32; 3] {
    let d = rc2k_core::scene::distance::approximate(differences.map(|d| d as i32));
    [d.length, d.middle, d.least]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let [length, middle, least] = approximate_distance([r.eax, r.ebx, r.ecx]);
    Ok(Left::nothing()
        .with(EAX, length)
        .with(EBX, middle)
        .with(ECX, least))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
