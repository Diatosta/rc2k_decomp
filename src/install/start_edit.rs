//! `install::start_edit` (0x4148D6): an edit item (kind 4) begins to be edited: its record's
//! bit 0 set, its text cut at its most bytes (`[record+1]`: a NUL there) and that and two
//! bytes more kept in `byte_95EAF8` (for Escape, `install::edit_key`).
//!
//! In:  esi the item (+8 the text, +0xC the record: +0 flags, +1 the most bytes)
//! Out: every register kept (`pusha` / `popa`); zero set, carry clear (`cmp eax, eax`)
//! Callers `install::item_enter`, `install::next_edit`. Shadow mode compares. Self-test:
//! lists of the test's against the original.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{SCRIPT_PATH, a, rb, rd, wb};
use crate::race::viewer::regs_entry;

pub fn start_edit(entry: &Regs) -> Regs {
    let record = rd(entry.esi + 0xC);
    let most = rb(record + 1);
    let text = rd(entry.esi + 8);
    // Converted: `rc2k_core::install::edits::start_edit`, on the text's `most` + 2 bytes.
    let words = unsafe { std::slice::from_raw_parts_mut(text as *mut u8, most as usize + 2) };
    let flags = rc2k_core::install::edits::start_edit(rb(record), most, words);
    wb(record, flags);
    let backup = a(SCRIPT_PATH);
    for (n, &b) in words.iter().enumerate() {
        wb(backup + n as u32, b);
    }
    let mut r = *entry;
    r.set_zero(true);
    r.set_carry(false);
    r
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(start_edit(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}
