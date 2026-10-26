//! `install::start_edit` (0x4148D6): an edit item (kind 4) begins to be edited: its record's
//! bit 0 set, its text cut at its most bytes (`[record+1]`: a NUL there) and that and two
//! bytes more kept in `byte_95EAF8` (for Escape, `install::edit_key`).
//!
//! In:  esi the item (+8 the text, +0xC the record: +0 flags, +1 the most bytes)
//! Out: every register kept (`pusha` / `popa`); zero set, carry clear (`cmp eax, eax`)
//! Callers `install::item_enter`, `install::next_edit`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::installer::{rb, rd, wb};
use crate::install::vars::INSTALLER_PATHS;

pub fn start_edit(item: u32) {
    let record = rd(item + 0xC);
    let most = rb(record + 1);
    let text = rd(item + 8);
    // Converted: `rc2k_core::install::edits::start_edit`, on the text's `most` + 2 bytes.
    let at = crate::pointers::GamePtr::new(text);
    let mut words = at
        .read_bytes(most as usize + 2, "an edited text")
        .unwrap_or_else(|e| crate::install::installer::refuse(e));
    let flags = rc2k_core::install::edits::start_edit(rb(record), most, &mut words);
    at.write_bytes(&words, "an edited text")
        .unwrap_or_else(|e| crate::install::installer::refuse(e));
    wb(record, flags);
    let backup = INSTALLER_PATHS.script_path.addr();
    for (n, &b) in words.iter().enumerate() {
        wb(backup + n as u32, b);
    }
}
