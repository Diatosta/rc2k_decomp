//! `install::draw_string_list` (0x402370): a list of `eax` records (16 bytes each: dx, dy,
//! font, string id) drawn, each at (dx + x, dy + y) with its font (`text::draw_string_by_id`).
//! The x and y come in ecx and edx (the caller's, read back from the saved slots).
//!
//! In:  eax the count, ebx the table, ecx x, edx y
//! Out: every register kept (`pusha` / `popa`)
//! Callers `install::install_files` and the copy-progress and page frames. Refuses a count of 0 (the original's loop is a do-while: it would draw
//! 2^32 records).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{draw_string, rd};
use rc2k_core::install::layout;

/// `count` records of `table` drawn at (x, y) on.
pub fn draw_string_list(count: u32, table: u32, x: u32, y: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::install::layout` (`list_records`, `list_place`).
    for (offset, _) in layout::list_records(count)? {
        let record = table.wrapping_add(offset);
        let (x, y) = layout::list_place(rd(record), rd(record + 4), x, y);
        draw_string(rd(record + 0xC), rd(record + 8), x, y)?;
    }
    Ok(())
}
