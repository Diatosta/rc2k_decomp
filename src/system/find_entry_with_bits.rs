//! `system::find_entry_with_bits` (0x4017AB): the first of ecx 8-byte entries from ebx
//! whose first dword has all of eax's bits.
//!
//! In:  eax the bits, ebx the table, ecx the count
//! Out: found: carry clear, ebx the entry, ecx the entries left with it; not found:
//!      carry set, ebx past the table, ecx 0 (a count of 0: ebx as it came); everything
//!      else preserved (edx is pushed and popped)
//! Callers `jb` on the carry: four, the screen modes' tables (`gfx::mark_3d_modes`,
//! 0x4046B7, 0x40471F, 0x404AE6). Refuses a null table with a count (before anything).
//! Manual patch with a private copy (0x17 bytes): the `jb` at +4 is in the first bytes.
//! Shadow mode: both, compared. Self-test: the original against the Rust version on a
//! table of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EBX, ECX, Regs};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- Rust version ----

/// (found, ebx, ecx).
pub fn find_entry_with_bits(bits: u32, table: u32, count: u32) -> (bool, u32, u32) {
    // Converted: `rc2k_core::system::tables::with_bits`.
    use rc2k_core::system::tables::{left, with_bits};
    let found = with_bits(
        unsafe { crate::system::find_entry::entries(table, count) },
        bits,
    );
    let (at, n) = left(table, count, found);
    (found.is_some(), at, n)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.ebx == 0 && r.ecx != 0 {
        return Err(BadInput("a null table").into());
    }
    let (found, ebx, ecx) = find_entry_with_bits(r.eax, r.ebx, r.ecx);
    let left = Left::nothing().with(EBX, ebx).with(ECX, ecx);
    Ok(Left {
        carry: Some(!found),
        // found: `cmp edx, eax` equal; not: the `dec ecx` that reached 0, or `cmp ecx, 1` on 0
        zero: Some(found || r.ecx != 0),
        ..left
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
