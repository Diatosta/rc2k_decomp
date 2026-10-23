//! `install::choice_back` (0x414826): an item of choices (kind 2) moved to the choice before
//! the chosen one that is not marked (bits 6, 7 of its second byte), round from the first
//! to the last; none: nothing changes.
//!
//! In:  esi the item (+1 the choices, +8 the chosen byte, +0xC the choices, 2 bytes each)
//! Out: edx the chosen byte's address, ebx the table, ecx the choice reached (byte); zero
//!      set, carry clear (`cmp eax, eax`)
//! The step down is `dec cl; jns`: from 0x80 .. 0xFF (and 0) it goes to the last choice.
//! A chosen byte past the choices with every choice marked never ends: refused (the
//! original goes round for ever). Callers `install::key_up`, `install::key_left`. Shadow
//! mode compares. Self-test: items of the test's against the original.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{rb, rd, wb};
use crate::race::viewer::regs_entry;
use rc2k_core::install::items::{self, MARKS};

/// An item of choices' places: the chosen byte (edx) and the table (ebx); and whether
/// choice `cl` is free.
pub fn choice_places(r: &mut Regs) -> impl Fn(u8) -> bool + use<> {
    r.edx = rd(r.esi + 8);
    r.ebx = rd(r.esi + 0xC);
    let table = r.ebx;
    move |cl: u8| rb(table.wrapping_add(2 * cl as u32 + 1)) & MARKS == 0
}

/// The choice reached stored (when it changed) and left in ecx; zero set, carry clear.
pub fn store_choice(r: &mut Regs, chosen: u8, cl: u8) {
    if cl != chosen {
        wb(r.edx, cl);
    }
    r.ecx = cl as u32;
    r.set_zero(true);
    r.set_carry(false);
}

pub fn choice_back(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    let free = choice_places(&mut r);
    let chosen = rb(r.edx);
    // Converted: `rc2k_core::install::items::choice_back`.
    let cl = items::choice_back(chosen, || rb(entry.esi + 1), free)?;
    store_choice(&mut r, chosen, cl);
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &choice_back)
}
