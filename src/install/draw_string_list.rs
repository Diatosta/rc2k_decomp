//! A list of `count` records (16 bytes each: dx, dy, font, string id) drawn, each at (dx + x, dy +
//! y) with its font (`text::draw_string_by_id`). Refuses a count of 0.

use crate::errors::Refused;
use crate::install::installer::draw_string;
use crate::install::vars::StringRecord;
use rc2k_core::install::layout;

/// The records of `table` drawn at (x, y) on. An empty table is refused.
pub(super) fn draw_string_list(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    table: &[&StringRecord],
    x: u32,
    y: u32,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    for (record, _) in table.iter().zip(layout::list_records(table.len() as u32)?) {
        let (x, y) = layout::list_place(record.dx.get(), record.dy.get(), x, y);
        let font = record
            .font_link()
            .font()
            .unwrap_or_else(|e| crate::install::installer::refuse(e));
        draw_string(so, record.string.get(), font, x, y, gfx)?;
    }
    Ok(())
}
