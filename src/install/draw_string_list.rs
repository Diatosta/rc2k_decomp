//! `install::draw_string_list` (0x402370): a list of `count` records (16 bytes each: dx, dy,
//! font, string id) drawn, each at (dx + x, dy + y) with its font (`text::draw_string_by_id`).
//! //!
//! Takes the count, table, x, y.
//! Callers `install::install_files` and the copy-progress and page frames. Refuses a count of 0 (the original's loop is a do-while: it would draw
//! 2^32 records).

use crate::errors::Refused;
use crate::install::installer::draw_string;
use crate::install::page_links::PageLink;
use crate::install::vars::StringRecord;
use rc2k_core::install::layout;

/// The records of `table` drawn at (x, y) on. An empty table is refused.
pub fn draw_string_list(table: &[&StringRecord], x: u32, y: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::install::layout` (`list_records`, `list_place`).
    for (record, _) in table.iter().zip(layout::list_records(table.len() as u32)?) {
        let (x, y) = layout::list_place(record.dx.get(), record.dy.get(), x, y);
        let font = PageLink::from_raw(record.font.get())
            .font()
            .unwrap_or_else(|e| crate::install::installer::refuse(e));
        draw_string(record.string.get(), font, x, y)?;
    }
    Ok(())
}
