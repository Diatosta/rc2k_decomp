//! `install::draw_string_list` (0x402370): a list of `eax` records (16 bytes each: dx, dy,
//! font, string id) drawn, each at (dx + x, dy + y) with its font (`text::draw_string_by_id`).
//! The x and y come in ecx and edx (the caller's, read back from the saved slots).
//!
//! In:  eax the count, ebx the table, ecx x, edx y
//! Out: every register kept (`pusha` / `popa`)
//! Callers `install::install_files` and the copy-progress and page frames. The draws
//! through `part`. Refuses a count of 0 (the original's loop is a do-while: it would draw
//! 2^32 records). Shadow mode runs the original alone (drawing twice would double it).
//! Self-test: a copy whose draw goes to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{DRAW_STRING, at, rd};
use crate::race::viewer::regs_entry;
use rc2k_core::install::layout;

pub fn draw_string_list(entry: &Regs) -> Result<Regs, Refused> {
    let (x, y) = (entry.ecx, entry.edx);
    // Converted: `rc2k_core::install::layout` (`list_records`, `list_place`).
    for (offset, left) in layout::list_records(entry.eax)? {
        let record = entry.ebx.wrapping_add(offset);
        let (x, y) = layout::list_place(rd(record), rd(record + 4), x, y);
        let mut r = Regs {
            ecx: x,
            edx: y,
            ebx: rd(record + 8),
            eax: rd(record + 0xC),
            edi: record.wrapping_add(layout::LIST_RECORD),
            ebp: left,
            ..*entry
        };
        crate::install::installer::game_part(at(DRAW_STRING), &mut r)?;
    }
    Ok(*entry)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| draw_string_list(r))
}
