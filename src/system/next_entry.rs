//! `system::next_entry` (0x4017E5): one step on in a table of 8-byte entries.
//!
//! In:  ebx the entry, ecx the entries left with it
//! Out: ecx 0: carry set, nothing changed. Else ebx + 8 and ecx - 1; carry set if that
//!      leaves 0, else the carry of the `add ebx, 8` (`dec` keeps it: clear unless ebx
//!      wrapped). Everything else preserved.
//! Callers: `gfx::mark_3d_modes` (which does not read the carry) and 0x404718. Manual patch
//! with a private copy (0xD bytes): the `jb` at +3 is in the first bytes. Shadow mode:
//! both, compared. Self-test: the original against the Rust version on counts and places
//! (one that wraps).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EBX, ECX, Regs};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- Rust version ----

/// (carry, ebx, ecx).
pub fn next_entry(entry: u32, left: u32) -> (bool, u32, u32) {
    // Converted: `rc2k_core::system::tables::next`.
    rc2k_core::system::tables::next(entry, left)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (carry, ebx, ecx) = next_entry(r.ebx, r.ecx);
    let left = Left::nothing().with(EBX, ebx).with(ECX, ecx);
    Ok(Left {
        carry: Some(carry),
        // `dec ecx`'s: set when it reached 0; with none left at the start `cmp ecx, 1`'s
        // (clear). Not the carry: an entry that wraps sets that with entries left.
        zero: Some(r.ecx != 0 && ecx == 0),
        ..left
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
