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
use crate::install::installer::{draw_string, rd, rl};
use rc2k_core::install::layout;

/// `count` records of `table` drawn at (x, y) on.
pub fn draw_string_list(
    count: u32,
    table: impl Into<crate::place::Place>,
    x: u32,
    y: u32,
) -> Result<(), Refused> {
    let table: crate::place::Place = table.into();
    // Converted: `rc2k_core::install::layout` (`list_records`, `list_place`).
    for (offset, _) in layout::list_records(count)? {
        let record = table.add(offset);
        let (x, y) = layout::list_place(rd(record), rd(record.add(4)), x, y);
        let font = rl(record.add(8))
            .font()
            .unwrap_or_else(|e| crate::install::installer::refuse(e));
        draw_string(rd(record.add(0xC)), font, x, y)?;
    }
    Ok(())
}
