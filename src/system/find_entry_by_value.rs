//! `system::find_entry_by_value` (0x4017D3): the first of ecx 8-byte entries from ebx
//! whose second dword is eax.
//!
//! In:  eax the value, ebx the table, ecx the count
//! Out: found: carry clear, ebx the entry, ecx the entries left with it; not found:
//!      carry set, ebx past the table, ecx 0 (a count of 0: ebx as it came); everything
//!      else preserved
//! Callers `jb` on the carry: twelve, the DirectDraw / DirectInput set-up's tables
//! (0x404xxx, 0x405C40, 0x40B31F, 0x41A6xx, 0x41B3xx). Refuses a null table with a
//! count (before anything). Shadow mode: both, compared.
//! Self-test: the original against the Rust version on a table of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EBX, ECX, Regs};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- Rust version ----

/// (found, ebx, ecx).
pub fn find_entry_by_value(value: u32, table: u32, count: u32) -> (bool, u32, u32) {
    // Converted: `rc2k_core::system::tables::by_value`.
    use rc2k_core::system::tables::{by_value, left};
    let found = by_value(
        unsafe { crate::system::find_entry::entries(table, count) },
        value,
    );
    let (at, n) = left(table, count, found);
    (found.is_some(), at, n)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.ebx == 0 && r.ecx != 0 {
        return Err(crate::errors::BadInput("a null table").into());
    }
    let (found, ebx, ecx) = find_entry_by_value(r.eax, r.ebx, r.ecx);
    let left = Left::nothing().with(EBX, ebx).with(ECX, ecx);
    Ok(Left {
        carry: Some(!found),
        // found: `cmp eax, [ebx+4]` equal; not: the `dec ecx` that reached 0, or `cmp ecx, 1` on 0
        zero: Some(found || r.ecx != 0),
        ..left
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
