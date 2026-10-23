//! `install::choice_by_number` (0x41486A): an item of choices (kind 2) set to choice ecx (a
//! number key's), if there is one and it is not marked.
//!
//! In:  esi the item, ecx the choice (`install::key_number`: the key less 2)
//! Out: edx the chosen byte's address, ebx the table; zero set, carry clear
//! The compare is of cl with the choices; the mark is read at `[ebx+ecx*2+1]`, all of ecx.
//! Callers `install::key_number`. Shadow mode compares. Self-test: items of the test's
//! against the original, ecx 0 .. 8.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{rb, rd, wb};
use crate::race::viewer::regs_entry;
use rc2k_core::install::items::{self, MARKS};

pub fn choice_by_number(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.edx = rd(r.esi + 8);
    r.ebx = rd(r.esi + 0xC);
    // Converted: `rc2k_core::install::items::choice_by_number`.
    let free = || rb(r.ebx.wrapping_add(r.ecx.wrapping_mul(2)).wrapping_add(1)) & MARKS == 0;
    if let Some(choice) = items::choice_by_number(r.ecx, rb(r.esi + 1), free) {
        wb(r.edx, choice);
    }
    r.set_zero(true);
    r.set_carry(false);
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &choice_by_number)
}
