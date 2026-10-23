//! `install::blit_glyph_list` (0x402628): a list of `eax` records (16 bytes each: dx, dy,
//! glyph, palette) blitted, each at (dx + x, dy + y) with its palette (`gfx::blit_glyph`).
//! The x and y come in ecx and edx.
//!
//! In:  eax the count, ebx the table, ecx x, edx y
//! Out: every register kept (`pusha` / `popa`)
//! Caller the copy-progress frame (the install bar). The blits through `part`. Shadow mode
//! runs the original alone. Refuses a count of 0 (the original's loop is a do-while: it
//! would blit 2^32 records). Self-test: a copy whose blit goes to a stand-in, against the
//! Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{BLIT_GLYPH, at, rd};
use crate::race::viewer::regs_entry;
use rc2k_core::install::layout;

pub fn blit_glyph_list(entry: &Regs) -> Result<Regs, Refused> {
    let (x, y) = (entry.ecx, entry.edx);
    // Converted: `rc2k_core::install::layout` (`list_records`, `list_place`).
    for (offset, left) in layout::list_records(entry.eax)? {
        let record = entry.ebx.wrapping_add(offset);
        let (x, y) = layout::list_place(rd(record), rd(record + 4), x, y);
        let mut r = Regs {
            ecx: x,
            edx: y,
            eax: rd(record + 8),
            ebx: rd(record + 0xC),
            edi: record.wrapping_add(layout::LIST_RECORD),
            ebp: left,
            ..*entry
        };
        crate::install::installer::game_part(at(BLIT_GLYPH), &mut r)?;
    }
    Ok(*entry)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| blit_glyph_list(r))
}
