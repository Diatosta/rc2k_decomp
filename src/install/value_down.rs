//! `install::value_down` (0x414881): an item of a value (kind 3) lowered by the key's
//! repeats (`g_listRepeats`), not below its least (`[esi+0Ch]`, signed compare).
//!
//! In:  esi the item (+8 the value's byte, +0xC the least)
//! Out: edx the value's address, ebx the least, ecx the new value; zero set, carry clear
//! Callers `install::key_left`. Shadow mode compares. Self-test: items of the test's
//! against the original.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{REPEATS, g, rb, rd, wb};
use crate::race::viewer::regs_entry;

pub fn value_down(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.edx = rd(r.esi + 8);
    r.ebx = rb(r.esi + 0xC) as u32;
    // Converted: `rc2k_core::install::items::value_down`.
    r.ecx = rc2k_core::install::items::value_down(rb(r.edx), g(REPEATS), r.ebx as u8);
    wb(r.edx, r.ecx as u8);
    r.set_zero(true);
    r.set_carry(false);
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &value_down)
}
