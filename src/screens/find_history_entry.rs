//! `screens::find_history_entry` (0x4A256C): from championship history entry edi on (0x78
//! bytes each), the first of class ebx and position eax; the entry edi points at is looked at
//! before the end test (-1); none is fatal error 0x8070A7FA ("Position in GB_ChampHST Not
//! Found"), raised by a `jmp` to `fatal::report`.
//!
//! In:  edi = entry, ebx = class, eax = position
//! Out: edi = the entry found; ZF set (no caller reads it); the rest untouched
//!
//! Caller: `screens::draw_championship_table` (0x4A502D). The loop jumps back to the first
//! byte: `.private_copy(0x18)`, the `jmp` to the fatal block relocated. Shadow mode: by state.
//! Self-test: a test table, hits on the first, a middle and the last entry (the missing case
//! is not run: the original ends the game).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EDI, Regs};
use crate::screens::championship_table::find_history_entry;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EDI, unsafe { find_history_entry(r.edi, r.ebx, r.eax) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
